mod com_interface_impl;
mod iid;
mod interface;
mod interface_impl;
mod interfaces;
mod vptr;
mod vtable;
mod vtable_macro;

pub use interface::Interface;
pub use interfaces::Interfaces;
use proc_macro2::{Ident, TokenStream};
use syn::{ItemStruct, Path};

use std::collections::HashMap;
use std::iter::FromIterator;

// Expansion entry point
pub fn expand_com_interfaces(interfaces: Interfaces) -> TokenStream {
    let mut out: Vec<TokenStream> = Vec::new();
    for interface in interfaces.inner {
        out.push(interface.to_struct_tokens());
        out.push(vtable::generate(&interface));
        out.push(vptr::generate(&interface));
        out.push(interface_impl::generate(&interface));
        out.push(com_interface_impl::generate(&interface));
        out.push(interface.to_iid_tokens());
    }
    out.extend(convert_impls(interfaces.parents));

    TokenStream::from_iter(out)
}

pub fn expand_derive(input: ItemStruct) -> TokenStream {
    vtable_macro::generate(&input).into()
}

fn convert_impls(parents: HashMap<Ident, Path>) -> Vec<TokenStream> {
    let mut result = Vec::new();
    let interfaces: Vec<Ident> = parents.keys().cloned().collect();

    for interface in interfaces {
        let name = interface;
        let mut current = &name;
        while let Some(p) = parents.get(current) {
            result.push(quote::quote! {
                impl ::std::convert::From<#name> for #p {
                    fn from(this: #name) -> Self {
                        unsafe { ::std::mem::transmute(this) }
                    }
                }
                impl <'a> ::std::convert::From<&'a #name> for &'a #p {
                    fn from(this: &'a #name) -> Self {
                        unsafe { ::std::mem::transmute(this) }
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
