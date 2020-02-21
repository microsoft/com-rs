use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemStruct};

/// Generates the IUnknown implementation for the COM Object.
/// Takes into account the base interfaces exposed, as well as
/// any interfaces exposed through an aggregated object.
pub fn generate(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    let query_interface = gen_query_interface(base_interface_idents, aggr_map);
    let add_ref = gen_add_ref();
    let release = gen_release(base_interface_idents, aggr_map, struct_ident);

    quote!(
        impl com::interfaces::IUnknown for #struct_ident {
            #query_interface
            #add_ref
            #release
        }
    )
}

pub fn gen_add_ref() -> HelperTokenStream {
    let add_ref_implementation = gen_add_ref_implementation();

    quote! {
        unsafe fn add_ref(&self) -> u32 {
            #add_ref_implementation
        }
    }
}

pub fn gen_add_ref_implementation() -> HelperTokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote!(
        let value = self.#ref_count_ident.get().checked_add(1).expect("Overflow of reference count");
        self.#ref_count_ident.set(value);
        value
    )
}

pub fn gen_release(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    struct_ident: &Ident,
) -> HelperTokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();

    let release_decrement = gen_release_decrement(&ref_count_ident);
    let release_assign_new_count_to_var =
        gen_release_assign_new_count_to_var(&ref_count_ident, &ref_count_ident);
    let release_new_count_var_zero_check = gen_new_count_var_zero_check(&ref_count_ident);
    let release_drops = gen_release_drops(base_interface_idents, aggr_map, struct_ident);

    quote! {
        unsafe fn release(&self) -> u32 {
            #release_decrement
            #release_assign_new_count_to_var
            if #release_new_count_var_zero_check {
                #release_drops
            }

            #ref_count_ident
        }
    }
}

pub fn gen_release_drops(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    struct_ident: &Ident,
) -> HelperTokenStream {
    let vptr_drops = gen_vptr_drops(base_interface_idents);
    let aggregate_drops = gen_aggregate_drops(aggr_map);
    let com_object_drop = gen_com_object_drop(struct_ident);

    quote!(
        #vptr_drops
        #aggregate_drops
        #com_object_drop
    )
}

fn gen_aggregate_drops(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
    let aggregate_drops = aggr_map.iter().map(|(aggr_field_ident, _)| {
        quote!(
            if !self.#aggr_field_ident.is_null() {
                let mut aggr_interface_ptr = com::ComPtr::<dyn com::interfaces::iunknown::IUnknown>::new(self.#aggr_field_ident as *mut _);
                aggr_interface_ptr.release();
            }
        )
    });

    quote!(#(#aggregate_drops)*)
}

fn gen_vptr_drops(base_interface_idents: &[Ident]) -> HelperTokenStream {
    let vptr_drops = base_interface_idents.iter().map(|base| {
        let vptr_field_ident = crate::utils::vptr_field_ident(&base);
        quote!(
            Box::from_raw(self.#vptr_field_ident as *mut <dyn #base as com::ComInterface>::VTable);
        )
    });

    quote!(#(#vptr_drops)*)
}

fn gen_com_object_drop(struct_ident: &Ident) -> HelperTokenStream {
    quote!(
        Box::from_raw(self as *const _ as *mut #struct_ident);
    )
}

pub fn gen_release_decrement(ref_count_ident: &Ident) -> HelperTokenStream {
    quote!(
        let value = self.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count");
        self.#ref_count_ident.set(value);
    )
}

pub fn gen_release_assign_new_count_to_var(
    ref_count_ident: &Ident,
    new_count_ident: &Ident,
) -> HelperTokenStream {
    quote!(
        let #new_count_ident = self.#ref_count_ident.get();
    )
}

pub fn gen_new_count_var_zero_check(new_count_ident: &Ident) -> HelperTokenStream {
    quote!(
        #new_count_ident == 0
    )
}

pub fn gen_query_interface(
    base_interface_idents: &[Ident],
    aggr_map: &HashMap<Ident, Vec<Ident>>,
) -> HelperTokenStream {
    let first_vptr_field = crate::utils::vptr_field_ident(&base_interface_idents[0]);

    // Generate match arms for implemented interfaces
    let base_match_arms = gen_base_match_arms(base_interface_idents);

    // Generate match arms for aggregated interfaces
    let aggr_match_arms = gen_aggregate_match_arms(aggr_map);

    quote!(
        unsafe fn query_interface(
            &self,
            riid: *const com::sys::IID,
            ppv: *mut *mut std::ffi::c_void
        ) -> com::sys::HRESULT {
            let riid = &*riid;

            if riid == &com::interfaces::iunknown::IID_IUNKNOWN {
                *ppv = &self.#first_vptr_field as *const _ as *mut std::ffi::c_void;
            } #base_match_arms #aggr_match_arms else {
                *ppv = std::ptr::null_mut::<std::ffi::c_void>();
                return com::sys::E_NOINTERFACE;
            }

            self.add_ref();
            com::sys::NOERROR
        }
    )
}

pub fn gen_base_match_arms(base_interface_idents: &[Ident]) -> HelperTokenStream {
    // Generate match arms for implemented interfaces
    let base_match_arms = base_interface_idents.iter().map(|base| {
        let match_condition =
            quote!(<dyn #base as com::ComInterface>::is_iid_in_inheritance_chain(riid));
        let vptr_field_ident = crate::utils::vptr_field_ident(&base);

        quote!(
            else if #match_condition {
                *ppv = &self.#vptr_field_ident as *const _ as *mut std::ffi::c_void;
            }
        )
    });

    quote!(#(#base_match_arms)*)
}

pub fn gen_aggregate_match_arms(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
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
                if self.#aggr_field_ident.is_null() {
                    *ppv = std::ptr::null_mut::<std::ffi::c_void>();
                    return com::sys::E_NOINTERFACE;
                }

                let mut aggr_interface_ptr = com::ComPtr::<dyn com::interfaces::iunknown::IUnknown>::new(self.#aggr_field_ident as *mut _);
                let hr = aggr_interface_ptr.query_interface(riid, ppv);
                if com::sys::FAILED(hr) {
                    *ppv = std::ptr::null_mut::<std::ffi::c_void>();
                    return com::sys::E_NOINTERFACE;
                }

                // We release it as the previous call add_ref-ed the inner object.
                // The intention is to transfer reference counting logic to the
                // outer object.
                aggr_interface_ptr.release();
            }
        )
    });

    quote!(#(#aggr_match_arms)*)
}
