use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemStruct};

/// Generates the IUnknown implementation for the COM Object.
/// Takes into account the base interfaces exposed, as well as
/// any interfaces exposed through an aggregated object.
pub fn generate(
    base_interface_idents: &[Ident],
    aggr_interface_idents: &HashMap<Ident, Vec<Ident>>,
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let ref_count_ident = macro_utils::ref_count_ident();
    
    let query_interface = gen_query_interface(base_interface_idents, aggr_interface_idents, struct_item);
    
    quote!(
        impl com::IUnknown for #struct_ident {
            
            #query_interface

            fn add_ref(&mut self) -> u32 {
                self.#ref_count_ident += 1;
                println!("Count now {}", self.#ref_count_ident);
                self.#ref_count_ident
            }

            fn release(&mut self) -> u32 {
                self.#ref_count_ident -= 1;
                println!("Count now {}", self.#ref_count_ident);
                let count = self.#ref_count_ident;
                if count == 0 {
                    println!("Count is 0 for {}. Freeing memory...", stringify!(#struct_ident));
                    // drop(self)
                    // TODO: This doesn't free the original heap allocation, as &mut self results
                    // in a copy from the raw pointer.
                    unsafe { Box::from_raw(self as *const _ as *mut #struct_ident); }
                }
                count
            }
        }
    )
}

fn gen_base_match_arms(
    base_interface_idents: &[Ident],
) -> HelperTokenStream {
    // Generate match arms for implemented interfaces
    let base_match_arms = base_interface_idents.iter().map(|base| {
        let match_condition =
            quote!(<dyn #base as com::ComInterface>::is_iid_in_inheritance_chain(riid));
        let vptr_field_ident = macro_utils::vptr_field_ident(&base);

        quote!(
            else if #match_condition {
                *ppv = &self.#vptr_field_ident as *const _ as *mut winapi::ctypes::c_void;
            }
        )
    });

    quote!(#(#base_match_arms)*)
}

fn gen_aggregate_match_arms(
    aggr_interface_idents: &HashMap<Ident, Vec<Ident>>,
) -> HelperTokenStream {
    let aggr_match_arms = aggr_interface_idents.iter().map(|(aggr_field_ident, aggr_base_interface_idents)| {

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
                    *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                    return winapi::shared::winerror::E_NOINTERFACE;
                }

                let mut aggr_interface_ptr: com::ComPtr<dyn com::IUnknown> = com::ComPtr::new(self.#aggr_field_ident as *mut winapi::ctypes::c_void);
                let hr = aggr_interface_ptr.query_interface(riid, ppv);
                if com::failed(hr) {
                    *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
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

    quote!(#(#aggr_match_arms)*)
}

fn gen_query_interface(
    base_interface_idents: &[Ident],
    aggr_interface_idents: &HashMap<Ident, Vec<Ident>>,
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    let first_vptr_field = macro_utils::vptr_field_ident(&base_interface_idents[0]);

    // Generate match arms for implemented interfaces
    let base_match_arms = gen_base_match_arms(base_interface_idents);

    // Generate match arms for aggregated interfaces
    let aggr_match_arms = gen_aggregate_match_arms(aggr_interface_idents);

    quote!(
        fn query_interface(
            &mut self,
            riid: *const winapi::shared::guiddef::IID,
            ppv: *mut *mut winapi::ctypes::c_void
        ) -> winapi::shared::winerror::HRESULT {
            unsafe {
                let riid = &*riid;

                if winapi::shared::guiddef::IsEqualGUID(riid, &com::IID_IUNKNOWN) {
                    *ppv = &self.#first_vptr_field as *const _ as *mut winapi::ctypes::c_void;
                } #base_match_arms #aggr_match_arms else {
                    *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                    println!("Returning NO INTERFACE.");
                    return winapi::shared::winerror::E_NOINTERFACE;
                }

                println!("Successful!.");
                self.add_ref();
                NOERROR
            }
        }
    )
}