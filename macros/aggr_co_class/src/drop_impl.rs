use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemStruct};

pub fn generate(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();
    let box_from_raws = base_interface_idents.iter().map(|base| {
        let vptr_field_ident = macro_utils::vptr_field_ident(&base);
        quote!(
            Box::from_raw(self.#vptr_field_ident as *mut <dyn #base as com::ComInterface>::VTable);
        )
    });

    let aggregate_drops = aggr_map.iter().map(|(aggr_field_ident, _)| {
        quote!(
            if !self.#aggr_field_ident.is_null() {
                let mut aggr_interface_ptr: com::ComPtr<dyn com::IUnknown> = com::ComPtr::new(self.#aggr_field_ident as *mut winapi::ctypes::c_void);
                aggr_interface_ptr.release();
                core::mem::forget(aggr_interface_ptr);
            }
        )
    });

    quote!(
        impl std::ops::Drop for #struct_ident {
            fn drop(&mut self) {
                let _ = unsafe {
                    #(#aggregate_drops)*
                    #(#box_from_raws)*
                    Box::from_raw(self.#non_delegating_iunknown_field_ident as *mut <dyn com::IUnknown as com::ComInterface>::VTable)
                };
            }
        }
    )
}
