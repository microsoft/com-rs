use super::Class;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(class: &Class) -> TokenStream {
    if class.class_factory {
        return TokenStream::new();
    }
    let class_factory_ident = crate::utils::class_factory_ident(&class.name);
    let class_name = &class.name;
    quote! {
        ::com::class! {
            pub factory #class_factory_ident: ::com::interfaces::IClassFactory {}

            impl ::com::interfaces::IClassFactory for #class_factory_ident {
                unsafe fn CreateInstance(
                    &self,
                    aggr: *mut std::ptr::NonNull<<::com::interfaces::IUnknown as ::com::Interface>::VTable>,
                    riid: *const com::sys::IID,
                    ppv: *mut *mut std::ffi::c_void,
                ) -> com::sys::HRESULT {
                    if aggr != std::ptr::null_mut() {
                        return com::sys::CLASS_E_NOAGGREGATION;
                    }

                    let mut instance = ::std::mem::ManuallyDrop::new(::std::boxed::Box::pin(<#class_name as ::std::default::Default>::default()));
                    instance.query_interface(riid, ppv)
                }

                unsafe fn LockServer(&self, _increment: com::sys::BOOL) -> com::sys::HRESULT {
                    com::sys::S_OK
                }
            }
        }
    }
}
