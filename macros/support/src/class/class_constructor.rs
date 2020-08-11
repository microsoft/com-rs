use super::{class::Interface, Class};
use proc_macro2::TokenStream;
use quote::quote;

/// Function used to instantiate the COM fields, such as vpointers for the COM object.
pub fn generate(class: &Class) -> TokenStream {
    let name = &class.name;
    let vis = &class.visibility;

    let parameters = &class.fields;
    let user_fields = class.fields.iter().map(|f| &f.ident);

    let interface_inits = gen_vpointer_inits(class);
    let ref_count_ident = crate::utils::ref_count_ident();

    let interfaces = &class.interfaces;
    let interface_fields = gen_allocate_interface_fields(interfaces);

    quote! {
        #vis fn new(#(#parameters),*) -> #name {
            #interface_inits

            #name {
                #interface_fields
                #ref_count_ident: std::cell::Cell::new(0),
                #(#user_fields),*
            }
        }

        #vis fn allocate<T: ::com::Interface>(instance: Self) -> Option<T> {
            let instance = ::std::boxed::Box::pin(instance);
            let mut result = None;
            let hr = unsafe {
                instance.add_ref();
                let hr = instance.query_interface(&T::IID, &mut result as *mut _ as _);
                instance.release();
                hr
            };

            if ::com::sys::FAILED(hr) {
                assert!(
                    hr == ::com::sys::E_NOINTERFACE || hr == ::com::sys::E_POINTER,
                    "QueryInterface returned non-standard error"
                );
                return None;
            }
            debug_assert!(result.is_some());
            result
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
