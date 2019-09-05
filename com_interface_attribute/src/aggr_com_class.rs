use proc_macro::TokenStream;
type HelperTokenStream = proc_macro2::TokenStream;
use quote::{format_ident, quote,};
use syn:: {
    ItemStruct, Ident, Meta, NestedMeta,
};

use std::iter::FromIterator;
use crate::utils::{camel_to_snake, get_vptr_ident, get_vtable_ident,};

// Helper functions

fn get_non_del_unk_field_ident() -> Ident {
    format_ident!("__non_delegating_unk")
}

fn get_iunk_to_use_field_ident() -> Ident {
    format_ident!("__iunk_to_use")
}

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

pub fn expand_aggregable_com_class(item: TokenStream) -> TokenStream {

    let input = syn::parse_macro_input!(item as ItemStruct);

    // Parse attributes
    let base_itf_idents = get_base_interface_idents(&input);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(gen_real_struct(&base_itf_idents, &input).into());
    out.push(gen_impl(&base_itf_idents, &input).into());
    out.push(gen_iunknown_impl(&input).into());
    out.push(gen_drop_impl(&base_itf_idents, &input).into());
    out.push(gen_deref_impl(&input).into());

    let out = TokenStream::from_iter(out);
    println!("Result:\n{}", out);
    out
}

fn gen_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {

    let real_ident = get_real_ident(&struct_item.ident);
    let allocate_fn = gen_allocate_fn(base_itf_idents, struct_item);
    let set_iunknown_fn = gen_set_iunknown_fn();
    let iunknown_fns = gen_iunknown_fns(base_itf_idents, struct_item);

    quote!(
        impl #real_ident {
            #allocate_fn
            #set_iunknown_fn
            #iunknown_fns
        }
    )
}

fn gen_set_iunknown_fn() -> HelperTokenStream {
    let iunk_to_use_field_ident = get_iunk_to_use_field_ident();
    let non_del_unk_field_ident = get_non_del_unk_field_ident();

    quote!(
        pub(crate) fn set_iunknown(&mut self, aggr: *mut IUnknownVPtr) {
            if aggr.is_null() {
                self.#iunk_to_use_field_ident = &self.#non_del_unk_field_ident as *const _ as *mut IUnknownVPtr;
            } else {
                self.#iunk_to_use_field_ident = aggr;
            }
        }
    )
}

fn gen_iunknown_fns(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = get_real_ident(&struct_item.ident);
    let ref_count_ident = get_ref_count_ident();
    let non_del_unk_field_ident = get_non_del_unk_field_ident();
    
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
        pub(crate) fn inner_query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
            println!("Non delegating QI");

            unsafe {
                let riid = &*riid;

                if IsEqualGUID(riid, &com::IID_IUNKNOWN) {
                    *ppv = &self.#non_del_unk_field_ident as *const _ as *mut c_void;
                } #(#match_arms)* else {
                    *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                    println!("Returning NO INTERFACE.");
                    return E_NOINTERFACE;
                }

                println!("Successful!.");
                self.inner_add_ref();
                NOERROR
            }
        }

        pub(crate) fn inner_add_ref(&mut self) -> u32 {
            self.#ref_count_ident += 1;
            println!("Count now {}", self.#ref_count_ident);
            self.#ref_count_ident
        }

        pub(crate) fn inner_release(&mut self) -> u32 {
            self.#ref_count_ident -= 1;
            println!("Count now {}", self.#ref_count_ident);
            let count = self.#ref_count_ident;
            if count == 0 {
                println!("Count is 0 for {}. Freeing memory...", stringify!(#real_ident));
                drop(self)
            }
            count
        }
    )
}

fn gen_drop_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = get_real_ident(&struct_item.ident);
    let non_del_unk_field_ident = get_non_del_unk_field_ident();
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
                    Box::from_raw(self.#non_del_unk_field_ident as *mut com::IUnknownVTable)
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

fn gen_iunknown_impl(struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = get_real_ident(&struct_item.ident);
    let iunk_to_use_field_ident = get_iunk_to_use_field_ident();

    quote!(
        impl com::IUnknown for #real_ident {
            fn query_interface(
                &mut self,
                riid: *const winapi::shared::guiddef::IID,
                ppv: *mut *mut winapi::ctypes::c_void
            ) -> winapi::shared::winerror::HRESULT {
                println!("Delegating QI");
                let mut iunk_to_use: com::ComPtr<dyn IUnknown> = unsafe { com::ComPtr::new(self.#iunk_to_use_field_ident as *mut c_void) };
                let hr = iunk_to_use.query_interface(riid, ppv);
                forget(iunk_to_use);

                hr
            }

            fn add_ref(&mut self) -> u32 {
                let mut iunk_to_use: com::ComPtr<dyn IUnknown> = unsafe { com::ComPtr::new(self.#iunk_to_use_field_ident as *mut c_void) };
                let res = iunk_to_use.add_ref();
                forget(iunk_to_use);

                res
            }

            fn release(&mut self) -> u32 {
                let mut iunk_to_use: com::ComPtr<dyn IUnknown> = unsafe { com::ComPtr::new(self.#iunk_to_use_field_ident as *mut c_void) };
                let res = iunk_to_use.release();
                forget(iunk_to_use);

                res
            }
        }
    )
}

fn gen_allocate_fn(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
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
    let iunk_to_use_field_ident = get_iunk_to_use_field_ident();
    let non_del_unk_field_ident = get_non_del_unk_field_ident();
    let non_del_unk_offset = base_itf_idents.len();

    quote!(
        fn allocate(init_struct: #init_ident) -> Box<#real_ident> {
            println!("Allocating new VTable for {}", stringify!(#real_ident));

            // Non-delegating methods.
            unsafe extern "stdcall" fn non_delegating_query_interface(
                this: *mut IUnknownVPtr,
                riid: *const IID,
                ppv: *mut *mut c_void,
            ) -> HRESULT {
                let this = this.sub(#non_del_unk_offset) as *mut #real_ident;
                (*this).inner_query_interface(riid, ppv)
            }

            unsafe extern "stdcall" fn non_delegating_add_ref(
                this: *mut IUnknownVPtr,
            ) -> u32 {
                let this = this.sub(#non_del_unk_offset) as *mut #real_ident;
                (*this).inner_add_ref()
            }

            unsafe extern "stdcall" fn non_delegating_release(
                this: *mut IUnknownVPtr,
            ) -> u32 {
                let this = this.sub(#non_del_unk_offset) as *mut #real_ident;
                (*this).inner_release()
            }

            let __non_del_unk_vtable = IUnknownVTable {
                QueryInterface: non_delegating_query_interface,
                Release: non_delegating_release,
                AddRef: non_delegating_add_ref,
            };
            let #non_del_unk_field_ident = Box::into_raw(Box::new(__non_del_unk_vtable));

            #(#base_inits)*
            let out = #real_ident {
                #(#base_fields,)*
                #non_del_unk_field_ident,
                #iunk_to_use_field_ident: std::ptr::null_mut::<IUnknownVPtr>(),
                #ref_count_ident: 0,
                #inner_init_field_ident: init_struct
            };
            Box::new(out)
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
    let non_del_unk_field_ident = get_non_del_unk_field_ident();
    let iunk_to_use_field_ident = get_iunk_to_use_field_ident();

    quote!(
        #[repr(C)]
        #vis struct #real_ident {
            #(#bases_itf_idents,)*
            #non_del_unk_field_ident: IUnknownVPtr,
            #iunk_to_use_field_ident: *mut IUnknownVPtr,
            #ref_count_ident: u32,
            #inner_init_field_ident: #init_ident
        }
    )
}