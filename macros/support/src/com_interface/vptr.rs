use super::vtable;
use super::Interface;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn generate(interface: &Interface) -> TokenStream {
    let vptr_ident = ident(&interface.name);
    let vtable_ident = vtable::ident(&interface.name.to_string());

    quote!(
        #[allow(missing_docs)]
        pub type #vptr_ident = ::std::ptr::NonNull<#vtable_ident>;
    )
}

pub fn ident(interface_name: &Ident) -> Ident {
    format_ident!("{}VPtr", interface_name)
}
