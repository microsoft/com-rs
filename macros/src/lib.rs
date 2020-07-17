use com_macros_support::com_interface::expand_com_interfaces;
use com_macros_support::CoClass;
use com_macros_support::Interfaces;

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::ItemStruct;

// All the Macro exports declared here. Delegates to respective crate for expansion.
#[proc_macro]
pub fn com_interface(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as Interfaces);

    expand_com_interfaces(input).into()
}

#[proc_macro]
pub fn co_class(input: TokenStream) -> TokenStream {
    let co_class = syn::parse_macro_input!(input as CoClass);
    co_class.to_tokens().into()
}
