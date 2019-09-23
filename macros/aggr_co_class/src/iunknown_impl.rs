use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::ItemStruct;

/// For an aggregable COM object, the default IUnknown implementation is
/// always the delegating IUnknown implementation. This will always
/// delegate to the interface pointer at __iunknown_to_use.
pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let iunknown_to_use_field_ident = macro_utils::iunknown_to_use_field_ident();
    let ptr_casting = quote! { as *mut winapi::ctypes::c_void };

    quote!(
        impl com::interfaces::iunknown::IUnknown for #struct_ident {
            unsafe fn query_interface(
                &self,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void
            ) -> winapi::shared::winerror::HRESULT {
                let iunknown_to_use = com::InterfacePtr::<dyn com::interfaces::iunknown::IUnknown>::new(self.#iunknown_to_use_field_ident #ptr_casting);
                iunknown_to_use.query_interface(riid, ppv)
            }

            fn add_ref(&self) -> u32 {
                let iunknown_to_use  = unsafe { com::InterfacePtr::<dyn com::interfaces::iunknown::IUnknown>::new(self.#iunknown_to_use_field_ident #ptr_casting) };
                iunknown_to_use.add_ref()
            }

            unsafe fn release(&self) -> u32 {
                let iunknown_to_use = com::InterfacePtr::<dyn com::interfaces::iunknown::IUnknown>::new(self.#iunknown_to_use_field_ident #ptr_casting);
                iunknown_to_use.release()
            }
        }
    )
}
