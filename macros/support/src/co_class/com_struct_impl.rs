use super::CoClass;
use proc_macro2::TokenStream as HelperTokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Ident, ItemStruct};

pub fn generate(co_class: &CoClass) -> HelperTokenStream {
    let allocate_fn = gen_allocate_fn(co_class);
    let struct_ident = &co_class.name;

    // let get_class_object_fn = gen_get_class_object_fn(struct_item);

    quote!(
        impl #struct_ident {
            #allocate_fn
            // #get_class_object_fn
        }
    )
}

/// Function used to instantiate the COM fields, such as vpointers for the COM object.
pub fn gen_allocate_fn(co_class: &CoClass) -> HelperTokenStream {
    let name = &co_class.name;

    // Allocate function signature
    let allocate_parameters = &co_class.fields;

    let base_inits = gen_allocate_base_inits(name, &co_class.interfaces);

    // Syntax for instantiating the fields of the struct.
    let base_fields = gen_allocate_base_fields(&co_class.interfaces);
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
pub fn gen_allocate_user_fields(co_class: &CoClass) -> HelperTokenStream {
    let field_idents = co_class.fields.iter().map(|f| &f.ident);

    quote!(#(#field_idents,)*)
}

// Reference count field initialisation.
pub fn gen_allocate_ref_count_field() -> HelperTokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote!(
        #ref_count_ident: std::cell::Cell::new(0),
    )
}

// Generate the vptr field idents needed in the instantiation syntax of the COM struct.
pub fn gen_allocate_base_fields(interface_idents: &[syn::Path]) -> HelperTokenStream {
    let base_fields = interface_idents
        .iter()
        .enumerate()
        .map(|(index, _)| quote::format_ident!("__{}", index));

    quote!(#(#base_fields,)*)
}

// Initialise VTables with the correct adjustor thunks, through the vtable! macro.
pub fn gen_allocate_base_inits(
    name: &Ident,
    base_interface_idents: &[syn::Path],
) -> HelperTokenStream {
    let base_inits = base_interface_idents
        .iter()
        .enumerate()
        .map(|(index, interface)| {
            let vptr_field_ident = format_ident!("__{}", index);

            let out = quote!(
                let #vptr_field_ident = com::vtable!(#name: #interface, #index);
                let #vptr_field_ident = Box::into_raw(Box::new(#vptr_field_ident));
            );

            out
        });

    quote!(#(#base_inits)*)
}

/// Function used by in-process DLL macro to get an instance of the
/// class object.
pub fn gen_get_class_object_fn(struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let class_factory_ident = crate::utils::class_factory_ident(&struct_ident);

    quote!(
        pub fn get_class_object() -> Box<#class_factory_ident> {
            <#class_factory_ident>::new()
        }
    )
}
