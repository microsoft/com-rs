use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use std::collections::HashMap;
use std::iter::FromIterator;

pub struct CoClass {
    pub name: Ident,
    pub class_factory: bool,
    pub docs: Vec<syn::Attribute>,
    pub visibility: syn::Visibility,
    pub interfaces: HashMap<syn::Path, Interface>,
    pub methods: HashMap<syn::Path, Vec<syn::ImplItemMethod>>,
    pub fields: Vec<syn::Field>,
}

impl CoClass {
    pub fn to_tokens(&self) -> TokenStream {
        let mut out: Vec<TokenStream> = Vec::new();
        out.push(self.to_struct_tokens());
        out.push(self.to_co_class_trait_impl_tokens());
        out.push(super::class_factory::generate(self));

        TokenStream::from_iter(out)
    }

    /// Parse the co_class macro syntax (without the `impl`s)
    fn parse_co_class(
        input: syn::parse::ParseStream,
        docs: Vec<syn::Attribute>,
    ) -> syn::Result<Self> {
        let mut interfaces = HashMap::new();
        let visibility = input.parse::<syn::Visibility>()?;
        let class_factory = input.peek(keywords::classfactory);
        if class_factory {
            let _ = input.parse::<keywords::classfactory>()?;
        } else {
            let _ = input.parse::<keywords::coclass>()?;
        };

        let name = input.parse::<Ident>()?;
        let _ = input.parse::<syn::Token!(:)>()?;

        while !input.peek(syn::token::Brace) {
            let path = input.parse::<syn::Path>()?;
            let interface = Interface {
                path: path.clone(),
                parent: None,
            };
            if let Some(_) = interfaces.insert(path.clone(), interface) {
                return Err(syn::Error::new(path.span(), "interface was redefined"));
            }

            let mut current = interfaces.get_mut(&path).unwrap();
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

        Ok(CoClass {
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

        let interfaces = self.interfaces.keys().collect::<Vec<_>>();
        let interface_fields = interfaces.iter().enumerate().map(|(index, base)| {
            let field_ident = quote::format_ident!("__{}", index);
            quote! {
                #field_ident: ::std::ptr::NonNull<<#base as ::com::ComInterface>::VTable>
            }
        });
        let ref_count_ident = crate::utils::ref_count_ident();

        let user_fields = &self.fields;
        let docs = &self.docs;
        let methods = self.methods.values().flat_map(|ms| ms);

        let iunknown = super::iunknown_impl::IUnknown::new(name.clone());
        let add_ref = iunknown.to_add_ref_tokens();
        let release = iunknown.to_release_tokens(&interfaces);
        let query_interface = iunknown.to_query_interface_tokens(&interfaces);
        let constructor = super::co_class_constructor::generate(self);

        quote!(
            #(#docs)*
            #[repr(C)]
            #vis struct #name {
                #(#interface_fields,)*
                #ref_count_ident: ::std::cell::Cell<u32>,
                #(#user_fields),*
            }
            impl #name {
                #constructor
                #(#methods)*
                #add_ref
                #release
                #query_interface
            }
        )
    }

    pub fn to_co_class_trait_impl_tokens(&self) -> TokenStream {
        if self.class_factory {
            return TokenStream::new();
        }

        let name = &self.name;
        let factory = crate::utils::class_factory_ident(name);

        quote! {
            unsafe impl com::production::CoClass for #name {
                type Factory = #factory;
            }
        }
    }
}

impl syn::parse::Parse for CoClass {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut co_class = None;
        let mut methods: HashMap<syn::Path, Vec<syn::ImplItemMethod>> = HashMap::new();
        while !input.is_empty() {
            let docs = input.call(syn::Attribute::parse_outer)?;
            if let Some(a) = docs.iter().find(|a| !a.path.is_ident("doc")) {
                return Err(syn::Error::new(a.path.span(), "Unrecognized attribute"));
            }

            if !input.peek(syn::Token!(impl)) {
                co_class = Some(Self::parse_co_class(input, docs)?);
            } else {
                let item = input.parse::<syn::ItemImpl>()?;
                // TODO: ensure that co_class idents line up
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
        let mut co_class = match co_class {
            Some(c) => c,
            None => {
                return Err(syn::Error::new(
                    input.span().clone(),
                    "no coclass was defined",
                ));
            }
        };
        co_class.methods = methods;
        Ok(co_class)
    }
}

mod keywords {
    syn::custom_keyword!(coclass);
    syn::custom_keyword!(classfactory);
}

pub struct Interface {
    path: syn::Path,
    parent: Option<Box<Interface>>,
}

impl Interface {
    /// Creates an intialized VTable for the interface
    pub fn to_initialized_vtable_tokens(&self, co_class: &CoClass, offset: usize) -> TokenStream {
        let co_class_name = &co_class.name;
        let vtable_ident = self.vtable_ident();
        let vtable_type = self.to_vtable_type_tokens();
        let parent = match self.parent.as_ref() {
            Some(p) => p.to_initialized_vtable_tokens(co_class, offset),
            None => Self::iunknown_tokens(co_class, offset),
        };
        let fields = co_class.methods.get(&self.path).unwrap().iter().map(|m| {
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
                    #co_class_name::#name(&*(this as *mut #co_class_name), #(#args),*)
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
            type #vtable_ident = <#name as ::com::ComInterface>::VTable;
        }
    }

    fn vtable_ident(&self) -> proc_macro2::Ident {
        let name = &self.path;
        quote::format_ident!("{}VTable", name.get_ident().unwrap())
    }

    fn iunknown_tokens(co_class: &CoClass, offset: usize) -> TokenStream {
        let name = &co_class.name;
        let iunknown = super::iunknown_impl::IUnknownAbi::new(name.clone(), offset);
        let add_ref = iunknown.to_add_ref_tokens();
        let release = iunknown.to_release_tokens();
        let query_interface = iunknown.to_query_interface_tokens();
        quote! {
            {
                type IUknownVTable = <::com::interfaces::IUnknown as ::com::ComInterface>::VTable;
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
