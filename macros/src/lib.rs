use aggr_co_class::expand_aggr_co_class;
use co_class::expand_co_class;
use com_interface_attribute::{expand_com_interface, expand_derive};

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
#[proc_macro_attribute]
pub fn co_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_co_class(attr, item)
}

#[proc_macro_attribute]
pub fn aggr_co_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_aggr_co_class(attr, item)
}

#[proc_macro_attribute]
pub fn com_implements(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn aggr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
