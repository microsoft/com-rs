use crate::vptr;
use macro_utils;
use proc_macro2::{Ident, TokenStream as HelperTokenStream};
use quote::{format_ident, quote};
use std::iter::FromIterator;
use syn::{FnArg, ItemTrait, TraitItem, TraitItemMethod, Type, TypeParamBound};

/// Generate an VTable for an interface trait
///
/// * `interface` is a trait representing a COM interface. This trait must either be
/// IUnknown and have no super traits or some other trait that has a parent trait
pub fn generate(interface: &ItemTrait) -> HelperTokenStream {
    let interface_ident = &interface.ident;
    let vtable_ident = ident(&interface_ident.to_string());
    let base_field = if interface_ident.to_string().to_uppercase() == "IUNKNOWN" {
        assert!(
            interface.supertraits.len() == 0,
            "IUnknown is a reserved interface"
        );
        quote! {}
    } else {
        assert!(
            !(interface.supertraits.len() > 1),
            "Multiple inheirtance is not supported in COM interfaces"
        );
        assert!(
            interface.supertraits.len() != 0,
            "All interfaces must inherit from another COM interface"
        );

        let base_trait_ident = match interface.supertraits.first().unwrap() {
            TypeParamBound::Trait(t) => t.path.get_ident().unwrap(),
            _ => panic!("Unhandled super trait typeparambound"),
        };

        let base_field_ident = base_field_ident(&base_trait_ident.to_string());
        quote! {
            pub #base_field_ident: <dyn #base_trait_ident as com::ComInterface>::VTable,
        }
    };
    let methods = gen_vtable_methods(&interface);

    quote!(
        #[allow(non_snake_case)]
        #[repr(C)]
        #[derive(com::VTableMacro)]
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
    format_ident!("{}_base", macro_utils::camel_to_snake(base_interface_name))
}

fn gen_vtable_methods(interface: &ItemTrait) -> HelperTokenStream {
    let mut methods: Vec<HelperTokenStream> = Vec::new();
    for trait_item in &interface.items {
        match trait_item {
            TraitItem::Method(m) => methods.push(gen_vtable_method(&interface.ident, m)),
            _ => panic!("Interface traits currently only support methods"),
        };
    }

    quote!(
        #(#methods)*
    )
}

fn gen_vtable_method(interface_ident: &Ident, method: &TraitItemMethod) -> HelperTokenStream {
    let method_ident = format_ident!("{}", macro_utils::snake_to_camel(&method.sig.ident.to_string()));
    let vtable_function_signature = gen_vtable_function_signature(interface_ident, method);

    quote!(
        pub #method_ident: #vtable_function_signature,
    )
}

fn gen_vtable_function_signature(
    trait_ident: &Ident,
    method: &TraitItemMethod,
) -> HelperTokenStream {
    let params = gen_raw_params(trait_ident, method);
    let return_type = &method.sig.output;

    quote!(
        unsafe extern "stdcall" fn(#params) #return_type
    )
}

fn gen_raw_params(trait_ident: &Ident, method: &TraitItemMethod) -> HelperTokenStream {
    let mut params = Vec::new();
    let vptr_ident = vptr::ident(&trait_ident.to_string());
    for param in method.sig.inputs.iter() {
        match param {
            FnArg::Receiver(_n) => {
                params.push(quote!(
                    *mut #vptr_ident,
                ));
            }
            FnArg::Typed(t) => {
                params.push(gen_raw_type(&*t.ty));
            }
        }
    }

    HelperTokenStream::from_iter(params)
}

fn gen_raw_type(t: &Type) -> HelperTokenStream {
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
