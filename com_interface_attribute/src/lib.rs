extern crate proc_macro;

mod utils;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as HelperTokenStream};

use quote::{format_ident, quote, ToTokens};
use syn::{
    FnArg, ItemStruct, ItemTrait, LitInt, TraitItem, TraitItemMethod, Type, TypeBareFn,
    TypeParamBound,
};

use std::iter::FromIterator;

// Macro entry point.

#[proc_macro_attribute]
pub fn com_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemTrait);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(input.to_token_stream().into());
    out.push(gen_vtable(&input).into());
    out.push(gen_vptr_type(&input).into());
    out.push(gen_comptr_impl(&input).into());
    out.push(gen_cominterface_impl(&input).into());
    out.push(gen_iid_struct(&attr, &input.ident).into());

    TokenStream::from_iter(out)
}

#[proc_macro_derive(VTableMacro)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemStruct);
    gen_vtable_macro(&input).into()
}

// Helper functions

fn get_iid_ident(trait_ident: &Ident) -> Ident {
    format_ident!(
        "IID_{}",
        utils::camel_to_snake(trait_ident.to_string()).to_uppercase()
    )
}

fn get_vtable_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VTable", trait_ident)
}

fn get_vptr_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VPtr", trait_ident)
}

fn get_vtable_macro_ident(struct_ident: &Ident) -> Ident {
    format_ident!(
        "{}_gen_vtable",
        struct_ident
            .to_string()
            .replace("VTable", "")
            .to_lowercase()
    )
}

// pub const IID_IUNKNOWN: GUID = GUID {

fn gen_iid_struct(attr: &TokenStream, trait_ident: &Ident) -> HelperTokenStream {
    let iid_value = attr.to_string().replace(' ', "");
    assert!(iid_value.len() == 36);

    let iid_ident = get_iid_ident(trait_ident);
    let iid_value = iid_value.as_str();
    let delimited: Vec<&str> = iid_value.split('-').collect();
    assert!(delimited.len() == 5);

    assert!(delimited[0].len() == 8);
    let data1 = LitInt::new(format!("0x{}", delimited[0]).as_str(), Span::call_site());

    assert!(delimited[1].len() == 4);
    let data2 = LitInt::new(format!("0x{}", delimited[1]).as_str(), Span::call_site());

    assert!(delimited[2].len() == 4);
    let data3 = LitInt::new(format!("0x{}", delimited[2]).as_str(), Span::call_site());

    assert!(delimited[3].len() == 4);
    let (data4_1, data4_2) = delimited[3].split_at(2);
    let data4_1 = LitInt::new(format!("0x{}", data4_1).as_str(), Span::call_site());
    let data4_2 = LitInt::new(format!("0x{}", data4_2).as_str(), Span::call_site());

    assert!(delimited[4].len() == 12);
    let (data4_3, rest) = delimited[4].split_at(2);
    let data4_3 = LitInt::new(format!("0x{}", data4_3).as_str(), Span::call_site());

    let (data4_4, rest) = rest.split_at(2);
    let data4_4 = LitInt::new(format!("0x{}", data4_4).as_str(), Span::call_site());

    let (data4_5, rest) = rest.split_at(2);
    let data4_5 = LitInt::new(format!("0x{}", data4_5).as_str(), Span::call_site());

    let (data4_6, rest) = rest.split_at(2);
    let data4_6 = LitInt::new(format!("0x{}", data4_6).as_str(), Span::call_site());

    let (data4_7, data4_8) = rest.split_at(2);
    let data4_7 = LitInt::new(format!("0x{}", data4_7).as_str(), Span::call_site());
    let data4_8 = LitInt::new(format!("0x{}", data4_8).as_str(), Span::call_site());

    quote!(
        #[allow(non_upper_case_globals)]
        pub const #iid_ident: com::_winapi::shared::guiddef::GUID = com::_winapi::shared::guiddef::GUID {
            Data1: #data1,
            Data2: #data2,
            Data3: #data3,
            Data4: [#data4_1, #data4_2, #data4_3, #data4_4, #data4_5, #data4_6, #data4_7, #data4_8]
        };
    )
}

// unsafe impl ComInterface for IUnknown

fn gen_cominterface_impl(itf: &ItemTrait) -> HelperTokenStream {
    let trait_ident = &itf.ident;
    let vtable_ident = get_vtable_ident(trait_ident);
    let iid_ident = get_iid_ident(trait_ident);

    quote!(
        unsafe impl com::ComInterface for dyn #trait_ident {
            type VTable = #vtable_ident;
            const IID: com::_winapi::shared::guiddef::IID = #iid_ident;
        }
    )
}

// pub type IUnknownVPtr = *const IUnknownVTable;

fn gen_vptr_type(itf: &ItemTrait) -> HelperTokenStream {
    let vptr_ident = get_vptr_ident(&itf.ident);
    let vtable_ident = get_vtable_ident(&itf.ident);

    quote!(
        pub type #vptr_ident = *const #vtable_ident;
    )
}

fn gen_vtable_macro(item: &ItemStruct) -> HelperTokenStream {
    let vtable_macro = get_vtable_macro_ident(&item.ident);
    let parent_vtable_macro = gen_parent_vtable_macro(item);
    let vtable_functions = gen_vtable_functions(item);
    let initialized_vtable = gen_initialized_vtable(item);
    quote! {
        #[macro_export]
        macro_rules! #vtable_macro {
            ($type:ty, $offset:literal) => {{
                #parent_vtable_macro
                #vtable_functions
                #initialized_vtable
            }};
        }
    }
}

