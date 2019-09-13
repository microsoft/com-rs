use proc_macro2::TokenStream as HelperTokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Fields, Ident, ItemStruct};

/// Generates the allocate and get_class_object function for the COM object.
/// allocate: instantiates the COM fields, such as vpointers for the COM object.
/// get_class_object: Instantiate an instance to the class object.
pub fn generate(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    // Allocate stuff
    let mut offset_count: usize = 0;
    let base_inits = base_interface_idents.iter().map(|base| {
        let vtable_var_ident = format_ident!("{}_vtable", base.to_string().to_lowercase());
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

    // GetClassObject stuff
    let class_factory_ident = macro_utils::class_factory_ident(&struct_ident);

    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct."),
    };
    let field_idents = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap().clone();
        quote!(#field_ident)
    });

    let aggregate_inits = aggr_map.iter().map(|(aggr_field_ident, _)| {
        quote!(
            #aggr_field_ident: std::ptr::null_mut()
        )
    });

    let set_aggregate_fns = gen_set_aggregate_fns(aggr_map);

    quote!(
        impl #struct_ident {
            fn allocate(#fields) -> Box<#struct_ident> {
                println!("Allocating new VTable for {}", stringify!(#struct_ident));
                #(#base_inits)*
                let out = #struct_ident {
                    #(#base_fields,)*
                    #ref_count_ident: std::cell::Cell::new(0),
                    #(#aggregate_inits,)*
                    #(#field_idents)*
                };
                Box::new(out)
            }

            pub fn get_class_object() -> Box<#class_factory_ident> {
                <#class_factory_ident>::new()
            }

            #set_aggregate_fns
        }
    )
}

fn gen_set_aggregate_fns(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
    let mut fns = Vec::new();
    for (aggr_field_ident, aggr_base_interface_idents) in aggr_map.iter() {
        for base in aggr_base_interface_idents {
            let set_aggregate_fn_ident = format_ident!(
                "set_aggregate_{}",
                macro_utils::camel_to_snake(&base.to_string())
            );
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
