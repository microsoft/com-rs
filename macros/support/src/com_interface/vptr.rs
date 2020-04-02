use super::vtable;
use proc_macro2::{Ident, TokenStream as HelperTokenStream};
use quote::{format_ident, quote};

pub fn generate(interface_ident: &Ident) -> HelperTokenStream {
    let vptr_ident = ident(&interface_ident.to_string());
    let vtable_ident = vtable::ident(&interface_ident.to_string());

    quote!(
        #[allow(missing_docs)]
        pub type #vptr_ident = *const #vtable_ident;
    )
}

pub fn ident(interface_name: &str) -> Ident {
    format_ident!("{}VPtr", interface_name)
}
