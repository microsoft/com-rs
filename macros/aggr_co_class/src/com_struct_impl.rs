use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemStruct, Fields,};

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
    let get_class_object_fn = gen_get_class_object_fn(struct_item);
    let set_aggregate_fns = gen_set_aggregate_fns(aggr_map);

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

/// Function used by in-process DLL macro to get an instance of the
/// class object.
fn gen_get_class_object_fn(struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let class_factory_ident = macro_utils::class_factory_ident(&struct_ident);

    quote!(
        pub fn get_class_object() -> Box<#class_factory_ident> {
            <#class_factory_ident>::new()
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
    let ref_count_ident = macro_utils::ref_count_ident();
    let inner_query_interface = gen_inner_query_interface(base_interface_idents, aggr_map);

    quote!(
        #inner_query_interface

        pub(crate) fn inner_add_ref(&mut self) -> u32 {
            self.#ref_count_ident += 1;
            println!("Count now {}", self.#ref_count_ident);
            self.#ref_count_ident
        }

        pub(crate) fn inner_release(&mut self) -> u32 {
            self.#ref_count_ident -= 1;
            println!("Count now {}", self.#ref_count_ident);
            let count = self.#ref_count_ident;
            if count == 0 {
                println!("Count is 0 for {}. Freeing memory...", stringify!(#struct_ident));
                // drop(self)
                unsafe { Box::from_raw(self as *const _ as *mut #struct_ident); }
            }
            count
        }
    )
}

/// Non-delegating query interface
fn gen_inner_query_interface(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
) -> HelperTokenStream {
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();

    // Generate match arms for implemented interfaces
    let match_arms = base_interface_idents.iter().map(|base| {
        let match_condition =
            quote!(<dyn #base as com::ComInterface>::is_iid_in_inheritance_chain(riid));
        let vptr_field_ident = macro_utils::vptr_field_ident(&base);

        quote!(
            else if #match_condition {
                *ppv = &self.#vptr_field_ident as *const _ as *mut winapi::ctypes::c_void;
            }
        )
    });

    // Generate match arms for aggregated interfaces
    let aggr_match_arms = aggr_map.iter().map(|(aggr_field_ident, aggr_base_interface_idents)| {

        // Construct the OR match conditions for a single aggregated object.
        let first_base_interface_ident = &aggr_base_interface_idents[0];
        let first_aggr_match_condition = quote!(
            <dyn #first_base_interface_ident as com::ComInterface>::is_iid_in_inheritance_chain(riid)
        );
        let rem_aggr_match_conditions = aggr_base_interface_idents.iter().skip(1).map(|base| {
            quote!(|| <dyn #base as com::ComInterface>::is_iid_in_inheritance_chain(riid))
        });

        quote!(
            else if #first_aggr_match_condition #(#rem_aggr_match_conditions)* {
                let mut aggr_interface_ptr: ComPtr<dyn com::IUnknown> = ComPtr::new(self.#aggr_field_ident as *mut winapi::ctypes::c_void);
                let hr = aggr_interface_ptr.query_interface(riid, ppv);
                if com::failed(hr) {
                    return winapi::shared::winerror::E_NOINTERFACE;
                }

                // We release it as the previous call add_ref-ed the inner object.
                // The intention is to transfer reference counting logic to the
                // outer object.
                aggr_interface_ptr.release();

                core::mem::forget(aggr_interface_ptr);
            }
        )
    });

    quote!(
        pub(crate) fn inner_query_interface(&mut self, riid: *const winapi::shared::guiddef::IID, ppv: *mut *mut winapi::ctypes::c_void) -> HRESULT {
            println!("Non delegating QI");

            unsafe {
                let riid = &*riid;

                if winapi::shared::guiddef::IsEqualGUID(riid, &com::IID_IUNKNOWN) {
                    *ppv = &self.#non_delegating_iunknown_field_ident as *const _ as *mut winapi::ctypes::c_void;
                } #(#match_arms)* #(#aggr_match_arms)* else {
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
fn gen_allocate_fn(aggr_map: &HashMap<Ident, Vec<Ident>>, base_interface_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    let mut offset_count: usize = 0;
    let base_inits = base_interface_idents.iter().map(|base| {
        let vtable_var_ident = quote::format_ident!("{}_vtable", base.to_string().to_lowercase());
        let vptr_field_ident = macro_utils::vptr_field_ident(&base);

        let out = quote!(
            let #vtable_var_ident = com::vtable!(#struct_ident: #base, #offset_count);
            let #vptr_field_ident = Box::into_raw(Box::new(#vtable_var_ident));
        );

        offset_count += 1;
        out
    });
    let base_fields = base_interface_idents.iter().map(|base| {
        let vptr_field_ident = macro_utils::vptr_field_ident(base);
        quote!(#vptr_field_ident)
    });
    let ref_count_ident = macro_utils::ref_count_ident();
    let iunknown_to_use_field_ident = macro_utils::iunknown_to_use_field_ident();
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();
    let non_delegating_iunknown_offset = base_interface_idents.len();

    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct.")
    };
    let field_idents = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap().clone();
        quote!(#field_ident)
    });

    let aggregate_inits = aggr_map.iter().map(|(aggr_field_ident, aggr_base_interface_idents)| {
        quote!(
            #aggr_field_ident: std::ptr::null_mut()
        )
    });

    quote!(
        fn allocate(#fields) -> Box<#struct_ident> {
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

            #(#base_inits)*
            let out = #struct_ident {
                #(#base_fields,)*
                #non_delegating_iunknown_field_ident,
                #iunknown_to_use_field_ident: std::ptr::null_mut::<<dyn com::IUnknown as com::ComInterface>::VPtr>(),
                #ref_count_ident: 0,
                #(#aggregate_inits,)*
                #(#field_idents)*
            };
            Box::new(out)
        }
    )
}

fn gen_set_aggregate_fns(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
    let mut fns = Vec::new();
    for (aggr_field_ident, aggr_base_interface_idents) in aggr_map.iter() {
        for base in aggr_base_interface_idents {
            let set_aggregate_fn_ident = macro_utils::set_aggregate_fn_ident(&base);
            fns.push(quote!(
                fn #set_aggregate_fn_ident(&mut self, aggr: *mut <dyn com::IUnknown as com::ComInterface>::VPtr) {
                    // TODO: What happens if we are overwriting an existing aggregate?
                    self.#aggr_field_ident = aggr
                }
            ));
        }
    }

    quote!(#(#fns)*)
}
