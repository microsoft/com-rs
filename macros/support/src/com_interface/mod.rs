mod com_interface_impl;
mod iid;
mod interface;
mod interface_impl;
mod vptr;
mod vtable;
mod vtable_macro;

pub use interface::Interface;
use proc_macro2::TokenStream;
use syn::ItemStruct;

use std::iter::FromIterator;

// Expansion entry point
pub fn expand_com_interface(interface: Interface) -> TokenStream {
    let mut out: Vec<TokenStream> = Vec::new();
    out.push(interface.to_struct_tokens());
    out.push(vtable::generate(&interface));
    out.push(vptr::generate(&interface));
    out.push(interface_impl::generate(&interface));
    out.push(com_interface_impl::generate(&interface));
    out.push(interface.to_iid_tokens());

    TokenStream::from_iter(out)
}

pub fn expand_derive(input: ItemStruct) -> TokenStream {
    vtable_macro::generate(&input).into()
}
