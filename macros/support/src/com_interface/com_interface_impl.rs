use crate::com_interface::{iid, vtable, vtable_macro};

use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{ItemTrait, TypeParamBound};

pub fn generate(trait_item: &ItemTrait) -> HelperTokenStream {
    let interface_ident = &trait_item.ident;
    let vtable_ident = vtable::ident(&interface_ident.to_string());
    let iid_ident = iid::ident(interface_ident);
    let vtable_macro = vtable_macro::ident(&interface_ident);
    let iid_check = quote! { com::_winapi::shared::guiddef::IsEqualGUID(riid, &Self::IID) };
    let recursive_iid_check = if let Some(TypeParamBound::Trait(t)) = trait_item.supertraits.first()
    {
        let supertrait_ident = t.path.get_ident().unwrap();
        quote! { #iid_check || <dyn #supertrait_ident as com::ComInterface>::is_iid_in_inheritance_chain(riid) }
    } else {
        iid_check
    };

    quote! {
        unsafe impl com::ComInterface for dyn #interface_ident {
            type VTable = #vtable_ident;
            const IID: com::_winapi::shared::guiddef::IID = #iid_ident;

            fn is_iid_in_inheritance_chain(riid: &com::_winapi::shared::guiddef::IID) -> bool {
                #recursive_iid_check
            }
        }

        impl <C: #interface_ident> com::ProductionComInterface<C> for dyn #interface_ident {
            fn vtable<O: com::offset::Offset>() -> Self::VTable {
                #vtable_macro!(C, O)
            }
        }
    }
}
