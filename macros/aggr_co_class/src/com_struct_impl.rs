use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Fields, Ident, ItemStruct};

/// Generates the methods that the com struct needs to have. These include:
/// allocate: To initialise the vtables, including the non_delegatingegating_iunknown one.
/// set_iunknown: For Class Objects to set the iunknown to use, for aggregation.
/// inner_iunknown: declare the non_delegatingegating iunknown functions on the com struct.
/// get_class_object: Entry point to obtain the IClassFactory Class Object suited for this class.
/// set_aggregate_*: Functions to initialise aggregation for the group the interface belongs to.
pub fn generate(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let allocate_fn = gen_allocate_fn(aggr_map, base_interface_idents, struct_item);
    let set_iunknown_fn = gen_set_iunknown_fn();
    let inner_iunknown_fns = gen_inner_iunknown_fns(base_interface_idents, aggr_map, struct_item);
    let get_class_object_fn = co_class::com_struct_impl::gen_get_class_object_fn(struct_item);
    let set_aggregate_fns = co_class::com_struct_impl::gen_set_aggregate_fns(aggr_map);

    quote!(
        impl #struct_ident {
            #allocate_fn
            #set_iunknown_fn
            #inner_iunknown_fns
            #get_class_object_fn
            #set_aggregate_fns
        }
    )
}

/// Function that should only be used by Class Object, to set the
/// object's iunknown_to_use, if the object is going to get aggregated.
fn gen_set_iunknown_fn() -> HelperTokenStream {
    let iunknown_to_use_field_ident = macro_utils::iunknown_to_use_field_ident();
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();

    quote!(
        pub(crate) fn set_iunknown(&mut self, aggr: *mut <dyn com::IUnknown as com::ComInterface>::VPtr) {
            if aggr.is_null() {
                self.#iunknown_to_use_field_ident = &self.#non_delegating_iunknown_field_ident as *const _ as *mut <dyn com::IUnknown as com::ComInterface>::VPtr;
            } else {
                self.#iunknown_to_use_field_ident = aggr;
            }
        }
    )
}

/// The non-delegating IUnknown implementation for an aggregable object. This will contain
/// the actual IUnknown implementations for the object.
fn gen_inner_iunknown_fns(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let inner_query_interface = gen_inner_query_interface(base_interface_idents, aggr_map);
    let inner_add_ref = gen_inner_add_ref();
    let inner_release = gen_inner_release(struct_ident);

    quote!(
        #inner_query_interface
        #inner_add_ref
        #inner_release
    )
}