fn gen_parent_vtable_macro(item: &ItemStruct) -> HelperTokenStream {
    let parent = item.fields.iter().nth(0).and_then(|f| f.ident.as_ref());
    if let Some(parent_name) = parent {
        let parent_name = parent_name.to_string();
        if parent_name.ends_with("_base") {
            let vtable_gen = format_ident!("{}_gen_vtable", parent_name.trim_end_matches("_base"));
            return quote! {
                let parent_vtable = #vtable_gen!($type, $offset);
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
    let method_name = format_ident!("{}", utils::camel_to_snake(method_name.to_string()));
    let function_ident = format_ident!(
        "{}_{}",
        struct_ident
            .to_string()
            .replace("VTable", "")
            .to_lowercase(),
        method_name
    );
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
        unsafe extern "stdcall" fn #function_ident(#(#params)*) #return_type {
            let this = arg0.sub($offset) as *mut $type;
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
            format_ident!("parent_vtable",)
        } else {
            format_ident!(
                "{}_{}",
                item.ident.to_string().replace("VTable", "").to_lowercase(),
                utils::camel_to_snake(method_ident.to_string())
            )
        };
        let method = quote! {
            #method_ident: #function_ident,
        };
        methods.push(method);
    }

    quote!(
        #(#methods)*
    )
}

// pub struct IUnknownVTable.

fn gen_vtable(itf: &ItemTrait) -> HelperTokenStream {
    let trait_ident = &itf.ident;
    let vtable_ident = get_vtable_ident(&itf.ident);

    let methods = gen_vtable_methods(&itf);

    if trait_ident.to_string().to_uppercase() == "IUNKNOWN" {
        assert!(
            itf.supertraits.len() == 0,
            "Interfaces must have some other interface as their parent"
        );

        quote!(
            #[allow(non_snake_case)]
            #[repr(C)]
            #[derive(com::VTableMacro)]
            pub struct #vtable_ident {
                #methods
            }
        )
    } else {
        assert!(
            itf.supertraits.len() == 1,
            "Multiple inheirtance is not supported in COM interfaces"
        );

        let base_trait_ident = match itf.supertraits.first().unwrap() {
            TypeParamBound::Trait(t) => t.path.get_ident().unwrap(),
            _ => panic!("Unhandled super trait typeparambound"),
        };

        let base_field_ident = format_ident!(
            "{}_base",
            utils::camel_to_snake(base_trait_ident.to_string())
        );

        quote!(
            #[allow(non_snake_case)]
            #[repr(C)]
            #[derive(com::VTableMacro)]
            pub struct #vtable_ident {
                pub #base_field_ident: <dyn #base_trait_ident as com::ComInterface>::VTable,
                #methods
            }
        )
    }
}

fn gen_vtable_methods(itf: &ItemTrait) -> HelperTokenStream {
    let mut methods: Vec<HelperTokenStream> = Vec::new();
    for trait_item in &itf.items {
        match trait_item {
            TraitItem::Method(n) => methods.push(gen_vtable_method(&itf.ident, n)),
            _ => println!("Unhandled trait item in gen_vtable_methods"),
        };
    }

    quote!(
        #(#methods)*
    )
}

fn gen_vtable_method(trait_ident: &Ident, method: &TraitItemMethod) -> HelperTokenStream {
    let method_ident = format_ident!("{}", utils::snake_to_camel(method.sig.ident.to_string()));
    let vtable_function_signature = gen_vtable_function_signature(trait_ident, None, method);

    quote!(
        pub #method_ident: #vtable_function_signature,
    )
}

fn gen_vtable_function_signature(
    trait_ident: &Ident,
    name: Option<&Ident>,
    method: &TraitItemMethod,
) -> HelperTokenStream {
    let name = match name {
        Some(n) => quote! { #n },
        None => quote! {},
    };
    let params = gen_raw_params(trait_ident, method);
    let return_type = &method.sig.output;

    quote!(
        unsafe extern "stdcall" fn #name(#params) #return_type
    )
}

fn gen_raw_params(trait_ident: &Ident, method: &TraitItemMethod) -> HelperTokenStream {
    let mut params = Vec::new();
    let vptr_ident = get_vptr_ident(trait_ident);
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

fn gen_comptr_impl(itf: &ItemTrait) -> HelperTokenStream {
    let trait_ident = &itf.ident;
    let mut impl_methods: Vec<HelperTokenStream> = Vec::new();

    for trait_item in &itf.items {
        match trait_item {
            TraitItem::Method(n) => {
                impl_methods.push(gen_comptr_impl_method(&itf.ident, n));
            }
            _ => println!("Ignored trait item for comptr_impl"),
        }
    }

    quote!(
        impl <T: #trait_ident + com::ComInterface + ?Sized> #trait_ident for com::ComPtr<T> {
            #(#impl_methods)*
        }
    )
}

fn gen_comptr_impl_method(trait_ident: &Ident, method: &TraitItemMethod) -> HelperTokenStream {
    let method_sig = &method.sig;
    let vptr_ident = get_vptr_ident(trait_ident);
    let method_ident = format_ident!("{}", utils::snake_to_camel(method.sig.ident.to_string()));
    let itf_ptr_ident = format_ident!("itf_ptr");

    let mut params = Vec::new();
    for param in method.sig.inputs.iter() {
        match param {
            FnArg::Receiver(_n) => params.push(quote!(#itf_ptr_ident)),
            // TODO: This may go wrong, I am using everything on the LHS.
            FnArg::Typed(n) => params.push(n.pat.to_token_stream()),
        }
    }

    quote!(
        #method_sig {
            let #itf_ptr_ident = self.into_raw() as *mut #vptr_ident;
            unsafe { ((**#itf_ptr_ident).#method_ident)(#(#params),*) }
        }
    )
}
