extern crate proc_macro;

mod com_interface_impl;
mod comptr_impl;
mod iid;
mod utils;
mod vptr;
mod vtable;
mod vtable_macro;

use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{ItemStruct, ItemTrait};

use std::iter::FromIterator;

mod com_class;
mod aggr_com_class;
use com_class::expand_com_class;
use aggr_com_class::expand_aggregable_com_class;

use utils::*;

// Macro entry points.
#[proc_macro_derive(CoClass, attributes(com_implements, aggr))]
pub fn derive_com_class(item: TokenStream) -> TokenStream {
    expand_com_class(item)
}

#[proc_macro_derive(AggrCoClass, attributes(com_implements, aggr))]
pub fn derive_aggregable_com_class(item: TokenStream) -> TokenStream {
    expand_aggregable_com_class(item)
}


#[proc_macro_attribute]
pub fn com_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemTrait);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(input.to_token_stream().into());
    out.push(vtable::generate(&input).into());
    out.push(vptr::generate(&input.ident).into());
    out.push(comptr_impl::generate(&input).into());
    out.push(com_interface_impl::generate(&input.ident).into());
    out.push(iid::generate(&attr, &input.ident).into());

    TokenStream::from_iter(out)
}

#[proc_macro_derive(VTableMacro)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemStruct);
    vtable_macro::generate(&input).into()
}