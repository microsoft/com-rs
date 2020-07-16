use super::CoClass;
use proc_macro2::TokenStream;
use quote::quote;

/// The COM class object
///
/// Structure of the object:
/// pub struct _ {
///     ..base interface vpointers..
///     ..ref count..
///     ..fields..
/// }
pub fn generate(co_class: &CoClass) -> TokenStream {
    let struct_ident = &co_class.name;
    let vis = &co_class.visibility;

    let base_fields = gen_base_fields(&co_class.interfaces);
    let ref_count_field = gen_ref_count_field();
    let user_fields = &co_class.fields;
    let docs = &co_class.docs;

    quote!(
        #(#docs)*
        #[repr(C)]
        #vis struct #struct_ident {
            #base_fields
            #ref_count_field
            #(#user_fields)*,
        }
    )
}

pub fn gen_base_fields(base_interface_idents: &[syn::Path]) -> TokenStream {
    let bases_interface_idents = base_interface_idents
        .iter()
        .enumerate()
        .map(|(index, base)| {
            let field_ident = quote::format_ident!("__{}", index);
            quote!(#field_ident: ::std::ptr::NonNull<<#base as ::com::ComInterface>::VTable>)
        });
    quote!(#(#bases_interface_idents,)*)
}

pub fn gen_ref_count_field() -> TokenStream {
    let ref_count_ident = crate::utils::ref_count_ident();
    quote!(#ref_count_ident: ::std::cell::Cell<u32>,)
}
