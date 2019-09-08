use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{ItemStruct,};

// We manually generate a ClassFactory without macros, otherwise
// it leads to an infinite loop.
pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let class_factory_ident = macro_utils::get_class_factory_ident(&real_ident);

    quote!(
        #[repr(C)]
        pub struct #class_factory_ident {
            inner: <com::IClassFactory as com::ComInterface>::VPtr,
            ref_count: u32,
        }

        impl com::IClassFactory for #class_factory_ident {
            fn create_instance(
                &mut self,
                aggr: *mut <com::IUnknown as com::ComInterface>::VPtr,
                riid: winapi::shared::guiddef::REFIID,
                ppv: *mut *mut winapi::ctypes::c_void,
            ) -> winapi::shared::winerror::HRESULT {
                use com::IUnknown;

                println!("Creating instance for {}", stringify!(#real_ident));
                if aggr != std::ptr::null_mut() {
                    return winapi::shared::winerror::CLASS_E_NOAGGREGATION;
                }

                let mut instance = #real_ident::new();
                instance.add_ref();
                let hr = instance.query_interface(riid, ppv);
                instance.release();

                Box::into_raw(instance);
                hr
            }

            fn lock_server(&mut self, _increment: winapi::shared::minwindef::BOOL) -> winapi::shared::winerror::HRESULT {
                println!("LockServer called");
                winapi::shared::winerror::S_OK
            }
        }

        impl com::IUnknown for #class_factory_ident {
            fn query_interface(&mut self, riid: *const winapi::shared::guiddef::IID, ppv: *mut *mut winapi::ctypes::c_void) -> winapi::shared::winerror::HRESULT {
                // Bringing trait into scope to access add_ref method.
                use com::IUnknown;

                unsafe {
                    println!("Querying interface on {}...", stringify!(#class_factory_ident));

                    let riid = &*riid;
                    if winapi::shared::guiddef::IsEqualGUID(riid, &<com::IUnknown as com::ComInterface>::IID) | winapi::shared::guiddef::IsEqualGUID(riid, &<com::IClassFactory as com::ComInterface>::IID) {
                        *ppv = &self.inner as *const _ as *mut winapi::ctypes::c_void;
                        self.add_ref();
                        winapi::shared::winerror::NOERROR
                    } else {
                        *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                        winapi::shared::winerror::E_NOINTERFACE
                    }
                }
            }

            fn add_ref(&mut self) -> u32 {
                self.ref_count += 1;
                println!("Count now {}", self.ref_count);
                self.ref_count
            }

            fn release(&mut self) -> u32 {
                self.ref_count -= 1;
                println!("Count now {}", self.ref_count);
                let count = self.ref_count;
                if count == 0 {
                    println!("Count is 0 for {}. Freeing memory...", stringify!(#class_factory_ident));
                    unsafe { Box::from_raw(self as *const _ as *mut #class_factory_ident); }
                }
                count
            }
        }

        impl #class_factory_ident {
            pub(crate) fn new() -> Box<#class_factory_ident> {
                use com::IClassFactory;

                println!("Allocating new Vtable for {}...", stringify!(#class_factory_ident));
                let class_vtable = com::vtable!(#class_factory_ident: IClassFactory);
                let vptr = Box::into_raw(Box::new(class_vtable));
                let class_factory = #class_factory_ident {
                    inner: vptr,
                    ref_count: 0,
                };
                Box::new(class_factory)
            }
        }
    )
}