use com_macros_support::co_class::expand_co_class;
use com_macros_support::com_interface::{expand_com_interface, expand_derive};
use com_macros_support::Interface;

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{AttributeArgs, ItemStruct};

// All the Macro exports declared here. Delegates to respective crate for expansion.
#[proc_macro]
pub fn com_interface(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as Interfaces);

    input
        .0
        .into_iter()
        .map(|i| expand_com_interface(i))
        .collect::<proc_macro2::TokenStream>()
        .into()
}

#[proc_macro_derive(VTable)]
pub fn derive_vtable(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);
    expand_derive(input).into()
}

// Macro entry points.
#[proc_macro_attribute]
pub fn co_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);
    let attr_args = syn::parse_macro_input!(attr as AttributeArgs);
    expand_co_class(&input, &attr_args).into()
}

struct Interfaces(Vec<Interface>);

impl syn::parse::Parse for Interfaces {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut interfaces = Vec::new();
        while !input.is_empty() {
            interfaces.push(input.parse()?)
        }
        Ok(Self(interfaces))
    }
}
