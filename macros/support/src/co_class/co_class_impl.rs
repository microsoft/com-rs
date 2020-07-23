use super::CoClass;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(co_class: &CoClass) -> TokenStream {
    if co_class.class_factory {
        return TokenStream::new();
    }

    let name = &co_class.name;
    let factory = crate::utils::class_factory_ident(name);

    quote! {
        unsafe impl com::production::CoClass for #name {
            type Factory = #factory;
        }
    }
}
