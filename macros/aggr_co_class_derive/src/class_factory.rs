use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{ItemStruct,};

// #[repr(C)]
// pub struct LocalFileManagerClassFactory {
//     inner: <dyn com::IClassFactory as com::ComInterface>::VPtr,
//     ref_count: u32,
// }
// impl com::IClassFactory for LocalFileManagerClassFactory {
//     fn create_instance(
//         &mut self,
//         aggr: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
//         riid: winapi::shared::guiddef::REFIID,
//         ppv: *mut *mut winapi::ctypes::c_void,
//     ) -> winapi::shared::winerror::HRESULT {
//         use com::IUnknown;
//         let riid = unsafe { &*riid };
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Creating instance for ", "\n"],
//                 &match (&"LocalFileManager",) {
//                     (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
//                 },
//             ));
//         };
//         if !aggr.is_null()
//             && !winapi::shared::guiddef::IsEqualGUID(
//                 riid,
//                 &<dyn com::IUnknown as com::ComInterface>::IID,
//             )
//         {
//             unsafe {
//                 *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
//             }
//             return winapi::shared::winerror::E_INVALIDARG;
//         }
//         let mut instance = LocalFileManager::new();
//         instance.set_iunknown(aggr);
//         instance.inner_add_ref();
//         let hr = instance.inner_query_interface(riid, ppv);
//         instance.inner_release();
//         Box::into_raw(instance);
//         hr
//     }
//     fn lock_server(
//         &mut self,
//         _increment: winapi::shared::minwindef::BOOL,
//     ) -> winapi::shared::winerror::HRESULT {
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["LockServer called\n"],
//                 &match () {
//                     () => [],
//                 },
//             ));
//         };
//         winapi::shared::winerror::S_OK
//     }
// }
// impl com::IUnknown for LocalFileManagerClassFactory {
//     fn query_interface(
//         &mut self,
//         riid: *const winapi::shared::guiddef::IID,
//         ppv: *mut *mut winapi::ctypes::c_void,
//     ) -> winapi::shared::winerror::HRESULT {
//         use com::IUnknown;
//         unsafe {
//             {
//                 ::std::io::_print(::std::fmt::Arguments::new_v1(
//                     &["Querying interface on ", "...\n"],
//                     &match (&"LocalFileManagerClassFactory",) {
//                         (arg0,) => {
//                             [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)]
//                         }
//                     },
//                 ));
//             };
//             let riid = &*riid;
//             if winapi::shared::guiddef::IsEqualGUID(
//                 riid,
//                 &<dyn com::IUnknown as com::ComInterface>::IID,
//             ) | winapi::shared::guiddef::IsEqualGUID(
//                 riid,
//                 &<dyn com::IClassFactory as com::ComInterface>::IID,
//             ) {
//                 *ppv = &self.inner as *const _ as *mut winapi::ctypes::c_void;
//                 self.add_ref();
//                 winapi::shared::winerror::NOERROR
//             } else {
//                 *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
//                 winapi::shared::winerror::E_NOINTERFACE
//             }
//         }
//     }
//     fn add_ref(&mut self) -> u32 {
//         self.ref_count += 1;
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Count now ", "\n"],
//                 &match (&self.ref_count,) {
//                     (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
//                 },
//             ));
//         };
//         self.ref_count
//     }
//     fn release(&mut self) -> u32 {
//         self.ref_count -= 1;
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Count now ", "\n"],
//                 &match (&self.ref_count,) {
//                     (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
//                 },
//             ));
//         };
//         let count = self.ref_count;
//         if count == 0 {
//             {
//                 ::std::io::_print(::std::fmt::Arguments::new_v1(
//                     &["Count is 0 for ", ". Freeing memory...\n"],
//                     &match (&"LocalFileManagerClassFactory",) {
//                         (arg0,) => {
//                             [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)]
//                         }
//                     },
//                 ));
//             };
//             unsafe {
//                 Box::from_raw(self as *const _ as *mut LocalFileManagerClassFactory);
//             }
//         }
//         count
//     }
// }
// impl LocalFileManagerClassFactory {
//     pub(crate) fn new() -> Box<LocalFileManagerClassFactory> {
//         use com::IClassFactory;
//         {
//             ::std::io::_print(::std::fmt::Arguments::new_v1(
//                 &["Allocating new Vtable for ", "...\n"],
//                 &match (&"LocalFileManagerClassFactory",) {
//                     (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
//                 },
//             ));
//         };
//         let class_vtable = <dyn IClassFactory as ::com::ProductionComInterface<
//             LocalFileManagerClassFactory,
//         >>::vtable::<::com::offset::Zero>();
//         let vptr = Box::into_raw(Box::new(class_vtable));
//         let class_factory = LocalFileManagerClassFactory {
//             inner: vptr,
//             ref_count: 0,
//         };
//         Box::new(class_factory)
//     }
// }

// We manually generate a ClassFactory without macros, otherwise
// it leads to an infinite loop.
pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let class_factory_ident = macro_utils::get_class_factory_ident(&real_ident);

    quote!(
        // We are not going to bother with using an Init_ struct here,
        // as we are not relying on the production macros.
        #[repr(C)]
        pub struct #class_factory_ident {
            inner: <dyn com::IClassFactory as com::ComInterface>::VPtr,
            ref_count: u32,
        }

        impl com::IClassFactory for #class_factory_ident {
            fn create_instance(
                &mut self,
                aggr: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
                riid: winapi::shared::guiddef::REFIID,
                ppv: *mut *mut winapi::ctypes::c_void,
            ) -> winapi::shared::winerror::HRESULT {
                // Bringing trait into scope to access IUnknown methods.
                use com::IUnknown;

                let riid = unsafe { &*riid };

                println!("Creating instance for {}", stringify!(#real_ident));
                if !aggr.is_null() && !winapi::shared::guiddef::IsEqualGUID(riid, &<dyn com::IUnknown as com::ComInterface>::IID) {
                    unsafe {
                        *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                    }
                    return winapi::shared::winerror::E_INVALIDARG;
                }

                let mut instance = #real_ident::new();

                // This check has to be here because it can only be done after object
                // is allocated on the heap (address of nonDelegatingUnknown fixed)
                instance.set_iunknown(aggr);

                // As an aggregable object, we have to add_ref through the
                // non-delegating IUnknown on creation. Otherwise, we might
                // add_ref the outer object if aggregated.
                instance.inner_add_ref();
                let hr = instance.inner_query_interface(riid, ppv);
                instance.inner_release();

                Box::into_raw(instance);
                hr
            }

            // TODO: Implement correctly
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
                    if winapi::shared::guiddef::IsEqualGUID(riid, &<dyn com::IUnknown as com::ComInterface>::IID) | winapi::shared::guiddef::IsEqualGUID(riid, &<dyn com::IClassFactory as com::ComInterface>::IID) {
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

        // Code here usually belongs to the allocate function, but for simplicity
        // we just wrote it directly.
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