use super::Interface;
use crate::com_interface::{iid, vtable};

use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(interface: &Interface) -> TokenStream {
    let interface_ident = &interface.name;
    let vtable_ident = vtable::ident(&interface_ident.to_string());
    let iid_ident = iid::ident(interface_ident);
    // let vtable_macro = vtable_macro::ident(&interface_ident);
    let parent = if let Some(p) = &interface.parent {
        quote! { #p }
    } else {
        quote! { #interface_ident }
    };

    quote! {
        unsafe impl com::ComInterface for #interface_ident {
            type VTable = #vtable_ident;
            type Super = #parent;
            const IID: com::sys::IID = #iid_ident;
        }
    }
}
