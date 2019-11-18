use com_macros_support::aggr_co_class::expand_aggr_co_class;
use com_macros_support::co_class::expand_co_class;
use com_macros_support::com_interface::{expand_com_interface, expand_derive};

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{AttributeArgs, ItemStruct, Meta, NestedMeta};

// All the Macro exports declared here. Delegates to respective crate for expansion.
#[proc_macro_attribute]
pub fn com_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_com_interface(attr, item)
}

#[proc_macro_derive(VTable)]
pub fn derive_vtable(input: TokenStream) -> TokenStream {
    expand_derive(input)
}

// Macro entry points.
#[proc_macro_attribute]
pub fn co_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);
    let attr_args = syn::parse_macro_input!(attr as AttributeArgs);
    if is_aggregatable(&attr_args) {
        expand_aggr_co_class(&input, &attr_args)
    } else {
        expand_co_class(&input, &attr_args)
    }
}

fn is_aggregatable(attr_args: &AttributeArgs) -> bool {
    attr_args
        .iter()
        .find(|arg| match arg {
            NestedMeta::Meta(Meta::Path(ref path)) => {
                let segments = &path.segments;
                segments.len() == 1
                    && segments.first().expect("Invalid attribute syntax").ident == "aggregatable"
            }
            _ => false,
        })
        .is_some()
}
