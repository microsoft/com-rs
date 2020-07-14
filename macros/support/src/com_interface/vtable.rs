use super::vptr;
use super::Interface;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;
use syn::{FnArg, TraitItemMethod, Type};

/// Generate an VTable for an interface
pub fn generate(interface: &Interface) -> TokenStream {
    let interface_ident = &interface.name;
    let vtable_ident = ident(&interface_ident.to_string());
    let base_field = match interface.parent {
        Some(ref parent) => {
            let last_ident = &parent
                .segments
                .last()
                .expect("Supertrait has empty path")
                .ident;
            let base_field_ident = base_field_ident(&last_ident.to_string());
            quote! {
                pub #base_field_ident: <#parent as com::ComInterface>::VTable,
            }
        }
        None => quote! {},
    };
    let methods = gen_vtable_methods(&interface);

    quote!(
        #[allow(non_snake_case, missing_docs)]
        #[repr(C)]
        #[derive(com::VTable)]
        pub struct #vtable_ident {
            #base_field
            #methods
        }
    )
}

pub fn ident(interface_name: &str) -> Ident {
    format_ident!("{}VTable", interface_name)
}

fn base_field_ident(base_interface_name: &str) -> Ident {
    format_ident!("{}_base", crate::utils::camel_to_snake(base_interface_name))
}

fn gen_vtable_methods(interface: &Interface) -> TokenStream {
    let mut methods: Vec<TokenStream> = Vec::new();
    for m in &interface.items {
        methods.push(gen_vtable_method(&interface.name, m));
    }

    quote!(
        #(#methods)*
    )
}

fn gen_vtable_method(interface_ident: &Ident, method: &TraitItemMethod) -> TokenStream {
    let method_ident = format_ident!(
        "{}",
        crate::utils::snake_to_camel(&method.sig.ident.to_string())
    );
    let vtable_function_signature = gen_vtable_function_signature(interface_ident, method);

    quote!(
        pub #method_ident: #vtable_function_signature,
    )
}

fn gen_vtable_function_signature(interface_ident: &Ident, method: &TraitItemMethod) -> TokenStream {
    let params = gen_raw_params(interface_ident, method);
    let return_type = &method.sig.output;

    quote!(
        unsafe extern "stdcall" fn(#params) #return_type
    )
}

fn gen_raw_params(interface_ident: &Ident, method: &TraitItemMethod) -> TokenStream {
    let mut params = Vec::new();
    let vptr_ident = vptr::ident(&interface_ident);

    for param in method.sig.inputs.iter() {
        match param {
            FnArg::Receiver(s) => {
                assert!(
                    s.reference.is_some(),
                    "COM interface methods cannot take ownership of self"
                );
                assert!(
                    s.mutability.is_none(),
                    "COM interface methods cannot take mutable reference to self"
                );
                params.push(quote!(
                    ::std::ptr::NonNull<#vptr_ident>,
                ));
            }
            FnArg::Typed(t) => {
                params.push(gen_raw_type(&*t.ty));
            }
        }
    }

    TokenStream::from_iter(params)
}

fn gen_raw_type(t: &Type) -> TokenStream {
    match t {
        Type::Array(_n) => panic!("Array type unhandled!"),
        Type::BareFn(_n) => panic!("BareFn type unhandled!"),
        Type::Group(_n) => panic!("Group type unhandled!"),
        Type::ImplTrait(_n) => panic!("ImplTrait type unhandled!"),
        Type::Infer(_n) => panic!("Infer type unhandled!"),
        Type::Macro(_n) => panic!("TypeMacro type unhandled!"),
        Type::Never(_n) => panic!("TypeNever type unhandled!"),
        Type::Paren(_n) => panic!("Paren type unhandled!"),
        Type::Path(_n) => quote!(#t,),
        Type::Ptr(_n) => quote!(#t,),
        Type::Reference(_n) => panic!("Reference type unhandled!"),
        Type::Slice(_n) => panic!("Slice type unhandled!"),
        Type::TraitObject(_n) => panic!("TraitObject type unhandled!"),
        Type::Tuple(_n) => panic!("Tuple type unhandled!"),
        Type::Verbatim(_n) => panic!("Verbatim type unhandled!"),
        _ => panic!("Rest unhandled!"),
    }
}
