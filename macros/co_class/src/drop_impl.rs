use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{Ident, ItemStruct};
use std::collections::HashMap;

//  impl std::ops::Drop for BritishShortHairCat {
//     fn drop(&mut self) {
//         let _ = unsafe {
//             Box::from_raw(self.__icatvptr as *mut <ICat as com::ComInterface>::VTable);
//             Box::from_raw(
//                 self.__idomesticanimalvptr
//                     as *mut <IDomesticAnimal as com::ComInterface>::VTable,
//             );
//         };
//     }
// }

pub fn generate(aggr_map: &HashMap<Ident, Vec<Ident>>, base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let box_from_raws = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = macro_utils::get_vptr_field_ident(&base);
        quote!(
            Box::from_raw(self.#vptr_field_ident as *mut <dyn #base as com::ComInterface>::VTable);
        )
    });

    let aggregate_drops = aggr_map.iter().map(|(aggr_field_ident, _)| {
        quote!(
            if !self.#aggr_field_ident.is_null() {
                let mut aggr_itf_ptr: com::ComPtr<dyn com::IUnknown> = com::ComPtr::new(self.#aggr_field_ident as *mut winapi::ctypes::c_void);
                aggr_itf_ptr.release();
                core::mem::forget(aggr_itf_ptr);
            }
        )
    });

    quote!(
        impl std::ops::Drop for #struct_ident {
            fn drop(&mut self) {
                use com::IUnknown;

                let _ = unsafe {
                    #(#aggregate_drops)*
                    #(#box_from_raws)*
                };
            }
        }
    )
}
