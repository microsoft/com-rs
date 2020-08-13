use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub struct Class {
    pub name: Ident,
    pub class_factory: bool,
    pub docs: Vec<syn::Attribute>,
    pub visibility: syn::Visibility,
    pub interfaces: Vec<Interface>,
    pub methods: HashMap<syn::Path, Vec<syn::ImplItemMethod>>,
    pub fields: Vec<syn::Field>,
}

impl Class {
    pub fn to_tokens(&self) -> TokenStream {
        let mut out: Vec<TokenStream> = Vec::new();
        out.push(self.to_struct_tokens());
        out.push(self.to_class_trait_impl_tokens());
        out.push(super::class_factory::generate(self));

        TokenStream::from_iter(out)
    }

    /// Get the paths of all interfaces including parent interfaces
    fn interfaces_paths<'a>(&'a self) -> HashSet<&'a syn::Path> {
        fn get_interface<'a>(interface: &'a Interface, result: &mut HashSet<&'a syn::Path>) {
            result.insert(&interface.path);
            if let Some(i) = &interface.parent {
                get_interface(i, result);
            }
        }
        let mut result = HashSet::new();
        for i in &self.interfaces {
            get_interface(i, &mut result)
        }
        result
    }

    /// Parse the class macro syntax (without the `impl`s)
    fn parse_class(input: syn::parse::ParseStream, docs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut interfaces: Vec<Interface> = Vec::new();
        let visibility = input.parse::<syn::Visibility>()?;
        let class_factory = input.peek(keywords::factory);
        if class_factory {
            let _ = input.parse::<keywords::factory>()?;
        } else {
            let _ = input.parse::<keywords::class>()?;
        };

        let name = input.parse::<Ident>()?;
        let _ = input.parse::<syn::Token!(:)>()?;

        while !input.peek(syn::token::Brace) {
            let path = input.parse::<syn::Path>()?;
            let interface = Interface {
                path: path.clone(),
                parent: None,
            };
            if interfaces.iter().any(|i| i.path == path) {
                return Err(syn::Error::new(path.span(), "interface was redefined"));
            }
            interfaces.push(interface);

            let mut current = interfaces.last_mut().unwrap();
            while input.peek(syn::token::Paren) {
                let contents;
                syn::parenthesized!(contents in input);
                let path = contents.parse::<syn::Path>()?;
                let interface = Interface { path, parent: None };
                current.parent = Some(Box::new(interface));
                current = current.parent.as_mut().unwrap().as_mut();
            }

            if !input.peek(syn::token::Brace) {
                let _ = input.parse::<syn::Token!(,)>()?;
            }
        }
        let fields;
        syn::braced!(fields in input);
        let fields =
            syn::punctuated::Punctuated::<syn::Field, syn::Token!(,)>::parse_terminated_with(
                &fields,
                syn::Field::parse_named,
            )?;
        let fields = fields.into_iter().collect();

        Ok(Class {
            name,
            class_factory,
            docs,
            visibility,
            interfaces,
            methods: HashMap::new(),
            fields,
        })
    }

    /// The COM class object struct and `impl`
    ///
    /// Structure of the object:
    /// ```rust
    /// pub struct ClassName {
    ///     // ..interface vpointers..
    ///     // ..ref count..
    ///     // ..user defined fields..
    /// }
    /// ```
    pub fn to_struct_tokens(&self) -> TokenStream {
        let name = &self.name;
        let vis = &self.visibility;

        let interfaces = &self.interfaces;
        let interface_fields = interfaces.iter().enumerate().map(|(index, interface)| {
            let interface_name = &interface.path;
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                #field_ident: ::std::ptr::NonNull<<#interface_name as ::com::Interface>::VTable>
            }
        });
        let ref_count_ident = crate::utils::ref_count_ident();

        let user_fields = self.fields.iter().map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! {
                #name: ::std::mem::ManuallyDrop<#ty>
            }
        });
        let docs = &self.docs;
        let methods = self.methods.values().flat_map(|ms| ms);

        let iunknown = super::iunknown_impl::IUnknown::new();
        let add_ref = iunknown.to_add_ref_tokens();
        let release = iunknown.to_release_tokens();
        let query_interface = iunknown.to_query_interface_tokens(interfaces);
        let query = iunknown.to_query_tokens();
        let constructor = super::class_constructor::generate(self);
        let default_constructor = super::class_constructor::generate_default(self);
        let unsafe_constructor = super::class_constructor::generate_unsafe(self);
        let interface_drops = interfaces.iter().enumerate().map(|(index, _)| {
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                let _ = ::std::boxed::Box::from_raw(self.#field_ident.as_ptr());
            }
        });
        let user_fields_drops = self.fields.iter().map(|f| {
            let name = &f.ident;
            quote! {
                ::std::mem::ManuallyDrop::drop(&mut self.#name);
            }
        });

        quote! {
            use super::*;
            #(#docs)*
            #[repr(C)]
            #vis struct #name {
                #(#interface_fields,)*
                #ref_count_ident: ::std::cell::Cell<u32>,
                #(#user_fields),*
            }
            impl #name {
                #constructor
                #default_constructor
                #unsafe_constructor
                #(#methods)*
                #add_ref
                #release
                #query_interface
                #query
            }
            impl ::std::ops::Drop for #name {
                fn drop(&mut self) {
                    let new_count = self.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count");
                    self.#ref_count_ident.set(new_count);
                    if new_count == 0 {
                        //Drop everything
                        unsafe {
                            #(#interface_drops)*
                            #(#user_fields_drops)*
                        }
                    }
                }
            }
        }
    }

    pub fn to_class_trait_impl_tokens(&self) -> TokenStream {
        if self.class_factory {
            return TokenStream::new();
        }

        let name = &self.name;
        let factory = crate::utils::class_factory_ident(name);

        quote! {
            unsafe impl com::production::Class for #name {
                type Factory = #factory;
            }
        }
    }
}

