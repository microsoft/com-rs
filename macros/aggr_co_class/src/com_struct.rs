use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{Ident, ItemStruct, Fields,};
use std::collections::HashMap;

/// As an aggregable COM object, you need to have an inner non-delegating IUnknown vtable.
/// All IUnknown calls to this COM object will delegate to the IUnknown interface pointer
/// __iunknown_to_use.
pub fn generate(aggr_map: &HashMap<Ident, Vec<Ident>>, base_interface_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let vis = &struct_item.vis;

    let bases_interface_idents = base_interface_idents.iter().map(|base| {
        let field_ident = macro_utils::get_vptr_field_ident(&base);
        quote!(#field_ident: <dyn #base as com::ComInterface>::VPtr)
    });

    let ref_count_ident = macro_utils::get_ref_count_ident();
    let non_delegating_iunknown_field_ident = macro_utils::get_non_delegating_iunknown_field_ident();
    let iunknown_to_use_field_ident = macro_utils::get_iunknown_to_use_field_ident();

    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct.")
    };

    let aggregates = aggr_map.iter().map(|(aggr_field_ident, aggr_base_interface_idents)| {
        quote!(
            #aggr_field_ident: *mut <dyn com::IUnknown as com::ComInterface>::VPtr
        )
    });

    quote!(
        #[repr(C)]
        #vis struct #struct_ident {
            #(#bases_interface_idents,)*
            #non_delegating_iunknown_field_ident: <dyn com::IUnknown as com::ComInterface>::VPtr,
            // Non-reference counted interface pointer to outer IUnknown.
            #iunknown_to_use_field_ident: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
            #ref_count_ident: u32,
            #(#aggregates,)*
            #fields
        }
    )
}
