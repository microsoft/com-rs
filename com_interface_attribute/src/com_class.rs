use proc_macro::TokenStream;
type HelperTokenStream = proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn:: {
    ItemStruct, Ident, Meta, Attribute, NestedMeta, Fields
};

use std::iter::FromIterator;

// REPEATED FUNCTIONS TO MOVE TO UTILS AFTER PULL REQUEST!
fn get_vptr_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VPtr", trait_ident)
}

fn get_vtable_macro_ident(trait_ident: &Ident) -> Ident {
    format_ident!(
        "{}_gen_vtable",
        camel_to_snake(trait_ident.to_string())
    )
}

pub fn camel_to_snake(input: String) -> String {
    let mut new = String::new();
    let mut seen_lowercase = false;

    for c in input.chars() {
        if c.is_uppercase() {
            if seen_lowercase {
                seen_lowercase = false;
                new.push_str("_");
            }
            new.push_str(&c.to_lowercase().to_string());
        } else {
            seen_lowercase = true;
            new.push_str(&c.to_string())
        }
    }

    new
}

// REPEATED END

fn get_ref_count_ident() -> Ident {
    format_ident!("__refcnt")
}

fn get_vptr_field_ident(trait_ident: &Ident) -> Ident {
    format_ident!("__{}vptr", trait_ident.to_string().to_lowercase())
}

pub fn expand_com_class(item: TokenStream) -> TokenStream {

    let input = syn::parse_macro_input!(item as ItemStruct);

    // Parse attributes
    let mut base_itf_idents = get_base_interface_idents(&input);
    for ident in &base_itf_idents {
        println!("Found base itf ident: {}", ident);
    }

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(gen_real_struct(&base_itf_idents, &input).into());
    out.push(gen_allocate_impl(&base_itf_idents, &input).into());

    let out = TokenStream::from_iter(out);
    println!("Result:\n{}", out);
    out
}

fn gen_allocate_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_name = get_real_name(&struct_item.ident);

    let mut offset_count = -1;
    let base_inits = base_itf_idents.iter().map(|base| {
        let vtable_var_ident = format_ident!("{}_vtable", base.to_string().to_lowercase());
        let vtable_macro_ident = get_vtable_macro_ident(&base);
        let vptr_field_ident = get_vptr_field_ident(&base);
        offset_count += 1;

        quote!(
            let #vtable_var_ident = #vtable_macro_ident!(#real_name, #offset_count);
            let #vptr_field_ident = Box::into_raw(Box::new(#vtable_var_ident));
        )
    });
    let base_fields = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = get_vptr_field_ident(base);
        quote!(#vptr_field_ident)
    });
    let ref_count_ident = get_ref_count_ident();

    quote!(
        impl #real_name {
            fn allocate(value: #init_ident) -> Box<#real_name> {
                println!("Allocating new VTable for {}", stringify!(#real_name));
                #(#base_inits)*
                let out = #real_name {
                    #(#base_fields,)*
                    #ref_count_ident: 0,
                    value,
                };
                Box::new(out);
            }
        }
    )
}

fn get_real_name(struct_ident: &Ident) -> Ident {
    if !struct_ident.to_string().starts_with("Init") {
        panic!("The target struct's name must begin with Init")
    }

    format_ident!("{}", &struct_ident.to_string()[4..])
}

fn get_base_interface_idents(struct_item: &ItemStruct) -> Vec<Ident> {
    let mut base_itf_idents = Vec::new();

    for attr in &struct_item.attrs {
        if let Ok(Meta::List(ref attr)) = attr.parse_meta() {
            if attr.path.segments.last().unwrap().ident != "com_implements" {
                continue;
            }

            for item in &attr.nested {
                if let NestedMeta::Meta(Meta::Path(p)) = item {
                    assert!(p.segments.len() == 1, "Incapable of handling multiple path segments yet.");
                    base_itf_idents.push(p.segments.last().unwrap().ident.clone());
                }
            }
        }
    }

    base_itf_idents
}

fn gen_real_struct(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_name = get_real_name(&struct_item.ident);
    let vis = &struct_item.vis;
    let fields = match &struct_item.fields {
        Fields::Named(n) => &n.named,
        _ => panic!("Encountered non-name field in struct declaration")
    };

    let bases_itf_idents = base_itf_idents.iter().map(|base| {
        let field_ident = get_vptr_field_ident(&base);
        let vptr_ident = get_vptr_ident(&base);
        quote!(#field_ident: #vptr_ident)
    });

    let ref_count_ident = get_ref_count_ident();

    quote!(
        #[repr(C)]
        #vis struct #real_name {
            #(#bases_itf_idents,)*
            #ref_count_ident: u32,
            value: #init_ident
        }
    )
}