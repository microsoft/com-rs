extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{AttributeArgs, ItemStruct};

use std::iter::FromIterator;

mod class_factory;
mod com_struct;
mod com_struct_impl;
mod drop_impl;
mod iunknown_impl;

// Macro expansion entry point.

pub fn expand_aggr_co_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);
    let attr_args = syn::parse_macro_input!(attr as AttributeArgs);

    // Parse attributes
    let base_interface_idents = macro_utils::base_interface_idents(&attr_args);
    let aggr_interface_idents = macro_utils::get_aggr_map(&attr_args);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(com_struct::generate(&aggr_interface_idents, &base_interface_idents, &input).into());
    out.push(
        com_struct_impl::generate(&base_interface_idents, &aggr_interface_idents, &input).into(),
    );
    out.push(iunknown_impl::generate(&input).into());
    out.push(drop_impl::generate(&aggr_interface_idents, &base_interface_idents, &input).into());
    out.push(class_factory::generate(&input).into());

    TokenStream::from_iter(out)
}
