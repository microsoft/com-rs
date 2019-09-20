extern crate proc_macro;

mod com_interface_impl;
mod iid;
mod interface_impl;
mod vptr;
mod vtable;
mod vtable_macro;

use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{ItemStruct, ItemTrait};

use std::iter::FromIterator;

// declare mods.

// Expansion entry point
pub fn expand_com_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemTrait);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(input.to_token_stream().into());
    out.push(vtable::generate(&input).into());
    out.push(vptr::generate(&input.ident).into());
    out.push(interface_impl::generate(&input).into());
    out.push(com_interface_impl::generate(&input).into());
    out.push(iid::generate(&attr, &input.ident).into());

    TokenStream::from_iter(out)
}

pub fn expand_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemStruct);
    vtable_macro::generate(&input).into()
}
