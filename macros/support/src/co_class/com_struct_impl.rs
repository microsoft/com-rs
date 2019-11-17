use proc_macro2::TokenStream as HelperTokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_quote, Fields, Ident, ItemStruct, Type};

pub fn generate(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();

    let allocate_fn = gen_allocate_fn(aggr_map, base_interface_idents, struct_item);
    let set_aggregate_fns = gen_set_aggregate_fns(aggr_map);
    let get_class_object_fn = gen_get_class_object_fn(struct_item);

    quote!(
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            #allocate_fn
            #get_class_object_fn
            #set_aggregate_fns
        }
    )
}

/// Function used to instantiate the COM fields, such as vpointers for the COM object.
pub fn gen_allocate_fn(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let (_, ty_generics, _) = struct_item.generics.split_for_impl();
    let struct_type = parse_quote!(#struct_ident #ty_generics);

    let base_inits = gen_allocate_base_inits(&struct_type, base_interface_idents);

    // Allocate function signature
    let allocate_parameters = gen_allocate_function_parameters_signature(struct_item);

    // Syntax for instantiating the fields of the struct.
    let base_fields = gen_allocate_base_fields(base_interface_idents);
    let ref_count_field = gen_allocate_ref_count_field();
    let user_fields = gen_allocate_user_fields(struct_item);
    let aggregate_fields = gen_allocate_aggregate_fields(aggr_map);

    // Initialise all aggregated objects as NULL.
    quote!(
        fn allocate(#allocate_parameters) -> Box<#struct_ident #ty_generics> {
            #base_inits

            let out = #struct_ident {
                #base_fields
                #ref_count_field
                #aggregate_fields
                #user_fields
            };
            Box::new(out)
        }
    )
}

pub fn gen_allocate_function_parameters_signature(struct_item: &ItemStruct) -> HelperTokenStream {
    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct."),
    };

    quote!(#fields)
}

pub fn gen_allocate_aggregate_fields(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
    let aggregate_inits = aggr_map.iter().map(|(aggr_field_ident, _)| {
        quote!(
            #aggr_field_ident: std::ptr::null_mut()
        )
    });

    quote!(#(#aggregate_inits,)*)
}

// User field input as parameters to the allocate function.
pub fn gen_allocate_user_fields(struct_item: &ItemStruct) -> HelperTokenStream {
    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct."),
    };
    let field_idents = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("Field has no ident").clone();
        quote!(#field_ident)
    });

    quote!(#(#field_idents)*)
}

// Reference count field initialisation.
pub fn gen_allocate_ref_count_field() -> HelperTokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote!(
        #ref_count_ident: std::cell::Cell::new(0),
    )
}

// Generate the vptr field idents needed in the instantiation syntax of the COM struct.
pub fn gen_allocate_base_fields(base_interface_idents: &[Ident]) -> HelperTokenStream {
    let base_fields = base_interface_idents.iter().map(|base| {
        let vptr_field_ident = crate::utils::vptr_field_ident(base);
        quote!(#vptr_field_ident)
    });

    quote!(#(#base_fields,)*)
}

// Initialise VTables with the correct adjustor thunks, through the vtable! macro.
pub fn gen_allocate_base_inits(
    struct_type: &Type,
    base_interface_idents: &[Ident],
) -> HelperTokenStream {
    let mut offset_count: usize = 0;
    let base_inits = base_interface_idents.iter().map(|base| {
        let vtable_var_ident = format_ident!("{}_vtable", base.to_string().to_lowercase());
        let vptr_field_ident = crate::utils::vptr_field_ident(&base);

        let out = quote!(
            let #vtable_var_ident = com::vtable!(#struct_type: #base, #offset_count);
            let #vptr_field_ident = Box::into_raw(Box::new(#vtable_var_ident));
        );

        offset_count += 1;
        out
    });

    quote!(#(#base_inits)*)
}

/// Function used by in-process DLL macro to get an instance of the
/// class object.
pub fn gen_get_class_object_fn(struct_item: &ItemStruct) -> HelperTokenStream {
    // Generic structs don't have class objects so we'll return an empty token stream for those.
    if struct_item.generics.lt_token.is_some() {
        return quote!();
    }

    let struct_ident = &struct_item.ident;
    let class_factory_ident = crate::utils::class_factory_ident(&struct_ident);

    quote!(
        pub fn get_class_object() -> Box<#class_factory_ident> {
            <#class_factory_ident>::new()
        }
    )
}

pub fn gen_set_aggregate_fns(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
    let mut fns = Vec::new();
    for (aggr_field_ident, aggr_base_interface_idents) in aggr_map.iter() {
        for base in aggr_base_interface_idents {
            let set_aggregate_fn_ident = crate::utils::set_aggregate_fn_ident(&base);
            fns.push(quote!(
                fn #set_aggregate_fn_ident(&mut self, aggr: com::InterfacePtr<dyn com::interfaces::iunknown::IUnknown>) {
                    // TODO: What happens if we are overwriting an existing aggregate?
                    self.#aggr_field_ident = aggr.as_raw() as *mut *const <dyn com::interfaces::iunknown::IUnknown as com::ComInterface>::VTable;
                }
            ));
        }
    }

    quote!(#(#fns)*)
}
