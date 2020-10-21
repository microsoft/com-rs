use com_macros_support::interface::expand_interfacess;
use com_macros_support::Class;
use com_macros_support::Interfaces;

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn interfaces(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as Interfaces);

    expand_interfacess(input).into()
}

#[proc_macro]
pub fn class(input: TokenStream) -> TokenStream {
    let class = syn::parse_macro_input!(input as Class);
    class.to_tokens().into()
}
