use crate::co_class;
use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemStruct};

/// As an aggregable COM object, you need to have an inner non-delegating IUnknown vtable.
/// All IUnknown calls to this COM object will delegate to the IUnknown interface pointer
/// __iunknown_to_use.
pub fn generate(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let vis = &struct_item.vis;

    let base_fields = co_class::com_struct::gen_base_fields(base_interface_idents);
    let ref_count_field = co_class::com_struct::gen_ref_count_field();
    let user_fields = co_class::com_struct::gen_user_fields(struct_item);
    let aggregate_fields = co_class::com_struct::gen_aggregate_fields(aggr_map);

    // COM Fields for an aggregable coclass.
    let non_delegating_iunknown_field_ident = crate::utils::non_delegating_iunknown_field_ident();
    let iunknown_to_use_field_ident = crate::utils::iunknown_to_use_field_ident();

    quote!(
        #[repr(C)]
        #vis struct #struct_ident {
            #base_fields
            #non_delegating_iunknown_field_ident: *const <dyn com::interfaces::iunknown::IUnknown as com::ComInterface>::VTable,
            // Non-reference counted interface pointer to outer IUnknown.
            #iunknown_to_use_field_ident: *mut *const <dyn com::interfaces::iunknown::IUnknown as com::ComInterface>::VTable,
            #ref_count_field
            #aggregate_fields
            #user_fields
        }
    )
}
