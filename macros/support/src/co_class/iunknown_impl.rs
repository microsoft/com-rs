use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn gen_add_ref(class_name: &Ident) -> TokenStream {
    let add_ref_implementation = gen_add_ref_implementation();
    let this_ptr = this_ptr();

    quote! {
        extern "stdcall" fn add_ref(this_ptr: #this_ptr) -> u32 {
            let this = this_ptr.as_ref().as_ref();
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

pub fn gen_release(interface_idents: &[&syn::Path], class_name: &Ident) -> TokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();

    let release_decrement = gen_release_decrement(&ref_count_ident);
    let release_assign_new_count_to_var =
        gen_release_assign_new_count_to_var(&ref_count_ident, &ref_count_ident);
    let release_new_count_var_zero_check = gen_new_count_var_zero_check(&ref_count_ident);
    let release_drops = gen_release_drops(interface_idents, class_name);
    let this_ptr = this_ptr();

    quote! {
        unsafe extern "stdcall" fn release(this_ptr: #this_ptr) -> u32 {
            let this = this_ptr.as_ref().as_ref();
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

pub fn gen_release_drops(interface_idents: &[&syn::Path], name: &Ident) -> TokenStream {
    let vptr_drops = gen_vptr_drops(interface_idents);
    let com_object_drop = gen_com_object_drop(name);

    quote!(
        #vptr_drops
        #com_object_drop
    )
}

fn gen_vptr_drops(interface_idents: &[&syn::Path]) -> TokenStream {
    let vptr_drops = interface_idents.iter().enumerate().map(|(index, _)| {
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

pub fn gen_query_interface(name: &Ident, interface_idents: &[&syn::Path]) -> TokenStream {
    // Generate match arms for implemented interfaces
    let base_match_arms = gen_base_match_arms(interface_idents);
    let this_ptr = this_ptr();

    quote! {
        unsafe extern "stdcall" fn query_interface(
            this_ptr: #this_ptr,
            riid: *const com::sys::IID,
            ppv: *mut *mut std::ffi::c_void
        ) -> com::sys::HRESULT {
            let this = this_ptr.as_ref().as_ref();
            let riid = &*riid;

            if riid == &com::interfaces::iunknown::IID_IUNKNOWN {
                *ppv = &this.__0 as *const _ as *mut std::ffi::c_void;
            } #base_match_arms else {
                *ppv = std::ptr::null_mut::<std::ffi::c_void>();
                return com::sys::E_NOINTERFACE;
            }

            #name::add_ref(this_ptr);
            com::sys::NOERROR
        }
    }
}

pub fn gen_base_match_arms(interface_idents: &[&syn::Path]) -> TokenStream {
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

fn this_ptr() -> TokenStream {
    quote! {
        ::std::ptr::NonNull<::std::ptr::NonNull<<::com::interfaces::IUnknown as ::com::ComInterface>::VTable>>
    }
}
