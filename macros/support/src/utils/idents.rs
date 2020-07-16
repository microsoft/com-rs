use quote::format_ident;
use syn::Ident;

pub fn class_factory_ident(class_ident: &Ident) -> Ident {
    format_ident!("{}ClassFactory", class_ident)
}

pub fn ref_count_ident() -> Ident {
    format_ident!("__refcnt")
}
