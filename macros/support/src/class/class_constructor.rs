use super::{class::Interface, Class};
use proc_macro2::TokenStream;
use quote::quote;

/// Function used to instantiate the COM class
pub fn generate(class: &Class) -> TokenStream {
    let name = &class.name;
    let vis = &class.visibility;

    let parameters = &class.fields;
    let user_fields = class.fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: ::std::mem::ManuallyDrop::new(#name)
        }
    });

    let interface_inits = gen_vpointer_inits(class);
    let ref_count_ident = crate::utils::ref_count_ident();

    let interfaces = &class.interfaces;
    let interface_fields = gen_allocate_interface_fields(interfaces);

    quote! {
        /// Allocate the COM class
        #vis fn allocate<T: ::com::Interface>(#(#parameters),*) -> Option<T> {
            #interface_inits
            let instance = #name {
                #interface_fields
                #ref_count_ident: std::cell::Cell::new(1),
                #(#user_fields),*
            };
            let instance = ::std::mem::ManuallyDrop::new(::std::boxed::Box::pin(instance));
            instance.query()
        }
    }
}

/// Function used to instantiate a default version COM class
pub fn generate_default(class: &Class) -> TokenStream {
    let name = &class.name;
    let vis = &class.visibility;

    let user_fields = class.fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: ::std::mem::ManuallyDrop::new(::std::default::Default::default())
        }
    });

    let interface_inits = gen_vpointer_inits(class);
    let ref_count_ident = crate::utils::ref_count_ident();

    let interfaces = &class.interfaces;
    let interface_fields = gen_allocate_interface_fields(interfaces);

    quote! {
        /// Allocate a default version of the COM class casting it to the supplied interface.
        #vis fn allocate_default<T: ::com::Interface>() -> Option<T> {
            #interface_inits
            let instance = #name {
                #interface_fields
                #ref_count_ident: std::cell::Cell::new(1),
                #(#user_fields),*
            };
            let instance = ::std::mem::ManuallyDrop::new(::std::boxed::Box::pin(instance));
            instance.query()
        }
    }
}

/// Function used to instantiate a class using a raw IID pointer
pub fn generate_unsafe(class: &Class) -> TokenStream {
    let name = &class.name;
    let vis = &class.visibility;

    let user_fields = class.fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: ::std::mem::ManuallyDrop::new(::std::default::Default::default())
        }
    });

    let interface_inits = gen_vpointer_inits(class);
    let ref_count_ident = crate::utils::ref_count_ident();

    let interfaces = &class.interfaces;
    let interface_fields = gen_allocate_interface_fields(interfaces);

    quote! {
        /// Allocate the COM class from a raw IID writing the interface to `interface` pointer
        ///
        /// # Safety
        /// This function is unsafe because the pointers are not guaranteed to be valid. Enure
        /// that the pointers are non-null and pointing to valid data.
        #vis unsafe fn allocate_to_interface(
            iid: *const ::com::sys::IID,
            interface: *mut *mut ::std::ffi::c_void,
        ) -> ::com::sys::HRESULT {
            #interface_inits
            let instance = #name {
                #interface_fields
                #ref_count_ident: std::cell::Cell::new(1),
                #(#user_fields),*
            };
            let instance = ::std::mem::ManuallyDrop::new(::std::boxed::Box::pin(instance));
            instance.query_interface(iid, interface)
        }
    }
}

// Generate the vptr field idents needed in the instantiation syntax of the COM struct.
fn gen_allocate_interface_fields(interface_idents: &[Interface]) -> TokenStream {
    let base_fields = interface_idents
        .iter()
        .enumerate()
        .map(|(index, _)| quote::format_ident!("__{}", index));

    quote!(#(#base_fields,)*)
}

// Initialise VTables with the correct adjustor thunks
fn gen_vpointer_inits(class: &Class) -> TokenStream {
    let interface_inits = class.interfaces
        .iter()
        .enumerate()
        .map(move |(index,  interface)| {
            let interface = interface.to_initialized_vtable_tokens(class, index);
            let vptr_field_ident = quote::format_ident!("__{}", index);
            quote! {
                let #vptr_field_ident = unsafe { ::std::ptr::NonNull::new_unchecked(Box::into_raw(Box::new(#interface))) };
            }
        });

    quote!(#(#interface_inits)*)
}
