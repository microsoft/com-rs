use com_interface_attribute::{expand_com_interface, expand_derive,};
use co_class_derive::expand_derive_com_class;
use aggr_co_class_derive::expand_derive_aggr_com_class;

extern crate proc_macro;
use proc_macro::TokenStream;

// All the Macro exports declared here. Delegates to respective crate for expansion.
#[proc_macro_attribute]
pub fn com_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_com_interface(attr, item)
}

#[proc_macro_derive(VTableMacro)]
pub fn derive(input: TokenStream) -> TokenStream {
    expand_derive(input)
}

// Macro entry points.
#[proc_macro_derive(CoClass, attributes(com_implements, aggr))]
pub fn derive_com_class(item: TokenStream) -> TokenStream {
    expand_derive_com_class(item)
}

#[proc_macro_derive(AggrCoClass, attributes(com_implements, aggr))]
pub fn derive_aggr_com_class(item: TokenStream) -> TokenStream {
    expand_derive_aggr_com_class(item)
}