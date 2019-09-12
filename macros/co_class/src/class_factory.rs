use proc_macro2::{Ident, TokenStream as HelperTokenStream};
use quote::quote;
use syn::ItemStruct;

// We manually generate a ClassFactory without macros, otherwise
// it leads to an infinite loop.
pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let class_factory_ident = macro_utils::class_factory_ident(&struct_ident);

    let struct_definition = gen_class_factory_struct_definition(&class_factory_ident);
    let lock_server = gen_lock_server();
    let iunknown_impl = gen_iunknown_impl(&class_factory_ident);
    let class_factory_impl = gen_class_factory_impl(&class_factory_ident);

    quote! {
        #struct_definition

        impl com::IClassFactory for #class_factory_ident {
            unsafe fn create_instance(
                &mut self,
                aggr: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
                riid: winapi::shared::guiddef::REFIID,
                ppv: *mut *mut winapi::ctypes::c_void,
            ) -> winapi::shared::winerror::HRESULT {
                // Bringing trait into scope to access IUnknown methods.
                use com::IUnknown;

                println!("Creating instance for {}", stringify!(#struct_ident));
                if aggr != std::ptr::null_mut() {
                    return winapi::shared::winerror::CLASS_E_NOAGGREGATION;
                }

                let mut instance = #struct_ident::new();
                instance.add_ref();
                let hr = instance.query_interface(riid, ppv);
                instance.release();

                core::mem::forget(instance);
                hr
            }

            #lock_server
        }

        #iunknown_impl

        #class_factory_impl
    }
}

pub fn gen_class_factory_struct_definition(class_factory_ident: &Ident) -> HelperTokenStream {
    let ref_count_ident = macro_utils::ref_count_ident();
    quote! {
        #[repr(C)]
        pub struct #class_factory_ident {
            inner: <dyn com::IClassFactory as com::ComInterface>::VPtr,
            #ref_count_ident: u32,
        }
    }
}

pub fn gen_lock_server() -> HelperTokenStream {
    quote! {
        // TODO: Implement correctly
        fn lock_server(&mut self, _increment: winapi::shared::minwindef::BOOL) -> winapi::shared::winerror::HRESULT {
            println!("LockServer called");
            winapi::shared::winerror::S_OK
        }
    }
}

pub fn gen_iunknown_impl(class_factory_ident: &Ident) -> HelperTokenStream {
    let query_interface = gen_query_interface(class_factory_ident);
    let add_ref = crate::iunknown_impl::gen_add_ref();
    let release = crate::iunknown_impl::gen_release(class_factory_ident);
    quote! {
        impl com::IUnknown for #class_factory_ident {
            #query_interface
            #add_ref
            #release
        }
    }
}

fn gen_query_interface(class_factory_ident: &Ident) -> HelperTokenStream {
    quote! {
        unsafe fn query_interface(&mut self, riid: *const winapi::shared::guiddef::IID, ppv: *mut *mut winapi::ctypes::c_void) -> winapi::shared::winerror::HRESULT {
            // Bringing trait into scope to access add_ref method.
            use com::IUnknown;

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
}

pub fn gen_class_factory_impl(class_factory_ident: &Ident) -> HelperTokenStream {
    let ref_count_ident = macro_utils::ref_count_ident();
    quote! {
        impl #class_factory_ident {
            pub(crate) fn new() -> Box<#class_factory_ident> {
                use com::IClassFactory;

                println!("Allocating new Vtable for {}...", stringify!(#class_factory_ident));
                let class_vtable = com::vtable!(#class_factory_ident: IClassFactory);
                // allocate directly since no macros generated an `allocate` function
                let vptr = Box::into_raw(Box::new(class_vtable));
                let class_factory = #class_factory_ident {
                    inner: vptr,
                    #ref_count_ident: 0,
                };
                Box::new(class_factory)
            }
        }
    }
}
