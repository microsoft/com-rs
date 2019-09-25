use proc_macro2::TokenStream as HelperTokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn generate(struct_item: &ItemStruct) -> HelperTokenStream {
    let struct_ident = &struct_item.ident;

    quote! {
        unsafe impl com::CoClass for #struct_ident {
        }
    }
}
