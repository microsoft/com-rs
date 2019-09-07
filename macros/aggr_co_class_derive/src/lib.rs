extern crate proc_macro;
use proc_macro::TokenStream;
type HelperTokenStream = proc_macro2::TokenStream;
use quote::{format_ident, quote,};
use syn:: {
    ItemStruct, Ident, Meta, NestedMeta, Fields,
};

use std::iter::FromIterator;
use std::collections::HashMap;
use macro_utils::*;

// Helper functions
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

fn get_aggr_map(struct_item: &ItemStruct) -> HashMap<Ident, Vec<Ident>> {
    let mut aggr_map = HashMap::new();

    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found field other than named fields in struct")
    };

    for field in fields {
        for attr in &field.attrs {
            if let Ok(Meta::List(ref attr)) = attr.parse_meta() {
                if attr.path.segments.last().unwrap().ident != "aggr" {
                    continue;
                }

                let mut aggr_interfaces_idents = Vec::new();


                assert!(attr.nested.len() > 0, "Need to expose at least one interface from aggregated COM object.");

                for item in &attr.nested {
                    if let NestedMeta::Meta(Meta::Path(p)) = item {
                        assert!(p.segments.len() == 1, "Incapable of handling multiple path segments yet.");
                        aggr_interfaces_idents.push(p.segments.last().unwrap().ident.clone());
                    }
                }
                let ident = field.ident.as_ref().unwrap().clone();
                aggr_map.insert(ident, aggr_interfaces_idents);
            }
        }
    }

    aggr_map
}

// Macro expansion entry point.

pub fn expand_derive_aggr_com_class(item: TokenStream) -> TokenStream {

    let input = syn::parse_macro_input!(item as ItemStruct);

    // Parse attributes
    let base_itf_idents = get_base_interface_idents(&input);
    let aggr_itf_idents = get_aggr_map(&input);

    let mut out: Vec<TokenStream> = Vec::new();
    out.push(gen_real_struct(&base_itf_idents, &input).into());
    out.push(gen_impl(&base_itf_idents, &aggr_itf_idents, &input).into());
    out.push(gen_iunknown_impl(&input).into());
    out.push(gen_drop_impl(&base_itf_idents, &input).into());
    out.push(gen_deref_impl(&input).into());

    let out = TokenStream::from_iter(out);
    println!("Result:\n{}", out);
    out
}

fn gen_impl(base_itf_idents: &[Ident], aggr_itf_idents: &HashMap<Ident, Vec<Ident>>, struct_item: &ItemStruct) -> HelperTokenStream {

    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let allocate_fn = gen_allocate_fn(base_itf_idents, struct_item);
    let set_iunknown_fn = gen_set_iunknown_fn();
    let inner_iunknown_fns = gen_inner_iunknown_fns(base_itf_idents, aggr_itf_idents, struct_item);

    quote!(
        impl #real_ident {
            #allocate_fn
            #set_iunknown_fn
            #inner_iunknown_fns
        }
    )
}

