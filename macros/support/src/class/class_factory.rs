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
        use ::com::interfaces::IClassFactory;
        ::com::class! {
            pub factory #class_factory_ident: IClassFactory {}

            impl IClassFactory for #class_factory_ident {
                unsafe fn CreateInstance(
                    &self,
                    aggr: *mut std::ptr::NonNull<<::com::interfaces::IUnknown as ::com::ComInterface>::VTable>,
                    riid: *const com::sys::IID,
                    ppv: *mut *mut std::ffi::c_void,
                ) -> com::sys::HRESULT {
                    if aggr != std::ptr::null_mut() {
                        return com::sys::CLASS_E_NOAGGREGATION;
                    }

                    let mut instance = ::std::boxed::Box::new(<#class_name as ::std::default::Default>::default());
                    instance.add_ref();
                    let hr = instance.query_interface(riid, ppv);
                    instance.release();

                    core::mem::forget(instance);
                    hr
                }

                unsafe fn LockServer(&self, _increment: com::sys::BOOL) -> com::sys::HRESULT {
                    com::sys::S_OK
                }
            }
        }
    }
}