impl syn::parse::Parse for Class {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut class = None;
        let mut methods: HashMap<syn::Path, Vec<syn::ImplItemMethod>> = HashMap::new();
        while !input.is_empty() {
            let docs = input.call(syn::Attribute::parse_outer)?;
            if let Some(a) = docs.iter().find(|a| !a.path.is_ident("doc")) {
                return Err(syn::Error::new(a.path.span(), "Unrecognized attribute"));
            }

            if !input.peek(syn::Token!(impl)) {
                class = Some(Self::parse_class(input, docs)?);
            } else {
                let item = input.parse::<syn::ItemImpl>()?;
                // TODO: ensure that class idents line up
                let interface = match item.trait_ {
                    Some((_, i, _)) => i,
                    None => {
                        return Err(syn::Error::new(
                            item.span(),
                            "Impl must be for an interface",
                        ))
                    }
                };
                let ms = item
                    .items
                    .into_iter()
                    .map(|i| match i {
                        syn::ImplItem::Method(m) => Ok(m),
                        _ => Err(syn::Error::new(
                            i.span().clone(),
                            "only trait methods are allowed when implementing an interface",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;

                if let Some(_) = methods.insert(interface.clone(), ms) {
                    return Err(syn::Error::new(
                        interface.span().clone(),
                        "interface was redefined",
                    ));
                }
            }
        }
        let mut class = match class {
            Some(c) => {
                let mut interface_paths = c.interfaces_paths();
                for i in methods.keys() {
                    if !interface_paths.remove(i) {
                        return Err(syn::Error::new(
                            i.span().clone(),
                            "impl for a non-declared interface",
                        ));
                    }
                }
                if let Some(i) = interface_paths.into_iter().next() {
                    return Err(syn::Error::new(
                        i.span().clone(),
                        "impl for interface is missing",
                    ));
                }
                c
            }
            None => {
                return Err(syn::Error::new(
                    input.span().clone(),
                    "no class was defined",
                ));
            }
        };
        class.methods = methods;
        Ok(class)
    }
}

mod keywords {
    syn::custom_keyword!(class);
    syn::custom_keyword!(factory);
}

pub struct Interface {
    pub path: syn::Path,
    pub parent: Option<Box<Interface>>,
}

impl Interface {
    /// Creates an intialized VTable for the interface
    pub fn to_initialized_vtable_tokens(&self, class: &Class, offset: usize) -> TokenStream {
        let class_name = &class.name;
        let vtable_ident = self.vtable_ident();
        let vtable_type = self.to_vtable_type_tokens();
        let parent = match self.parent.as_ref() {
            Some(p) => p.to_initialized_vtable_tokens(class, offset),
            None => Self::iunknown_tokens(class, offset),
        };
        let fields = class.methods.get(&self.path).unwrap().iter().map(|m| {
            let name = &m.sig.ident;
            let params = m.sig.inputs.iter().filter_map(|p| {
                match p {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(p) => Some(p),
                }
            });
            let args = m.sig.inputs.iter().filter_map(|p| {
                match p {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(p) => Some(&p.pat),
                }
            });
            let ret = &m.sig.output;
            let method = quote! {
                unsafe extern "stdcall" fn #name(this: ::std::ptr::NonNull<::std::ptr::NonNull<#vtable_ident>>, #(#params),*) #ret {
                    let this = this.as_ptr().sub(#offset);
                    let this = ::std::mem::ManuallyDrop::new(::std::pin::Pin::new(::std::boxed::Box::from_raw(this as *mut _ as *mut #class_name)));
                    #class_name::#name(&this, #(#args),*)
                }
            };
            let field_name = Ident::new(&crate::utils::snake_to_camel(&name.to_string()), proc_macro2::Span::call_site());
            quote! {
                #field_name: {
                    #method
                    #name
                }
            }
        });
        quote! {
            {
                #vtable_type
                #vtable_ident {
                    parent: #parent,
                    #(#fields),*
                }
            }
        }
    }

    fn to_vtable_type_tokens(&self) -> TokenStream {
        let name = &self.path;
        let vtable_ident = self.vtable_ident();
        quote! {
            type #vtable_ident = <#name as ::com::Interface>::VTable;
        }
    }

    fn vtable_ident(&self) -> proc_macro2::Ident {
        let name = &self.path;
        quote::format_ident!("{}VTable", name.segments.last().unwrap().ident)
    }

    fn iunknown_tokens(class: &Class, offset: usize) -> TokenStream {
        let iunknown = super::iunknown_impl::IUnknownAbi::new(class.name.clone(), offset);
        let add_ref = iunknown.to_add_ref_tokens();
        let release = iunknown.to_release_tokens();
        let query_interface = iunknown.to_query_interface_tokens();
        quote! {
            {
                type IUknownVTable = <::com::interfaces::IUnknown as ::com::Interface>::VTable;
                #add_ref
                #release
                #query_interface
                IUknownVTable {
                    AddRef: add_ref,
                    Release: release,
                    QueryInterface: query_interface,
                }
            }
        }
    }
}
