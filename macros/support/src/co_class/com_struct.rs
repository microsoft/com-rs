use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Fields, Ident, ItemStruct};

/// The actual COM object that wraps around the Init struct.
/// Structure of the object:
/// pub struct _ {
///     ..base interface vpointers..
///     ..ref count..
///     ..init struct..
/// }
pub fn generate(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let vis = &struct_item.vis;

    let base_fields = gen_base_fields(base_interface_idents);
    let ref_count_field = gen_ref_count_field();
    let user_fields = gen_user_fields(struct_item);
    let aggregate_fields = gen_aggregate_fields(aggr_map);

    quote!(
        #[repr(C)]
        #vis struct #struct_ident {
            #base_fields
            #ref_count_field
            #aggregate_fields
            #user_fields
        }
    )
}

pub fn gen_base_fields(base_interface_idents: &[Ident]) -> HelperTokenStream {
    let bases_interface_idents = base_interface_idents.iter().map(|base| {
        let field_ident = crate::utils::vptr_field_ident(&base);
        quote!(#field_ident: *const <dyn #base as com::ComInterface>::VTable)
    });
    quote!(#(#bases_interface_idents,)*)
}

pub fn gen_ref_count_field() -> HelperTokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote!(#ref_count_ident: std::cell::Cell<u32>,)
}

pub fn gen_aggregate_fields(aggr_map: &HashMap<Ident, Vec<Ident>>) -> HelperTokenStream {
    let aggregates = aggr_map.iter().map(|(aggr_field_ident, _)| {
        quote!(
            #aggr_field_ident: *mut *const <dyn com::interfaces::iunknown::IUnknown as com::ComInterface>::VTable
        )
    });

    quote!(#(#aggregates,)*)
}

pub fn gen_user_fields(struct_item: &ItemStruct) -> HelperTokenStream {
    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct."),
    };

    quote!(#fields)
}