pub fn gen_inner_add_ref() -> HelperTokenStream {
    let ref_count_ident = macro_utils::ref_count_ident();
    quote! {
        pub(crate) fn inner_add_ref(&mut self) -> u32 {
            self.#ref_count_ident = self.#ref_count_ident.checked_add(1).expect("Overflow of reference count");
            println!("Count now {}", self.#ref_count_ident);
            self.#ref_count_ident
        }
    }
}

pub fn gen_inner_release(struct_ident: &Ident) -> HelperTokenStream {
    let ref_count_ident = macro_utils::ref_count_ident();
    quote! {
        pub(crate) unsafe fn inner_release(&mut self) -> u32 {
            self.#ref_count_ident = self.#ref_count_ident.checked_sub(1).expect("Underflow of reference count");
            println!("Count now {}", self.#ref_count_ident);
            let count = self.#ref_count_ident;
            if count == 0 {
                println!("Count is 0 for {}. Freeing memory...", stringify!(#struct_ident));
                Box::from_raw(self as *const _ as *mut #struct_ident);
            }
            count
        }
    }
}

/// Non-delegating query interface
fn gen_inner_query_interface(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
) -> HelperTokenStream {
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();

    // Generate match arms for implemented interfaces
    let base_match_arms = co_class::iunknown_impl::gen_base_match_arms(base_interface_idents);

    // Generate match arms for aggregated interfaces
    let aggr_match_arms = co_class::iunknown_impl::gen_aggregate_match_arms(aggr_map);

    quote!(
        pub(crate) fn inner_query_interface(&mut self, riid: *const winapi::shared::guiddef::IID, ppv: *mut *mut winapi::ctypes::c_void) -> HRESULT {
            println!("Non delegating QI");

            unsafe {
                let riid = &*riid;

                if winapi::shared::guiddef::IsEqualGUID(riid, &com::IID_IUNKNOWN) {
                    *ppv = &self.#non_delegating_iunknown_field_ident as *const _ as *mut winapi::ctypes::c_void;
                } #base_match_arms #aggr_match_arms else {
                    *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                    println!("Returning NO INTERFACE.");
                    return winapi::shared::winerror::E_NOINTERFACE;
                }

                println!("Successful!.");
                self.inner_add_ref();
                NOERROR
            }
        }
    )
}

/// For an aggregable object, we have to do more work here. We need to
/// instantiate the non-delegating IUnknown vtable. The unsafe extern "stdcall"
/// methods belonging to the non-delegating IUnknown vtable are also defined here.
fn gen_allocate_fn(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    let base_inits = co_class::com_struct_impl::gen_allocate_base_inits(struct_ident, base_interface_idents);

    // Allocate function signature
    let allocate_parameters = co_class::com_struct_impl::gen_allocate_function_parameters_signature(struct_item);

    // Syntax for instantiating the fields of the struct.
    let base_fields = co_class::com_struct_impl::gen_allocate_base_fields(base_interface_idents);
    let ref_count_field = co_class::com_struct_impl::gen_allocate_ref_count_field();
    let user_fields = co_class::com_struct_impl::gen_allocate_user_fields(struct_item);
    let aggregate_fields = co_class::com_struct_impl::gen_allocate_aggregate_fields(aggr_map);

    // Aggregable COM struct specific fields  
    let iunknown_to_use_field_ident = macro_utils::iunknown_to_use_field_ident();
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();
    let non_delegating_iunknown_offset = base_interface_idents.len();

    quote!(
        fn allocate(#allocate_parameters) -> Box<#struct_ident> {
            println!("Allocating new VTable for {}", stringify!(#struct_ident));

            // Non-delegating methods.
            unsafe extern "stdcall" fn non_delegatingegating_query_interface(
                this: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void,
            ) -> HRESULT {
                let this = this.sub(#non_delegating_iunknown_offset) as *mut #struct_ident;
                (*this).inner_query_interface(riid, ppv)
            }

            unsafe extern "stdcall" fn non_delegatingegating_add_ref(
                this: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
            ) -> u32 {
                let this = this.sub(#non_delegating_iunknown_offset) as *mut #struct_ident;
                (*this).inner_add_ref()
            }

            unsafe extern "stdcall" fn non_delegatingegating_release(
                this: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
            ) -> u32 {
                let this = this.sub(#non_delegating_iunknown_offset) as *mut #struct_ident;
                (*this).inner_release()
            }

            // Rust Parser limitation? Unable to construct associated type directly.
            type __iunknown_vtable_type = <dyn com::IUnknown as com::ComInterface>::VTable;
            let __non_delegating_iunknown_vtable =  __iunknown_vtable_type {
                QueryInterface: non_delegatingegating_query_interface,
                Release: non_delegatingegating_release,
                AddRef: non_delegatingegating_add_ref,
            };
            let #non_delegating_iunknown_field_ident = Box::into_raw(Box::new(__non_delegating_iunknown_vtable));

            #base_inits

            let out = #struct_ident {
                #base_fields
                #non_delegating_iunknown_field_ident,
                #iunknown_to_use_field_ident: std::ptr::null_mut::<<dyn com::IUnknown as com::ComInterface>::VPtr>(),
                #ref_count_field
                #aggregate_fields
                #user_fields
            };
            Box::new(out)
        }
    )
}