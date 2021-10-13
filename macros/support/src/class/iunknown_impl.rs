use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::class::Interface;

pub struct IUnknownAbi {
    class_name: Ident,
    offset: usize,
}

impl IUnknownAbi {
    pub fn new(class_name: Ident, offset: usize) -> Self {
        Self { class_name, offset }
    }

    pub fn to_add_ref_tokens(&self) -> TokenStream {
        let this_ptr = this_ptr_type();
        let munge = self.borrowed_pointer_munging();

        quote! {
            unsafe extern "system" fn AddRef(this: #this_ptr) -> u32 {
                #munge
                munged.AddRef()
            }
        }
    }

    pub fn to_release_tokens(&self) -> TokenStream {
        let this_ptr = this_ptr_type();
        let munge = self.borrowed_pointer_munging();
        let ref_count_ident = crate::utils::ref_count_ident();

        quote! {
            unsafe extern "system" fn Release(this: #this_ptr) -> u32 {
                #munge
                let new_ref_count = ::com::refcounting::release(&munged.#ref_count_ident);
                if new_ref_count == 0 {
                    // The last reference has been dropped.
                    munged.drop_inner();
                }
                new_ref_count
            }
        }
    }

    pub fn to_query_interface_tokens(&self) -> TokenStream {
        let this_ptr = this_ptr_type();
        let munge = self.borrowed_pointer_munging();

        quote! {
            unsafe extern "system" fn QueryInterface(
                this: #this_ptr,
                riid: *const ::com::sys::IID,
                ppv: *mut *mut ::core::ffi::c_void
            ) -> ::com::sys::HRESULT {
                #munge
                munged.QueryInterface(riid, ppv)
            }
        }
    }

    fn owned_pointer_munging(&self) -> TokenStream {
        let offset = self.offset;
        let class_name = &self.class_name;

        quote! {
            let munged = this.as_ptr().sub(#offset);
            let munged = ::com::production::ClassAllocation::from_raw(munged as *mut _ as *mut #class_name);
        }
    }

    fn borrowed_pointer_munging(&self) -> TokenStream {
        let owned = self.owned_pointer_munging();

        quote! {
            #owned
            let mut munged = ::core::mem::ManuallyDrop::new(munged);
        }
    }
}

pub struct IUnknown;

impl IUnknown {
    pub fn new() -> Self {
        Self
    }

    pub fn to_add_ref_tokens(&self) -> TokenStream {
        let ref_count_ident = crate::utils::ref_count_ident();
        quote! {
            pub unsafe fn AddRef(self: &::core::pin::Pin<::com::alloc::boxed::Box<Self>>) -> u32 {
                ::com::refcounting::addref(&self.#ref_count_ident)
            }
        }
    }

    pub fn to_query_interface_tokens(&self, interfaces: &[Interface]) -> TokenStream {
        // Generate match arms for implemented interfaces
        let base_match_arms = Self::gen_base_match_arms(interfaces);

        quote! {
            // We don't want this inlined into every interface chain. QueryInterface
            // can generate a lot of code.
            #[inline(never)]
            pub unsafe fn QueryInterface(
                self: &::core::pin::Pin<::com::alloc::boxed::Box<Self>>,
                riid: *const ::com::sys::IID,
                ppv: *mut *mut ::core::ffi::c_void
            ) -> ::com::sys::HRESULT {
                let riid = &*riid;

                // Use 'pv' for definite assignment analysis, to guarantee
                // that we always assign *ppv.
                let pv: *const ::core::ffi::c_void =
                    #base_match_arms {
                        *ppv = ::core::ptr::null_mut::<::core::ffi::c_void>();
                        return ::com::sys::E_NOINTERFACE;
                    };

                *ppv = pv as *mut ::core::ffi::c_void;

                self.AddRef();
                ::com::sys::NOERROR
            }
        }
    }

    fn gen_base_match_arms(interfaces: &[Interface]) -> TokenStream {
        // Generate match arms for implemented interfaces
        interfaces.iter().enumerate().map(|(index, interface)| {
            let interface_path = &interface.path;
            let interface_field_ident = interface.chain_ident(index);
            let or_iunknown_clause =
                if index == 0 {
                    // If we're querying for IUnknown, any interface chain will do.
                    // So we always pick the first one.
                    quote!(riid == &::com::interfaces::iunknown::IID_IUNKNOWN ||)
                } else {
                    quote!()
                };
            quote! {
                if #or_iunknown_clause <#interface_path as ::com::Interface>::is_iid_in_inheritance_chain(riid) {
                    // Cast the &Pin<Box<T>> as a pointer and then dereference
                    // it to get the Pin<Box> as a pointer
                    &self.#interface_field_ident as *const _ as *const ::core::ffi::c_void
                } else
            }
        }).collect()
    }
}

fn this_ptr_type() -> TokenStream {
    quote! {
        ::core::ptr::NonNull<::core::ptr::NonNull<<::com::interfaces::IUnknown as ::com::Interface>::VTable>>
    }
}
