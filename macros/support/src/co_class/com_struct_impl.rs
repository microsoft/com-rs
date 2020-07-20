use super::CoClass;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemStruct};

pub fn generate(co_class: &CoClass) -> TokenStream {
    let allocate_fn = gen_allocate_fn(co_class);
    let struct_ident = &co_class.name;

    let get_class_object_fn = gen_get_class_object_fn(co_class);

    quote!(
        impl #struct_ident {
            #allocate_fn
            #get_class_object_fn
        }
    )
}

/// Function used to instantiate the COM fields, such as vpointers for the COM object.
pub fn gen_allocate_fn(co_class: &CoClass) -> TokenStream {
    let name = &co_class.name;

    // Allocate function signature
    let allocate_parameters = &co_class.fields;

    let base_inits = gen_vpointer_inits(co_class);

    // Syntax for instantiating the fields of the struct.
    let interfaces = &co_class.interfaces.keys().collect::<Vec<_>>();
    let base_fields = gen_allocate_base_fields(interfaces);
    let ref_count_field = gen_allocate_ref_count_field();
    let user_fields = gen_allocate_user_fields(co_class);

    quote! {
        fn allocate(#(#allocate_parameters),*) -> Box<#name> {
            #base_inits

            let out = #name {
                #base_fields
                #ref_count_field
                #user_fields
            };
            Box::new(out)
        }
    }
}

// User field input as parameters to the allocate function.
pub fn gen_allocate_user_fields(co_class: &CoClass) -> TokenStream {
    let field_idents = co_class.fields.iter().map(|f| &f.ident);

    quote!(#(#field_idents,)*)
}

// Reference count field initialisation.
pub fn gen_allocate_ref_count_field() -> TokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote!(
        #ref_count_ident: std::cell::Cell::new(0),
    )
}

// Generate the vptr field idents needed in the instantiation syntax of the COM struct.
pub fn gen_allocate_base_fields(interface_idents: &[&syn::Path]) -> TokenStream {
    let base_fields = interface_idents
        .iter()
        .enumerate()
        .map(|(index, _)| quote::format_ident!("__{}", index));

    quote!(#(#base_fields,)*)
}

// Initialise VTables with the correct adjustor thunks, through the vtable! macro.
pub fn gen_vpointer_inits(co_class: &CoClass) -> TokenStream {
    let interface_inits = co_class.interfaces
        .iter()
        .enumerate()
        .map(move |(index, (_, interface))| {
            let interface = interface.to_initialized_vtable_tokens(co_class, index);
            let vptr_field_ident = quote::format_ident!("__{}", index);
            quote!(
                let #vptr_field_ident = unsafe { ::std::ptr::NonNull::new_unchecked(Box::into_raw(Box::new(#interface))) };
            )
        });

    quote!(#(#interface_inits)*)
}

/// Function used by in-process DLL macro to get an instance of the
/// class object.
pub fn gen_get_class_object_fn(co_class: &CoClass) -> TokenStream {
    if &co_class.name == "BritishShortHairCatClassFactory" {
        return TokenStream::new();
    }
    let name = &co_class.name;
    let class_factory_ident = crate::utils::class_factory_ident(&name);

    quote!(
        pub fn get_class_object() -> Box<#class_factory_ident> {
            <#class_factory_ident>::new()
        }
    )
}
