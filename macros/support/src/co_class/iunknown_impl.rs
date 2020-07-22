use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub struct IUnknown {
    class_name: Ident,
    offset: usize,
}

impl IUnknown {
    pub fn new(class_name: Ident, offset: usize) -> Self {
        Self { class_name, offset }
    }

    pub fn to_add_ref_stdcall_tokens(&self) -> TokenStream {
        let this_ptr = Self::this_ptr_type();
        let munge = self.pointer_munging();
        let class_name = &self.class_name;

        quote! {
            unsafe extern "stdcall" fn add_ref(this_ptr: #this_ptr) -> u32 {
                #munge
                #class_name::add_ref(this)
            }
        }
    }

    pub fn to_add_ref_tokens(&self) -> TokenStream {
        let ref_count_ident = crate::utils::ref_count_ident();
        quote! {
            unsafe fn add_ref(&self) -> u32 {
                let value = self.#ref_count_ident.get().checked_add(1).expect("Overflow of reference count");
                self.#ref_count_ident.set(value);
                value
            }
        }
    }

    pub fn to_release_stdcall_tokens(&self) -> TokenStream {
        let this_ptr = Self::this_ptr_type();
        let munge = self.pointer_munging();
        let class_name = &self.class_name;

        quote! {
            unsafe extern "stdcall" fn release(this_ptr: #this_ptr) -> u32 {
                #munge
                #class_name::release(this)
            }
        }
    }

    pub fn to_release_tokens(&self, interface_idents: &[&syn::Path]) -> TokenStream {
        let offset = self.offset;
        let class_name = &self.class_name;
        let ref_count_ident = crate::utils::ref_count_ident();

        let vptr_drops = interface_idents.iter().enumerate().map(|(index, _)| {
            let vptr_field_ident = quote::format_ident!("__{}", index);
            quote! {
                Box::from_raw(this.#vptr_field_ident.as_ptr());
            }
        });

        let this_ptr = Self::this_ptr_type();

        quote! {
            unsafe fn release(&self) -> u32 {
                let value = self.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count");
                self.#ref_count_ident.set(value);
                let #ref_count_ident = self.#ref_count_ident.get();
                if #ref_count_ident == 0 {
                    #(#vptr_drops)*
                    Box::from_raw(self as *const _ as *mut #class_name);
                }

                #ref_count_ident
            }
        }
    }

    pub fn to_query_interface_stdcall_tokens(
        &self,
        interface_idents: &[&syn::Path],
    ) -> TokenStream {
        let offset = self.offset;
        let class_name = &self.class_name;
        // Generate match arms for implemented interfaces
        let base_match_arms = Self::gen_base_match_arms(interface_idents);
        let this_ptr = Self::this_ptr_type();
        let munge = self.pointer_munging();

        quote! {
            unsafe extern "stdcall" fn query_interface(
                this_ptr: #this_ptr,
                riid: *const com::sys::IID,
                ppv: *mut *mut std::ffi::c_void
            ) -> com::sys::HRESULT {
                #munge
                #class_name::query_interface(this)
            }
        }
    }

    pub fn to_query_interface_tokens(&self, interface_idents: &[&syn::Path]) -> TokenStream {
        let offset = self.offset;
        let class_name = &self.class_name;
        // Generate match arms for implemented interfaces
        let base_match_arms = Self::gen_base_match_arms(interface_idents);
        let this_ptr = Self::this_ptr_type();

        quote! {
            unsafe fn query_interface(
                &self,
                riid: *const com::sys::IID,
                ppv: *mut *mut std::ffi::c_void
            ) -> com::sys::HRESULT {
                let riid = &*riid;

                if riid == &com::interfaces::iunknown::IID_IUNKNOWN {
                    *ppv = &self.__0 as *const _ as *mut std::ffi::c_void;
                } #base_match_arms else {
                    *ppv = std::ptr::null_mut::<std::ffi::c_void>();
                    return com::sys::E_NOINTERFACE;
                }

                self.add_ref();
                com::sys::NOERROR
            }
        }
    }

    pub fn gen_base_match_arms(interface_idents: &[&syn::Path]) -> TokenStream {
        // Generate match arms for implemented interfaces
        let base_match_arms = interface_idents
            .iter()
            .enumerate()
            .map(|(index, interface)| {
                let match_condition =
                    quote!(<#interface as com::ComInterface>::is_iid_in_inheritance_chain(riid));
                let vptr_field_ident = quote::format_ident!("__{}", index);

                quote!(
                    else if #match_condition {
                        *ppv = &self.#vptr_field_ident as *const _ as *mut std::ffi::c_void;
                    }
                )
            });

        quote!(#(#base_match_arms)*)
    }

    fn pointer_munging(&self) -> TokenStream {
        let offset = self.offset;
        let class_name = &self.class_name;

        quote! {
            let this_ptr = this_ptr.as_ptr().sub(#offset);
            let this = &(*(&this_ptr as *const _ as *const #class_name));
        }
    }

    fn this_ptr_type() -> TokenStream {
        quote! {
            ::std::ptr::NonNull<::std::ptr::NonNull<<::com::interfaces::IUnknown as ::com::ComInterface>::VTable>>
        }
    }
}
