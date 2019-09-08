use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{Ident, ItemStruct};

// #[repr(C)]
// pub struct LocalFileManager {
//     __ilocalfilemanagervptr: <dyn ILocalFileManager as com::ComInterface>::VPtr,
//     __non_delegating_unk: <dyn com::IUnknown as com::ComInterface>::VPtr,
//     __iunk_to_use: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
//     __refcnt: u32,
//     __init_struct: InitLocalFileManager,
// }

/// As an aggregable COM object, you need to have an inner non-delegating IUnknown vtable.
/// All IUnknown calls to this COM object will delegate to the IUnknown interface pointer
/// __iunk_to_use.
pub fn generate(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let vis = &struct_item.vis;

    let bases_itf_idents = base_itf_idents.iter().map(|base| {
        let field_ident = macro_utils::get_vptr_field_ident(&base);
        quote!(#field_ident: <dyn #base as com::ComInterface>::VPtr)
    });

    let ref_count_ident = macro_utils::get_ref_count_ident();
    let inner_init_field_ident = macro_utils::get_inner_init_field_ident();
    let non_del_unk_field_ident = macro_utils::get_non_del_unk_field_ident();
    let iunk_to_use_field_ident = macro_utils::get_iunk_to_use_field_ident();

    quote!(
        #[repr(C)]
        #vis struct #real_ident {
            #(#bases_itf_idents,)*
            #non_del_unk_field_ident: <dyn com::IUnknown as com::ComInterface>::VPtr,
            // Non-reference counted interface pointer to outer IUnknown.
            #iunk_to_use_field_ident: *mut <dyn com::IUnknown as com::ComInterface>::VPtr,
            #ref_count_ident: u32,
            #inner_init_field_ident: #init_ident
        }
    )
}
