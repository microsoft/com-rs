extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{AttributeArgs, ItemStruct};

use std::iter::FromIterator;

pub mod class_factory;
pub mod com_struct;
pub mod com_struct_impl;
pub mod iunknown_impl;

pub fn expand_co_class(input: &ItemStruct, attr_args: &AttributeArgs) -> TokenStream {
    let base_interface_idents = macro_utils::base_interface_idents(attr_args);
    let aggr_interface_idents = macro_utils::get_aggr_map(attr_args);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(
        com_struct::generate(input, &aggr_interface_idents, &base_interface_idents, false).into(),
    );
    out.push(
        com_struct_impl::generate(&aggr_interface_idents, &base_interface_idents, input).into(),
    );
    out.push(iunknown_impl::generate(&base_interface_idents, &aggr_interface_idents, input).into());
    out.push(class_factory::generate(input).into());

    TokenStream::from_iter(out)
}
