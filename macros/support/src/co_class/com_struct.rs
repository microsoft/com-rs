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
    let name = &co_class.name;
    let vis = &co_class.visibility;

    let interfaces = co_class.interfaces.keys().collect::<Vec<_>>();
    let interface_fields = gen_interface_fields(&interfaces);
    let ref_count_ident = crate::utils::ref_count_ident();

    let user_fields = &co_class.fields;
    let docs = &co_class.docs;
    let methods = co_class.methods.values().flat_map(|ms| ms);

    let iunknown = super::iunknown_impl::IUnknown::new(name.clone());
    let add_ref = iunknown.to_add_ref_tokens();
    let release = iunknown.to_release_tokens(&interfaces);
    let query_interface = iunknown.to_query_interface_tokens(&interfaces);

    quote!(
        #(#docs)*
        #[repr(C)]
        #vis struct #name {
            #interface_fields
            #ref_count_ident: ::std::cell::Cell<u32>,
            #(#user_fields)*,
        }
        impl #name {
            #(#methods)*
            #add_ref
            #release
            #query_interface
        }
    )
}

pub fn gen_interface_fields(interface_idents: &[&syn::Path]) -> TokenStream {
    let interface_idents = interface_idents.iter().enumerate().map(|(index, base)| {
        let field_ident = quote::format_ident!("__{}", index);
        quote!(#field_ident: ::std::ptr::NonNull<<#base as ::com::ComInterface>::VTable>)
    });
    quote!(#(#interface_idents,)*)
}
