mod iid;
#[allow(clippy::module_inception)]
mod interface;
mod interface_impl;
mod interfaces;
mod vptr;
pub mod vtable;

pub use interface::{Interface, InterfaceMethod};
pub use interfaces::Interfaces;
use proc_macro2::{Ident, TokenStream};
use syn::Path;

use std::collections::HashMap;
use std::iter::FromIterator;

// Expansion entry point
pub fn expand_interfacess(interfaces: Interfaces) -> TokenStream {
    let mut out: Vec<TokenStream> = Vec::new();
    for interface in interfaces.inner {
        out.push(interface.to_struct_tokens());
        out.push(vtable::generate(&interface).unwrap_or_else(|e| e.to_compile_error()));
        out.push(vptr::generate(&interface));
        out.push(interface_impl::generate(&interface));
        out.push(interface.to_iid_tokens());
    }
    out.extend(convert_impls(interfaces.parents));

    TokenStream::from_iter(out)
}

fn convert_impls(parents: HashMap<Ident, Path>) -> Vec<TokenStream> {
    let mut result = Vec::new();
    let interfaces: Vec<Ident> = parents.keys().cloned().collect();

    for interface in interfaces {
        let name = interface;
        let mut current = &name;
        while let Some(p) = parents.get(current) {
            result.push(quote::quote! {
                impl ::core::convert::From<#name> for #p {
                    fn from(this: #name) -> Self {
                        unsafe { ::core::mem::transmute(this) }
                    }
                }
                impl <'a> ::core::convert::From<&'a #name> for &'a #p {
                    fn from(this: &'a #name) -> Self {
                        unsafe { ::core::mem::transmute(this) }
                    }
                }
                #[allow(clippy::from_over_into)]
                impl <'a> ::core::convert::Into<::com::Param<'a, #p>> for #name {
                    fn into(self) -> ::com::Param<'a, #p> {
                        ::com::Param::Owned(self.into())
                    }
                }
                #[allow(clippy::from_over_into)]
                impl <'a> ::core::convert::Into<::com::Param<'a, #p>> for &'a #name {
                    fn into(self) -> ::com::Param<'a, #p> {
                        ::com::Param::Borrowed(self.into())
                    }
                }
            });
            match p.get_ident() {
                Some(n) => current = n,
                None => break,
            }
        }
    }
    result
}
