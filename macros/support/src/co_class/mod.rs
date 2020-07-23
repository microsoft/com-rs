use proc_macro2::{Ident, TokenStream};

use std::collections::HashMap;
use std::iter::FromIterator;

pub mod class_factory;
pub mod co_class_impl;
pub mod com_struct;
pub mod com_struct_impl;
pub mod iunknown_impl;

pub struct CoClass {
    name: Ident,
    docs: Vec<syn::Attribute>,
    visibility: syn::Visibility,
    interfaces: std::collections::HashMap<syn::Path, Interface>,
    methods: std::collections::HashMap<syn::Path, Vec<syn::ImplItemMethod>>,
    fields: Vec<syn::Field>,
}

struct Interface {
    path: syn::Path,
    parent: Option<Box<Interface>>,
}

impl Interface {
    /// Creates an intialized VTable for the interface
    fn to_initialized_vtable_tokens(&self, co_class: &CoClass, offset: usize) -> TokenStream {
        let mut already_initialized = std::collections::HashSet::new();
        let vtable_ident = self.vtable_ident();
        let vtable_type = self.to_vtable_type_tokens();
        let iunknown_path = &syn::parse_str::<syn::Path>("IUknown").unwrap();
        let parent = match self
            .parent
            .as_ref() {
                Some(p) if already_initialized.get(&p.path).is_none() => {
                    already_initialized.insert(&p.path);
                    Some(p.to_initialized_vtable_tokens(co_class, offset))
                }
                None if already_initialized.get(iunknown_path).is_none() => {
                    already_initialized.insert(iunknown_path);
                    Some(Self::iunknown_tokens(co_class, offset))
                }
                _ => None
            };
        let fields = co_class.methods.get(&self.path).unwrap().iter().map(|m| {
            let name = &m.sig.ident;
            let params = m.sig.inputs.iter().filter_map(|p| 
                match p {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(p) => Some(p),
                }
            );
            let ret = &m.sig.output;
            let co_class_name = &co_class.name;
            let method = quote::quote! {
                unsafe extern "stdcall" fn #name(this: ::std::ptr::NonNull<::std::ptr::NonNull<#vtable_ident>>, #(#params),*) #ret {
                    let this = this.as_ptr().sub(#offset);
                    #co_class_name::#name(&*(this as *mut #co_class_name), )
                }
            };
            quote::quote! {
                #name: {
                    #method
                    #name
                }
            }
        });
        let parent = parent.as_ref().map(|p| quote::quote! { parent: #parent, });
        quote::quote! {
            {
                #vtable_type
                #vtable_ident {
                    #parent
                    #(#fields),*
                }
            }
        }
    }

    fn to_vtable_type_tokens(&self) -> TokenStream {
        let name = &self.path;
        let vtable_ident = self.vtable_ident();
        quote::quote! {
            type #vtable_ident = <#name as ::com::ComInterface>::VTable;
        }
    }

    fn vtable_ident(&self) -> proc_macro2::Ident {
        let name = &self.path;
        quote::format_ident!("{}VTable", name.get_ident().unwrap())
    }

    fn iunknown_tokens(co_class: &CoClass, offset: usize) -> TokenStream {
        let name = &co_class.name;
        let interfaces = &co_class.interfaces.keys().collect::<Vec<_>>();
        let iunknown = iunknown_impl::IUnknown::new(name.clone(), offset);
        let add_ref_std = iunknown.to_add_ref_stdcall_tokens();
        let add_ref = iunknown.to_add_ref_tokens();
        let release_std = iunknown.to_release_stdcall_tokens();
        let release = iunknown.to_release_tokens(interfaces);
        let query_interface_std = iunknown.to_query_interface_stdcall_tokens(interfaces);
        let query_interface = iunknown.to_query_interface_tokens(interfaces);
        quote::quote! {
            {
                type IUknownVTable = <::com::interfaces::IUnknown as ::com::ComInterface>::VTable;
                #add_ref_std
                #release_std
                #query_interface_std
                impl #name {
                    #add_ref
                    #release
                    #query_interface
                }
                IUknownVTable {
                    AddRef: add_ref,
                    Release: release,
                    QueryInterface: query_interface,
                }
            }
        }
    }
}

impl syn::parse::Parse for CoClass {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut co_class = None;
        while !input.is_empty() {
            let docs = input.call(syn::Attribute::parse_outer)?;
            //TODO: ensure only docs attributes
            if !input.peek(syn::Token!(impl)) {
                let mut interfaces = HashMap::new();
                let visibility = input.parse::<syn::Visibility>()?;
                let _ = input.parse::<keywords::coclass>()?;
                let name = input.parse::<Ident>()?;
                let _ = input.parse::<syn::Token!(:)>()?;
                while !input.peek(syn::token::Brace) {
                    let path = input.parse::<syn::Path>()?;
                    let interface = Interface {
                        path: path.clone(),
                        parent: None,
                    };
                    interfaces.insert(path.clone(), interface);

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
                        syn::Field::parse_named
                    )?;
                let fields = fields.into_iter().collect();

                co_class = Some(CoClass {
                    name,
                    docs,
                    visibility,
                    interfaces,
                    methods: HashMap::new(),
                    fields,
                });
            } else {

                let item = input.parse::<syn::ItemImpl>()?;
                // TODO: ensure that co_class idents line up
                let (_, interface, _) = item.trait_.unwrap();
                let methods = item
                    .items
                    .into_iter()
                    .map(|i| match i {
                        syn::ImplItem::Method(m) => m,
                        _ => panic!(""),
                    })
                    .collect::<Vec<_>>();
                let co_class = co_class.as_mut().unwrap();

                // ensure not already there
                co_class.methods.insert(interface, methods);
            }
        }
        Ok(co_class.unwrap())
    }
}

impl CoClass {
    pub fn to_tokens(&self) -> TokenStream {
        let mut out: Vec<TokenStream> = Vec::new();
        out.push(com_struct::generate(self));

        out.push(com_struct_impl::generate(self));

        // out.push(co_class_impl::generate(self));

        // out.push(class_factory::generate(self).into());

        TokenStream::from_iter(out)
    }
}

mod keywords {
    syn::custom_keyword!(coclass);
}
