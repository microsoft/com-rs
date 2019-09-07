use macro_utils;

use proc_macro2::{Ident, TokenStream as HelperTokenStream};
use quote::{format_ident, quote};
use syn::{ItemStruct, Type, TypeBareFn};

pub fn generate(item: &ItemStruct) -> HelperTokenStream {
    let vtable_macro = ident(&item.ident);
    let parent_vtable_binding = gen_parent_vtable_binding(item);
    let vtable_functions = gen_vtable_functions(item);
    let initialized_vtable = gen_initialized_vtable(item);
    quote! {
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
        macro_utils::camel_to_snake(&struct_ident.to_string().replace("VTable", ""))
    )
}

fn gen_parent_vtable_binding(item: &ItemStruct) -> HelperTokenStream {
    let parent = item.fields.iter().nth(0).and_then(|f| f.ident.as_ref());
    if let Some(parent) = parent {
        let parent = parent.to_string();
        if parent.ends_with("_base") {
            let parent = format_ident!(
                "I{}",
                macro_utils::snake_to_camel(
                    parent.trim_end_matches("_base").trim_start_matches("i")
                )
            );
            return quote! {
                let parent_vtable = <dyn #parent as com::ProductionComInterface<$class>>::vtable::<$offset>();
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
    let method_name = format_ident!("{}", macro_utils::camel_to_snake(&method_name.to_string()));
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
        unsafe extern "stdcall" fn #function_ident<C: #interface_ident, O: com::offset::Offset>(#(#params)*) #return_type {
            let this = arg0.sub(O::VALUE) as *mut C;
            (*this).#method_name(#(#args)*)
        }
    }
}

fn gen_initialized_vtable(item: &ItemStruct) -> HelperTokenStream {
    let name = &item.ident;
    let methods = gen_vtable_method_initialization(item);
    quote! {
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
                macro_utils::camel_to_snake(&method_ident.to_string())
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
