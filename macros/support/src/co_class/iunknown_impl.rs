use super::CoClass;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates the IUnknown implementation for the COM Object.
/// Takes into account the base interfaces exposed, as well as
/// any interfaces exposed through an aggregated object.
pub fn generate(co_class: &CoClass) -> TokenStream {
    let struct_ident = &co_class.name;

    let add_ref = gen_add_ref();
    let release = gen_release(&co_class.interfaces, struct_ident);
    let query_interface = gen_query_interface(&co_class.interfaces);

    quote!(
        impl #struct_ident {
            #add_ref
            #release
            #query_interface
        }
    )
}

pub fn gen_add_ref() -> TokenStream {
    let add_ref_implementation = gen_add_ref_implementation();

    quote! {
        unsafe extern "stdcall" fn add_ref(this_ptr: *mut *mut Self) -> u32 {
            assert!(!this_ptr.is_null());
            let this = &(*(*this_ptr));
            #add_ref_implementation
        }
    }
}

pub fn gen_add_ref_implementation() -> TokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote! {
        let value = this.#ref_count_ident.get().checked_add(1).expect("Overflow of reference count");
        this.#ref_count_ident.set(value);
        value
    }
}

pub fn gen_release(interface_idents: &[syn::Path], name: &Ident) -> TokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();

    let release_decrement = gen_release_decrement(&ref_count_ident);
    let release_assign_new_count_to_var =
        gen_release_assign_new_count_to_var(&ref_count_ident, &ref_count_ident);
    let release_new_count_var_zero_check = gen_new_count_var_zero_check(&ref_count_ident);
    let release_drops = gen_release_drops(interface_idents, name);

    quote! {
        unsafe extern "stdcall" fn release(this_ptr: *mut *mut Self) -> u32 {
            assert!(!this_ptr.is_null());
            let this = &(*(*this_ptr));
            #release_decrement
            #release_assign_new_count_to_var
            if #release_new_count_var_zero_check {
                #release_drops
            }

            #ref_count_ident
        }
    }
}

pub fn gen_release_decrement(ref_count_ident: &Ident) -> TokenStream {
    quote! {
        let value = this.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count");
        this.#ref_count_ident.set(value);
    }
}

pub fn gen_release_assign_new_count_to_var(
    ref_count_ident: &Ident,
    new_count_ident: &Ident,
) -> TokenStream {
    quote!(
        let #new_count_ident = this.#ref_count_ident.get();
    )
}

pub fn gen_new_count_var_zero_check(new_count_ident: &Ident) -> TokenStream {
    quote!(
        #new_count_ident == 0
    )
}

pub fn gen_release_drops(interface_idents: &[syn::Path], name: &Ident) -> TokenStream {
    let vptr_drops = gen_vptr_drops(interface_idents);
    let com_object_drop = gen_com_object_drop(name);

    quote!(
        #vptr_drops
        #com_object_drop
    )
}

fn gen_vptr_drops(interface_idents: &[syn::Path]) -> TokenStream {
    let vptr_drops = interface_idents
        .iter()
        .enumerate()
        .map(|(index, interface)| {
            let vptr_field_ident = quote::format_ident!("__{}", index);
            quote!(
                Box::from_raw(this.#vptr_field_ident.as_ptr());
            )
        });

    quote!(#(#vptr_drops)*)
}

fn gen_com_object_drop(name: &Ident) -> TokenStream {
    quote!(
        Box::from_raw(this_ptr as *const _ as *mut #name);
    )
}

pub fn gen_query_interface(interface_idents: &[syn::Path]) -> TokenStream {
    // Generate match arms for implemented interfaces
    let base_match_arms = gen_base_match_arms(interface_idents);

    quote! {
        unsafe extern "stdcall" fn query_interface(
            this_ptr: *mut *mut Self,
            riid: *const com::sys::IID,
            ppv: *mut *mut std::ffi::c_void
        ) -> com::sys::HRESULT {
            assert!(!this_ptr.is_null());
            let this = &(*(*this_ptr));
            let riid = &*riid;

            if riid == &com::interfaces::iunknown::IID_IUNKNOWN {
                *ppv = &this.__0 as *const _ as *mut std::ffi::c_void;
            } #base_match_arms else {
                *ppv = std::ptr::null_mut::<std::ffi::c_void>();
                return com::sys::E_NOINTERFACE;
            }

            Self::add_ref(this_ptr);
            com::sys::NOERROR
        }
    }
}

pub fn gen_base_match_arms(interface_idents: &[syn::Path]) -> TokenStream {
    // Generate match arms for implemented interfaces
    let base_match_arms = interface_idents
        .iter()
        .enumerate()
        .map(|(index, interface)| {
            let match_condition =
                quote!(<#interface as com::ComInterface>::is_iid_in_inheritance_chain(riid));
            let vptr_field_ident = quote::format_ident!("__{}", index);

            quote!(
                else if #match_condition {
                    *ppv = &this.#vptr_field_ident as *const _ as *mut std::ffi::c_void;
                }
            )
        });

    quote!(#(#base_match_arms)*)
}
