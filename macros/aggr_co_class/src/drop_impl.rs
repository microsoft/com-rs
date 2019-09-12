use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemStruct};

pub fn generate(
    aggr_map: &HashMap<Ident, Vec<Ident>>,
    base_interface_idents: &[Ident],
    struct_item: &ItemStruct,
) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    let aggregate_drops = co_class::drop_impl::gen_aggregate_drops(aggr_map);
    let vptr_drops = co_class::drop_impl::gen_vptr_drops(base_interface_idents);
    let non_delegating_iunknown_drop = gen_non_delegating_iunknown_drop();

    quote!(
        impl std::ops::Drop for #struct_ident {
            fn drop(&mut self) {
                let _ = unsafe {
                    #aggregate_drops
                    #vptr_drops
                    #non_delegating_iunknown_drop
                };
            }
        }
    )
}


fn gen_non_delegating_iunknown_drop() -> HelperTokenStream {
    let non_delegating_iunknown_field_ident = macro_utils::non_delegating_iunknown_field_ident();
    quote!(
        Box::from_raw(self.#non_delegating_iunknown_field_ident as *mut <dyn com::IUnknown as com::ComInterface>::VTable)
    )
}