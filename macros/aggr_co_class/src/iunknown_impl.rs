use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::ItemStruct;

/// For an aggregable COM object, the default IUnknown implementation is
/// always the delegating IUnknown implementation. This will always
/// delegate to the interface pointer at __iunknown_to_use.
///
/// TODO: We are always leaking ComPtr, since we do not yet have a struct to
/// represent a non-reference counted interface pointer. Or we could maybe store
/// __iunknown_to_use as a ComPtr?
pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let iunknown_to_use_field_ident = macro_utils::iunknown_to_use_field_ident();

    quote!(
        impl com::IUnknown for #struct_ident {
            unsafe fn query_interface(
                &mut self,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void
            ) -> winapi::shared::winerror::HRESULT {
                println!("Delegating QI");
                let mut iunknown_to_use: com::ComPtr<dyn com::IUnknown> = com::ComPtr::new(self.#iunknown_to_use_field_ident as *mut winapi::ctypes::c_void);
                let hr = iunknown_to_use.query_interface(riid, ppv);
                core::mem::forget(iunknown_to_use);

                hr
            }

            fn add_ref(&mut self) -> u32 {
                let mut iunknown_to_use: com::ComPtr<dyn com::IUnknown> = unsafe { com::ComPtr::new(self.#iunknown_to_use_field_ident as *mut winapi::ctypes::c_void) };
                let res = iunknown_to_use.add_ref();
                core::mem::forget(iunknown_to_use);

                res
            }

            unsafe fn release(&mut self) -> u32 {
                let mut iunknown_to_use: com::ComPtr<dyn com::IUnknown> = com::ComPtr::new(self.#iunknown_to_use_field_ident as *mut winapi::ctypes::c_void);
                let res = iunknown_to_use.release();
                core::mem::forget(iunknown_to_use);

                res
            }
        }
    )
}
