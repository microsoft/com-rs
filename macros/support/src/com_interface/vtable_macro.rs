use proc_macro2::{Ident, TokenStream as HelperTokenStream};
use quote::{format_ident, quote};
use syn::{ItemStruct, Type, TypeBareFn};

pub fn generate(item: &ItemStruct) -> HelperTokenStream {
    let vtable_macro = ident(&item.ident);
    let parent_vtable_binding = gen_parent_vtable_binding(item);
    let vtable_functions = gen_vtable_functions(item);
    let initialized_vtable = gen_initialized_vtable(item);
    quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #vtable_macro {
            ($class:ty, $offset:ty) => {{
                #parent_vtable_binding
                #vtable_functions
                #initialized_vtable
            }};
        }
    }
}

pub fn ident(struct_ident: &Ident) -> Ident {
    format_ident!(
        "{}_gen_vtable",
        crate::utils::camel_to_snake(&struct_ident.to_string().replace("VTable", ""))
    )
}

fn gen_parent_vtable_binding(item: &ItemStruct) -> HelperTokenStream {
    let parent = item.fields.iter().nth(0);
    if let Some(parent) = parent {
        let is_base = parent
            .ident
            .as_ref()
            .map(|i| i.to_string().ends_with("_base"))
            .unwrap_or(false);
        if is_base {
            let type_path = match &parent.ty {
                syn::Type::Path(type_path) => type_path,
                _ => panic!("vtable fields types must be type paths"),
            };
            let qself = type_path
                .qself
                .as_ref()
                .expect("vtable type paths must use associated types");
            let parent_ty = &qself.ty;
            return quote! {
                let parent_vtable = <#parent_ty as com::ProductionComInterface<$class>>::vtable::<$offset>();
            };
        }
    }

    quote! {}
}

fn gen_vtable_functions(item: &ItemStruct) -> HelperTokenStream {
    let mut functions = Vec::new();
    for field in &item.fields {
        let method_name = field
            .ident
            .as_ref()
            .expect("Only works with structs with named fields");
        match &field.ty {
            Type::Path(_) => {}
            Type::BareFn(fun) => {
                functions.push(gen_vtable_function(&item.ident, method_name, fun));
            }
            _ => panic!("Only supports structs with fields that are functions"),
        };
    }
    quote! {
        #(#functions)*
    }
}

fn gen_vtable_function(
    struct_ident: &Ident,
    method_name: &Ident,
    fun: &TypeBareFn,
) -> HelperTokenStream {
    assert!(fun.unsafety.is_some(), "Function must be marked unsafe");
    assert!(fun.abi.is_some(), "Function must have marked ABI");
    let method_name = format_ident!("{}", crate::utils::camel_to_snake(&method_name.to_string()));
    let interface_name = struct_ident.to_string().replace("VTable", "");
    let interface_ident = format_ident!("{}", interface_name);
    let function_ident = format_ident!("{}_{}", interface_name.to_lowercase(), method_name);
    let params: Vec<_> = fun
        .inputs
        .iter()
        .enumerate()
        .map(|(i, input)| {
            let ident = format_ident!("arg{}", i);
            let ty = &input.ty;
            quote! { #ident: #ty, }
        })
        .collect();
    let args = (0..(params.len())).skip(1).map(|i| {
        let ident = format_ident!("arg{}", i);
        quote! { #ident, }
    });
    let return_type = &fun.output;
    quote! {
        #[allow(missing_docs)]
        unsafe extern "stdcall" fn #function_ident<O: com::offset::Offset>(#(#params)*) #return_type {
            let this = arg0.sub(O::VALUE) as *const #interface_ident as *mut #interface_ident;
            (*this).#method_name(#(#args)*)
        }
    }
}

fn gen_initialized_vtable(item: &ItemStruct) -> HelperTokenStream {
    let name = &item.ident;
    let methods = gen_vtable_method_initialization(item);
    quote! {
        #[allow(missing_docs)]
        #name {
            #methods
        }
    }
}

fn gen_vtable_method_initialization(item: &ItemStruct) -> HelperTokenStream {
    let mut methods = Vec::new();
    for field in &item.fields {
        let method_ident = field
            .ident
            .as_ref()
            .expect("Only works with structs with named fields");

        let function_ident = if method_ident.to_string().ends_with("_base") {
            let parent = format_ident!("parent_vtable",);
            quote! { #parent, }
        } else {
            let function_ident = format_ident!(
                "{}_{}",
                item.ident.to_string().replace("VTable", "").to_lowercase(),
                crate::utils::camel_to_snake(&method_ident.to_string())
            );
            quote! {
                #function_ident::<$class, $offset>,
            }
        };
        let method = quote! {
            #method_ident: #function_ident
        };
        methods.push(method);
    }

    quote!(
        #(#methods)*
    )
}
