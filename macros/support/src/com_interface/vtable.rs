use super::vptr;
use super::{interface::MethodDecl, Interface, InterfaceMethod};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;
use syn::spanned::Spanned;
use syn::Type;

/// Generate an VTable for an interface
pub fn generate(interface: &Interface) -> syn::Result<TokenStream> {
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
    let methods = gen_vtable_methods(&interface)?;
    let vis = &interface.visibility;

    Ok(quote!(
        #[allow(non_snake_case, missing_docs)]
        #[repr(C)]
        #[derive(com::VTable)]
        #vis struct #vtable_ident {
            #base_field
            #methods
        }
    ))
}

pub fn ident(interface_name: &str) -> Ident {
    format_ident!("{}VTable", interface_name)
}

fn base_field_ident(base_interface_name: &str) -> Ident {
    format_ident!("{}_base", crate::utils::camel_to_snake(base_interface_name))
}

fn gen_vtable_methods(interface: &Interface) -> syn::Result<TokenStream> {
    let mut methods: Vec<TokenStream> = Vec::new();
    for (i, m) in interface.methods.iter().enumerate() {
        methods.push(gen_vtable_method(&interface.name, m, i)?);
    }

    Ok(quote!(
        #(#methods)*
    ))
}

fn gen_vtable_method(
    interface_ident: &Ident,
    method: &MethodDecl,
    offset: usize,
) -> syn::Result<TokenStream> {
    match method {
        MethodDecl::Method(method) => {
            let method_ident =
                format_ident!("{}", crate::utils::snake_to_camel(&method.name.to_string()));
            let vtable_function_signature = gen_vtable_function_signature(interface_ident, method)?;

            Ok(quote!(
                pub #method_ident: #vtable_function_signature,
            ))
        }
        MethodDecl::PlaceHolder(n) => {
            let mut placeholders = Vec::with_capacity(*n as usize);
            for n in 0..*n {
                let n = quote::format_ident!("fn{}{}", offset, n);
                placeholders.push(quote! {
                    #n: unsafe extern "stdcall" fn(),
                })
            }
            Ok(quote! { #(#placeholders)* })
        }
    }
}

fn gen_vtable_function_signature(
    interface_ident: &Ident,
    method: &InterfaceMethod,
) -> syn::Result<TokenStream> {
    let params = gen_raw_params(interface_ident, method)?;
    let return_type = &method.ret;

    Ok(quote!(
        unsafe extern "stdcall" fn(#params) #return_type
    ))
}

fn gen_raw_params(interface_ident: &Ident, method: &InterfaceMethod) -> syn::Result<TokenStream> {
    let vptr_ident = vptr::ident(&interface_ident);
    let mut params = vec![quote!(
        ::std::ptr::NonNull<#vptr_ident>,
    )];

    for param in method.args.iter() {
        params.push(gen_raw_type(&*param.ty)?);
    }

    Ok(TokenStream::from_iter(params))
}

fn gen_raw_type(t: &Type) -> syn::Result<TokenStream> {
    let ty = match t {
        Type::Array(_n) => "array type",
        Type::BareFn(_n) => "barefn type",
        Type::Group(_n) => "group type",
        Type::ImplTrait(_n) => "implTrait type",
        Type::Infer(_n) => "infer type",
        Type::Macro(_n) => "typeMacro type",
        Type::Never(_n) => "typeNever type",
        Type::Paren(_n) => "paren type",
        Type::Path(_) | Type::Ptr(_) => return Ok(quote!(<#t as ::com::AbiTransferable>::Abi,)),
        Type::Reference(_n) => "reference type",
        Type::Slice(_n) => "slice type",
        Type::TraitObject(_n) => "traitObject type",
        Type::Tuple(_n) => "tuple type",
        Type::Verbatim(_n) => "verbatim type",
        _ => "other type",
    };
    Err(syn::Error::new(
        t.span(),
        format!("unexpected argument type: {}", ty),
    ))
}
