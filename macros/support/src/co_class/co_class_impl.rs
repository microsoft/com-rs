use super::CoClass;
use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;

pub fn generate(struct_item: &CoClass) -> HelperTokenStream {
    let struct_ident = &struct_item.name;

    quote! {
        unsafe impl com::CoClass for #struct_ident {}
    }
}