fn gen_set_iunknown_fn() -> HelperTokenStream {
    let iunk_to_use_field_ident = macro_utils::get_iunk_to_use_field_ident();
    let non_del_unk_field_ident = macro_utils::get_non_del_unk_field_ident();

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

fn gen_inner_iunknown_fns(base_itf_idents: &[Ident], aggr_itf_idents: &HashMap<Ident, Vec<Ident>>, struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let ref_count_ident = macro_utils::get_ref_count_ident();
    let inner_query_interface = gen_inner_query_interface(base_itf_idents, aggr_itf_idents, struct_item);

    quote!(
        #inner_query_interface

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
                // drop(self)
                unsafe { Box::from_raw(self as *const _ as *mut #real_ident); }
            }
            count
        }
    )
}

fn gen_inner_query_interface(base_itf_idents: &[Ident], aggr_itf_idents: &HashMap<Ident, Vec<Ident>>, struct_item: &ItemStruct) -> HelperTokenStream {
    let non_del_unk_field_ident = macro_utils::get_non_del_unk_field_ident();

    // Generate match arms for implemented interfaces
    let match_arms = base_itf_idents.iter().map(|base| {
        let match_condition = quote!(<dyn #base as com::ComInterface>::iid_in_inheritance_chain(riid));
        let vptr_field_ident = macro_utils::get_vptr_field_ident(&base);

        quote!(
            else if #match_condition {
                *ppv = &self.#vptr_field_ident as *const _ as *mut c_void;
            }
        )
    });

    // Generate match arms for aggregated interfaces
    let aggr_match_arms = aggr_itf_idents.iter().map(|(aggr_field_ident, aggr_base_itf_idents)| {

        // Construct the OR match conditions for a single aggregated object.
        let first_base_itf_ident = &aggr_base_itf_idents[0];
        let first_aggr_match_condition = quote!(
            <dyn #first_base_itf_ident as com::ComInterface>::iid_in_inheritance_chain(riid)
        );
        let rem_aggr_match_conditions = aggr_base_itf_idents.iter().skip(1).map(|base| {
            quote!(|| <dyn #base as com::ComInterface>::iid_in_inheritance_chain(riid))
        });

        quote!(
            else if #first_aggr_match_condition #(#rem_aggr_match_conditions)* {
                let mut aggr_itf_ptr: ComPtr<dyn IUnknown> = ComPtr::new(self.#aggr_field_ident as *mut c_void);
                let hr = aggr_itf_ptr.query_interface(riid, ppv);
                if com::failed(hr) {
                    return winapi::shared::winerror::E_NOINTERFACE;
                }

                // We release it as the previous call add_ref-ed the inner object.
                // The intention is to transfer reference counting logic to the
                // outer object.
                aggr_itf_ptr.release();

                forget(aggr_itf_ptr);
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
                } #(#match_arms)* #(#aggr_match_arms)* else {
                    *ppv = std::ptr::null_mut::<winapi::ctypes::c_void>();
                    println!("Returning NO INTERFACE.");
                    return E_NOINTERFACE;
                }

                println!("Successful!.");
                self.inner_add_ref();
                NOERROR
            }
        }
    )
}

fn gen_drop_impl(base_itf_idents: &[Ident], struct_item: &ItemStruct) -> HelperTokenStream {
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let non_del_unk_field_ident = macro_utils::get_non_del_unk_field_ident();
    let box_from_raws = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = macro_utils::get_vptr_field_ident(&base);
        let vtable_ident = macro_utils::get_vtable_ident(&base);
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
    let real_ident = macro_utils::get_real_ident(init_ident);
    let inner_init_field_ident = macro_utils::get_inner_init_field_ident();

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
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let iunk_to_use_field_ident = macro_utils::get_iunk_to_use_field_ident();

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
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);

    let mut offset_count : usize = 0;
    let base_inits = base_itf_idents.iter().map(|base| {
        let vtable_var_ident = format_ident!("{}_vtable", base.to_string().to_lowercase());
        let vptr_field_ident = macro_utils::get_vptr_field_ident(&base);
        

        let out = quote!(
            let #vtable_var_ident = com::vtable!(#real_ident: #base, #offset_count);
            let #vptr_field_ident = Box::into_raw(Box::new(#vtable_var_ident));
        );

        offset_count += 1;
        out
    });
    let base_fields = base_itf_idents.iter().map(|base| {
        let vptr_field_ident = macro_utils::get_vptr_field_ident(base);
        quote!(#vptr_field_ident)
    });
    let ref_count_ident = macro_utils::get_ref_count_ident();
    let inner_init_field_ident = macro_utils::get_inner_init_field_ident();
    let iunk_to_use_field_ident = macro_utils::get_iunk_to_use_field_ident();
    let non_del_unk_field_ident = macro_utils::get_non_del_unk_field_ident();
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
    let real_ident = macro_utils::get_real_ident(&struct_item.ident);
    let vis = &struct_item.vis;

    let bases_itf_idents = base_itf_idents.iter().map(|base| {
        let field_ident = macro_utils::get_vptr_field_ident(&base);
        let vptr_ident = macro_utils::get_vptr_ident(&base);
        quote!(#field_ident: #vptr_ident)
    });

    let ref_count_ident = macro_utils::get_ref_count_ident();
    let inner_init_field_ident = macro_utils::get_inner_init_field_ident();
    let non_del_unk_field_ident = macro_utils::get_non_del_unk_field_ident();
    let iunk_to_use_field_ident = macro_utils::get_iunk_to_use_field_ident();

    quote!(
        #[repr(C)]
        #vis struct #real_ident {
            #(#bases_itf_idents,)*
            #non_del_unk_field_ident: IUnknownVPtr,
            // Non-reference counted interface pointer to outer IUnknown.
            #iunk_to_use_field_ident: *mut IUnknownVPtr,
            #ref_count_ident: u32,
            #inner_init_field_ident: #init_ident
        }
    )
}