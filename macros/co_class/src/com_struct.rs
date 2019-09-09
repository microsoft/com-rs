use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::{Ident, ItemStruct, Fields};

// #[repr(C)]
// pub struct BritishShortHairCat {
//     __icatvptr: <dyn ICat as com::ComInterface>::VPtr,
//     __idomesticanimalvptr: <dyn IDomesticAnimal as com::ComInterface>::VPtr,
//     __refcnt: u32,
//     __init_struct: InitBritishShortHairCat,
// }

/// The actual COM object that wraps around the Init struct.
/// Structure of the object:
/// pub struct _ {
///     ..base interface vpointers..
///     ..ref count..
///     ..init struct..
/// }
pub fn generate(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;
    let vis = &struct_item.vis;

    let bases_itf_idents = base_itf_idents.iter().map(|base| {
        let field_ident = macro_utils::get_vptr_field_ident(&base);
        quote!(#field_ident: <dyn #base as com::ComInterface>::VPtr)
    });

    let ref_count_ident = macro_utils::get_ref_count_ident();

    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found non Named fields in struct.")
    };

    quote!(
        #[repr(C)]
        #vis struct #struct_ident {
            #(#bases_itf_idents,)*
            #ref_count_ident: u32,
            #fields
        }
    )
}
