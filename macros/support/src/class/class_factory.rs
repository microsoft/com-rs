use super::Class;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(class: &Class) -> TokenStream {
    if !class.has_class_factory {
        return TokenStream::new();
    }

    let class_factory_ident = crate::utils::class_factory_ident(&class.name);
    let class_name = &class.name;
    let user_fields = class.fields.iter().map(|f| {
        let ty = &f.ty;
        quote! { <#ty as ::core::default::Default>::default() }
    });
    quote! {
        ::com::class! {
            #[no_class_factory]
            pub class #class_factory_ident: ::com::interfaces::IClassFactory {}

            impl ::com::interfaces::IClassFactory for #class_factory_ident {
                unsafe fn CreateInstance(
                    &self,
                    aggr: *mut ::core::ptr::NonNull<<::com::interfaces::IUnknown as ::com::Interface>::VTable>,
                    riid: *const ::com::sys::IID,
                    ppv: *mut *mut ::core::ffi::c_void,
                ) -> ::com::sys::HRESULT {
                    assert!(!riid.is_null(), "iid passed to CreateInstance was null");
                    if !aggr.is_null() {
                        return ::com::sys::CLASS_E_NOAGGREGATION;
                    }

                    let instance = #class_name::allocate(#(#user_fields),*);
                    instance.QueryInterface(riid, ppv)
                }

                unsafe fn LockServer(&self, _increment: com::sys::BOOL) -> com::sys::HRESULT {
                    ::com::sys::S_OK
                }
            }
        }
    }
}
