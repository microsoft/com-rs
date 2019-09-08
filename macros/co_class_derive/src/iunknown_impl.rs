use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{ItemStruct, Ident,};
use std::collections::HashMap;

// impl com::IUnknown for BritishShortHairCat {
//     fn query_interface(
//         &mut self,
//         riid: *const winapi::shared::guiddef::IID,
//         ppv: *mut *mut winapi::ctypes::c_void,
//     ) -> winapi::shared::winerror::HRESULT {
//         unsafe {
//             let riid = &*riid;
//             if winapi::shared::guiddef::IsEqualGUID(riid, &com::IID_IUNKNOWN) {
//                 *ppv = &self.__icatvptr as *const _ as *mut winapi::ctypes::c_void;
//             } else if <dyn ICat as com::ComInterface>::iid_in_inheritance_chain(riid) {
//                 *ppv = &self.__icatvptr as *const _ as *mut winapi::ctypes::c_void;
//             } else if <dyn IDomesticAnimal as com::ComInterface>::iid_in_inheritance_chain(riid)
//             {
//                 *ppv = &self.__idomesticanimalvptr as *const _ as *mut winapi::ctypes::c_void;
//             } else {
//                 *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
//                 {
//                     ::std::io::_print(::std::fmt::Arguments::new_v1(
//                         &["Returning NO INTERFACE.\n"],
//                         &match () {
//                             () => [],
//                         },
//                     ));
//                 };
//                 return winapi::shared::winerror::E_NOINTERFACE;
//             }
//             {
//                 ::std::io::_print(::std::fmt::Arguments::new_v1(
//                     &["Successful!.\n"],
//                     &match () {
//                         () => [],
//                     },
//                 ));
//             };
//             self.add_ref();
//             NOERROR
//         }
//     }
//     fn add_ref(&mut self) -> u32 {
//         self.__refcnt += 1;
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Count now ", "\n"],
//                 &match (&self.__refcnt,) {
//                     (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
//                 },
//             ));
//         };
//         self.__refcnt
//     }
//     fn release(&mut self) -> u32 {
//         self.__refcnt -= 1;
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Count now ", "\n"],
//                 &match (&self.__refcnt,) {
//                     (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
//                 },
//             ));
//         };
//         let count = self.__refcnt;
//         if count == 0 {
//             {
//                 ::std::io::_print(::std::fmt::Arguments::new_v1(
//                     &["Count is 0 for ", ". Freeing memory...\n"],
//                     &match (&"BritishShortHairCat",) {
//                         (arg0,) => {
//                             [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)]
//                         }
//                     },
//                 ));
//             };
//             unsafe {
//                 Box::from_raw(self as *const _ as *mut BritishShortHairCat);
//             }
//         }
//         count
//     }
// }

pub fn generate(
    base_itf_idents: &[Ident],
    aggr_itf_idents: &HashMap<Ident, Vec<Ident>>,
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let ref_count_ident = macro_utils::get_ref_count_ident();

    let first_vptr_field = macro_utils::get_vptr_field_ident(&base_itf_idents[0]);

    // Generate match arms for implemented interfaces
    let base_match_arms = base_itf_idents.iter().map(|base| {
        let match_condition =
            quote!(<dyn #base as com::ComInterface>::iid_in_inheritance_chain(riid));
        let vptr_field_ident = macro_utils::get_vptr_field_ident(&base);

        quote!(
            else if #match_condition {
                *ppv = &self.#vptr_field_ident as *const _ as *mut winapi::ctypes::c_void;
            }
        )
    });

    // Generate match arms for aggregated interfaces
    let aggr_match_arms = aggr_itf_idents.iter().map(|(aggr_field_ident, aggr_base_itf_idents)| {

        // Construct the OR match conditions for a single aggregated object.
        let first_base_itf_ident = &aggr_base_itf_idents[0];
        let first_aggr_match_condition = quote!(
            <dyn #first_base_itf_ident as com::ComInterface>::iid_in_inheritance_chain(riid)
        );
        let rem_aggr_match_conditions = aggr_base_itf_idents.iter().skip(1).map(|base| {
            quote!(|| <dyn #base as com::ComInterface>::iid_in_inheritance_chain(riid))
        });

        quote!(
            else if #first_aggr_match_condition #(#rem_aggr_match_conditions)* {
                let mut aggr_itf_ptr: ComPtr<dyn IUnknown> = ComPtr::new(self.#aggr_field_ident as *mut winapi::ctypes::c_void);
                let hr = aggr_itf_ptr.query_interface(riid, ppv);
                if com::failed(hr) {
                    return winapi::shared::winerror::E_NOINTERFACE;
                }

                // We release it as the previous call add_ref-ed the inner object.
                // The intention is to transfer reference counting logic to the
                // outer object.
                aggr_itf_ptr.release();

                forget(aggr_itf_ptr);
            }
        )
    });

    quote!(
        impl com::IUnknown for #real_ident {
            fn query_interface(
                &mut self,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void
            ) -> winapi::shared::winerror::HRESULT {
                unsafe {
                    let riid = &*riid;

                    if winapi::shared::guiddef::IsEqualGUID(riid, &com::IID_IUNKNOWN) {
                        *ppv = &self.#first_vptr_field as *const _ as *mut winapi::ctypes::c_void;
                    } #(#base_match_arms)* #(#aggr_match_arms)* else {
                        *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                        println!("Returning NO INTERFACE.");
                        return winapi::shared::winerror::E_NOINTERFACE;
                    }

                    println!("Successful!.");
                    self.add_ref();
                    NOERROR
                }
            }

            fn add_ref(&mut self) -> u32 {
                self.#ref_count_ident += 1;
                println!("Count now {}", self.#ref_count_ident);
                self.#ref_count_ident
            }

            fn release(&mut self) -> u32 {
                self.#ref_count_ident -= 1;
                println!("Count now {}", self.#ref_count_ident);
                let count = self.#ref_count_ident;
                if count == 0 {
                    println!("Count is 0 for {}. Freeing memory...", stringify!(#real_ident));
                    // drop(self)
                    unsafe { Box::from_raw(self as *const _ as *mut #real_ident); }
                }
                count
            }
        }
    )
    // unimplemented!()
}