use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{Ident, ItemStruct};

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

pub fn generate(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let box_from_raws = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = macro_utils::get_vptr_field_ident(&base);
        quote!(
            Box::from_raw(self.#vptr_field_ident as *mut <dyn #base as com::ComInterface>::VTable);
        )
    });

    quote!(
        impl std::ops::Drop for #real_ident {
            fn drop(&mut self) {
                let _ = unsafe {
                    #(#box_from_raws)*
                };
            }
        }
    )
}
