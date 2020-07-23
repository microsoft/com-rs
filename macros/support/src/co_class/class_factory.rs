use super::CoClass;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(co_class: &CoClass) -> TokenStream {
    if co_class.class_factory {
        return TokenStream::new();
    }
    let class_factory_ident = crate::utils::class_factory_ident(&co_class.name);
    let co_class_name = &co_class.name;
    quote! {
        use ::com::interfaces::IClassFactory;
        ::com::co_class! {
            pub classfactory #class_factory_ident: IClassFactory {}

            impl IClassFactory for #class_factory_ident {
                fn CreateInstance(
                    &self,
                    aggr: *mut std::ptr::NonNull<<::com::interfaces::IUnknown as ::com::ComInterface>::VTable>,
                    riid: *const com::sys::IID,
                    ppv: *mut *mut std::ffi::c_void,
                ) -> com::sys::HRESULT {
                    if aggr != std::ptr::null_mut() {
                        return com::sys::CLASS_E_NOAGGREGATION;
                    }

                    let mut instance = <#co_class_name as ::std::default::Default>::default();
                    instance.add_ref();
                    let hr = instance.query_interface(riid, ppv);
                    instance.release();

                    core::mem::forget(instance);
                    hr
                }

                fn LockServer(&self, _increment: com::sys::BOOL) -> com::sys::HRESULT {
                    com::sys::S_OK
                }
            }
        }
    }
}
