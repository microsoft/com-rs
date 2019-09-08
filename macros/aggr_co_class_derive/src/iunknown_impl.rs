use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::ItemStruct;

// impl com::IUnknown for LocalFileManager {
//     fn query_interface(
//         &mut self,
//         riid: *const winapi::shared::guiddef::IID,
//         ppv: *mut *mut winapi::ctypes::c_void,
//     ) -> winapi::shared::winerror::HRESULT {
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Delegating QI\n"],
//                 &match () {
//                     () => [],
//                 },
//             ));
//         };
//         let mut iunk_to_use: com::ComPtr<dyn com::IUnknown> =
//             unsafe { com::ComPtr::new(self.__iunk_to_use as *mut winapi::ctypes::c_void) };
//         let hr = iunk_to_use.query_interface(riid, ppv);
//         core::mem::forget(iunk_to_use);
//         hr
//     }
//     fn add_ref(&mut self) -> u32 {
//         let mut iunk_to_use: com::ComPtr<dyn com::IUnknown> =
//             unsafe { com::ComPtr::new(self.__iunk_to_use as *mut winapi::ctypes::c_void) };
//         let res = iunk_to_use.add_ref();
//         core::mem::forget(iunk_to_use);
//         res
//     }
//     fn release(&mut self) -> u32 {
//         let mut iunk_to_use: com::ComPtr<dyn com::IUnknown> =
//             unsafe { com::ComPtr::new(self.__iunk_to_use as *mut winapi::ctypes::c_void) };
//         let res = iunk_to_use.release();
//         core::mem::forget(iunk_to_use);
//         res
//     }
// }

/// For an aggregable COM object, the default IUnknown implementation is
/// always the delegating IUnknown implementation. This will always
/// delegate to the interface pointer at __iunk_to_use.
///
/// TODO: We are always leaking ComPtr, since we do not yet have a struct to
/// represent a non-reference counted interface pointer. Or we could maybe store
/// __iunk_to_use as a ComPtr?
pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let iunk_to_use_field_ident = macro_utils::get_iunk_to_use_field_ident();

    quote!(
        impl com::IUnknown for #real_ident {
            fn query_interface(
                &mut self,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void
            ) -> winapi::shared::winerror::HRESULT {
                println!("Delegating QI");
                let mut iunk_to_use: com::ComPtr<dyn com::IUnknown> = unsafe { com::ComPtr::new(self.#iunk_to_use_field_ident as *mut winapi::ctypes::c_void) };
                let hr = iunk_to_use.query_interface(riid, ppv);
                core::mem::forget(iunk_to_use);

                hr
            }

            fn add_ref(&mut self) -> u32 {
                let mut iunk_to_use: com::ComPtr<dyn com::IUnknown> = unsafe { com::ComPtr::new(self.#iunk_to_use_field_ident as *mut winapi::ctypes::c_void) };
                let res = iunk_to_use.add_ref();
                core::mem::forget(iunk_to_use);

                res
            }

            fn release(&mut self) -> u32 {
                let mut iunk_to_use: com::ComPtr<dyn com::IUnknown> = unsafe { com::ComPtr::new(self.#iunk_to_use_field_ident as *mut winapi::ctypes::c_void) };
                let res = iunk_to_use.release();
                core::mem::forget(iunk_to_use);

                res
            }
        }
    )
}
