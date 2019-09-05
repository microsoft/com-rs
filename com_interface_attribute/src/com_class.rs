use proc_macro::TokenStream;
type HelperTokenStream = proc_macro2::TokenStream;
use quote::{format_ident, quote,};
use syn:: {
    ItemStruct, Ident, Meta, NestedMeta,
};

use std::iter::FromIterator;
use crate::utils::{camel_to_snake, get_vptr_ident, get_vtable_ident,};

// Helper functions

fn get_vtable_macro_ident(trait_ident: &Ident) -> Ident {
    format_ident!(
        "{}_gen_vtable",
        camel_to_snake(trait_ident.to_string())
    )
}

fn get_ref_count_ident() -> Ident {
    format_ident!("__refcnt")
}

fn get_vptr_field_ident(trait_ident: &Ident) -> Ident {
    format_ident!("__{}vptr", trait_ident.to_string().to_lowercase())
}

fn get_real_ident(struct_ident: &Ident) -> Ident {
    if !struct_ident.to_string().starts_with("Init") {
        panic!("The target struct's name must begin with Init")
    }

    format_ident!("{}", &struct_ident.to_string()[4..])
}

fn get_inner_init_field_ident() -> Ident {
    format_ident!("__init_struct")
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

// Macro expansion entry point.

pub fn expand_com_class(item: TokenStream) -> TokenStream {

    let input = syn::parse_macro_input!(item as ItemStruct);

    // Parse attributes
    let base_itf_idents = get_base_interface_idents(&input);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(gen_real_struct(&base_itf_idents, &input).into());
    out.push(gen_allocate_impl(&base_itf_idents, &input).into());
    out.push(gen_iunknown_impl(&base_itf_idents, &input).into());
    out.push(gen_drop_impl(&base_itf_idents, &input).into());
    out.push(gen_deref_impl(&input).into());

    let out = TokenStream::from_iter(out);
    println!("Result:\n{}", out);
    out
}

fn gen_drop_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = get_real_ident(&struct_item.ident);
    let box_from_raws = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = get_vptr_field_ident(&base);
        let vtable_ident = get_vtable_ident(&base);
        quote!(
            Box::from_raw(self.#vptr_field_ident as *mut #vtable_ident);
        )
    });

    quote!(
        impl std::ops::Drop for #real_ident {
            fn drop(&mut self) {
                let _ = unsafe {
                    #(#box_from_raws)*
                };
            }
        }
    )
}

fn gen_deref_impl(struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_ident = get_real_ident(init_ident);
    let inner_init_field_ident = get_inner_init_field_ident();

    quote!(
        impl std::ops::Deref for #real_ident {
            type Target = #init_ident;
            fn deref(&self) -> &Self::Target {
                &self.#inner_init_field_ident
            }
        }
    )
}

fn gen_iunknown_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = get_real_ident(&struct_item.ident);
    let ref_count_ident = get_ref_count_ident();

    let first_vptr_field = get_vptr_field_ident(&base_itf_idents[0]);

    let match_arms = base_itf_idents.iter().map(|base| {
        let match_condition = quote!(<dyn #base as com::ComInterface>::iid_in_inheritance_chain(riid));
        let vptr_field_ident = get_vptr_field_ident(&base);

        quote!(
            else if #match_condition {
                *ppv = &self.#vptr_field_ident as *const _ as *mut c_void;
            }
        )
    });

    quote!(
        impl com::IUnknown for #real_ident {
            fn query_interface(
                &mut self,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void
            ) -> winapi::shared::winerror::HRESULT {
                unsafe {
                    let riid = &*riid;

                    if IsEqualGUID(riid, &com::IID_IUNKNOWN) {
                        *ppv = &self.#first_vptr_field as *const _ as *mut c_void;
                    } #(#match_arms)* else {
                        *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                        println!("Returning NO INTERFACE.");
                        return E_NOINTERFACE;
                    }

                    println!("Successful!.");
                    self.add_ref();
                    NOERROR
                }
            }

            fn add_ref(&mut self) -> u32 {
                self.#ref_count_ident += 1;
                println!("Count now {}", self.#ref_count_ident);
                self.#ref_count_ident
            }

            fn release(&mut self) -> u32 {
                self.#ref_count_ident -= 1;
                println!("Count now {}", self.#ref_count_ident);
                let count = self.#ref_count_ident;
                if count == 0 {
                    println!("Count is 0 for BritishShortHairCat. Freeing memory...");
                    drop(self)
                }
                count
            }
        }
    )
    // unimplemented!()
}

fn gen_allocate_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_ident = get_real_ident(&struct_item.ident);

    let mut offset_count : usize = 0;
    let base_inits = base_itf_idents.iter().map(|base| {
        let vtable_var_ident = format_ident!("{}_vtable", base.to_string().to_lowercase());
        let vtable_macro_ident = get_vtable_macro_ident(&base);
        let vptr_field_ident = get_vptr_field_ident(&base);
        

        let out = quote!(
            let #vtable_var_ident = #vtable_macro_ident!(#real_ident, #offset_count);
            let #vptr_field_ident = Box::into_raw(Box::new(#vtable_var_ident));
        );

        offset_count += 1;
        out
    });
    let base_fields = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = get_vptr_field_ident(base);
        quote!(#vptr_field_ident)
    });
    let ref_count_ident = get_ref_count_ident();
    let inner_init_field_ident = get_inner_init_field_ident();

    quote!(
        impl #real_ident {
            fn allocate(init_struct: #init_ident) -> Box<#real_ident> {
                println!("Allocating new VTable for {}", stringify!(#real_ident));
                #(#base_inits)*
                let out = #real_ident {
                    #(#base_fields,)*
                    #ref_count_ident: 0,
                    #inner_init_field_ident: init_struct
                };
                Box::new(out)
            }
        }
    )
}

fn gen_real_struct(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let init_ident = &struct_item.ident;
    let real_ident = get_real_ident(&struct_item.ident);
    let vis = &struct_item.vis;

    let bases_itf_idents = base_itf_idents.iter().map(|base| {
        let field_ident = get_vptr_field_ident(&base);
        let vptr_ident = get_vptr_ident(&base);
        quote!(#field_ident: #vptr_ident)
    });

    let ref_count_ident = get_ref_count_ident();
    let inner_init_field_ident = get_inner_init_field_ident();

    quote!(
        #[repr(C)]
        #vis struct #real_ident {
            #(#bases_itf_idents,)*
            #ref_count_ident: u32,
            #inner_init_field_ident: #init_ident
        }
    )
}