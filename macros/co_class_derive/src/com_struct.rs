use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{ItemStruct, Ident,};

// #[repr(C)]
// pub struct BritishShortHairCat {
//     __icatvptr: <dyn ICat as com::ComInterface>::VPtr,
//     __idomesticanimalvptr: <dyn IDomesticAnimal as com::ComInterface>::VPtr,
//     __refcnt: u32,
//     __init_struct: InitBritishShortHairCat,
// }

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

    quote!(
        #[repr(C)]
        #vis struct #real_ident {
            #(#bases_itf_idents,)*
            #ref_count_ident: u32,
            #inner_init_field_ident: #init_ident
        }
    )
}
