use crate::{iid, vtable, vtable_macro};

use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::Ident;

pub fn generate(interface_ident: &Ident) -> HelperTokenStream {
    let vtable_ident = vtable::ident(&interface_ident.to_string());
    let iid_ident = iid::ident(interface_ident);
    let vtable_macro = vtable_macro::ident(&interface_ident);

    quote!(
        unsafe impl com::ComInterface for dyn #interface_ident {
            type VTable = #vtable_ident;
            const IID: com::_winapi::shared::guiddef::IID = #iid_ident;
        }

        impl <C: #interface_ident> com::ProductionComInterface<C> for dyn #interface_ident {
            fn vtable<O: com::offset::Offset>() -> Self::VTable {
                #vtable_macro!(C, O)
            }
        }
    )
}
