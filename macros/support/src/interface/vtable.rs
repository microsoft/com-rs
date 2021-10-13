use super::vptr;
use super::{Interface, InterfaceMethod};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::Type;

/// Generate an VTable for an interface
pub fn generate(interface: &Interface) -> syn::Result<TokenStream> {
    let interface_ident = &interface.name;
    let vtable_ident = ident(&interface_ident.to_string());
    let parent_field = match interface.parent {
        Some(ref parent) => {
            quote! {
                pub parent: <#parent as com::Interface>::VTable,
            }
        }
        None => quote! {},
    };
    let methods = gen_vtable_methods(interface)?;
    let vis = &interface.visibility;

    Ok(quote!(
        #[allow(non_snake_case, missing_docs)]
        #[repr(C)]
        #vis struct #vtable_ident {
            #parent_field
            #methods
        }
    ))
}

pub fn ident(interface_name: &str) -> Ident {
    format_ident!("{}VTable", interface_name)
}

fn gen_vtable_methods(interface: &Interface) -> syn::Result<TokenStream> {
    let mut methods: Vec<TokenStream> = Vec::new();
    for m in interface.methods.iter() {
        methods.push(gen_vtable_method(&interface.name, m)?);
    }

    Ok(quote!(
        #(#methods)*
    ))
}

fn gen_vtable_method(
    interface_ident: &Ident,
    method: &InterfaceMethod,
) -> syn::Result<TokenStream> {
    let method_ident = format_ident!("{}", crate::utils::snake_to_camel(&method.name.to_string()));
    let vtable_function_signature = gen_vtable_function_signature(interface_ident, method)?;

    Ok(quote!(
        pub #method_ident: #vtable_function_signature,
    ))
}

fn gen_vtable_function_signature(
    interface_ident: &Ident,
    method: &InterfaceMethod,
) -> syn::Result<TokenStream> {
    let params = gen_raw_params(interface_ident, method)?;
    let return_type = &method.ret;

    Ok(quote!(
        unsafe extern "system" fn(#params) #return_type
    ))
}

fn gen_raw_params(interface_ident: &Ident, method: &InterfaceMethod) -> syn::Result<TokenStream> {
    let vptr_ident = vptr::ident(interface_ident);
    let mut params = quote!(
        ::core::ptr::NonNull<#vptr_ident>,
    );

    for param in method.args.iter() {
        params.extend(gen_raw_type(param)?);
    }

    Ok(params)
}

fn gen_raw_type(p: &super::interface::InterfaceMethodArg) -> syn::Result<TokenStream> {
    let t = &*p.ty;
    let ty = match t {
        Type::Path(_) | Type::Ptr(_) if !p.pass_through => {
            return Ok(quote!(<#t as ::com::AbiTransferable>::Abi,))
        }
        Type::Path(_) | Type::Ptr(_) => return Ok(quote!(#t,)),
        Type::Array(_n) => "array type",
        Type::BareFn(_n) => "barefn type",
        Type::Group(_n) => "group type",
        Type::ImplTrait(_n) => "implTrait type",
        Type::Infer(_n) => "infer type",
        Type::Macro(_n) => "typeMacro type",
        Type::Never(_n) => "typeNever type",
        Type::Paren(_n) => "paren type",
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
