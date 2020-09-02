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
            unsafe extern "stdcall" fn add_ref(this: #this_ptr) -> u32 {
                #munge
                munged.add_ref()
            }
        }
    }

    pub fn to_release_tokens(&self) -> TokenStream {
        let this_ptr = this_ptr_type();
        let munge = self.owned_pointer_munging();
        let ref_count_ident = crate::utils::ref_count_ident();

        quote! {
            unsafe extern "stdcall" fn release(this: #this_ptr) -> u32 {
                #munge
                munged.#ref_count_ident.get().checked_sub(1).expect("Underflow of reference count")
            }
        }
    }

    pub fn to_query_interface_tokens(&self) -> TokenStream {
        let this_ptr = this_ptr_type();
        let munge = self.borrowed_pointer_munging();

        quote! {
            unsafe extern "stdcall" fn query_interface(
                this: #this_ptr,
                riid: *const ::com::sys::IID,
                ppv: *mut *mut ::std::ffi::c_void
            ) -> ::com::sys::HRESULT {
                #munge
                munged.query_interface(riid, ppv)
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
            let munged = ::std::mem::ManuallyDrop::new(munged);
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
            pub unsafe fn add_ref(self: &::std::pin::Pin<::std::boxed::Box<Self>>) -> u32 {
                let value = self.#ref_count_ident.get().checked_add(1).expect("Overflow of reference count");
                self.#ref_count_ident.set(value);
                value
            }
        }
    }

    pub fn to_query_interface_tokens(&self, interfaces: &[Interface]) -> TokenStream {
        // Generate match arms for implemented interfaces
        let base_match_arms = Self::gen_base_match_arms(interfaces);

        quote! {
            pub unsafe fn query_interface(
                self: &::std::pin::Pin<::std::boxed::Box<Self>>,
                riid: *const ::com::sys::IID,
                ppv: *mut *mut ::std::ffi::c_void
            ) -> ::com::sys::HRESULT {
                let riid = &*riid;

                if riid == &::com::interfaces::iunknown::IID_IUNKNOWN {
                    // Cast the &Pin<Box<T>> as a pointer and then dereference
                    // it to get the Pin<Box> as a pointer
                    *ppv = *(self as *const _ as *const *mut ::std::ffi::c_void);
                } #base_match_arms else {
                    *ppv = ::std::ptr::null_mut::<::std::ffi::c_void>();
                    return ::com::sys::E_NOINTERFACE;
                }

                self.add_ref();
                ::com::sys::NOERROR
            }
        }
    }

    fn gen_base_match_arms(interfaces: &[Interface]) -> TokenStream {
        // Generate match arms for implemented interfaces
        let base_match_arms = interfaces.iter().enumerate().map(|(index, interface)| {
            let interface = &interface.path;

            quote! {
                else if <#interface as ::com::Interface>::is_iid_in_inheritance_chain(riid) {
                    // Cast the &Pin<Box<T>> as a pointer and then dereference
                    // it to get the Pin<Box> as a pointer
                    *ppv = (*(self as *const _ as *const *mut usize)).add(#index) as *mut ::std::ffi::c_void;
                }
            }
        });

        quote!(#(#base_match_arms)*)
    }

    pub fn to_query_tokens(&self) -> TokenStream {
        quote! {
            pub fn query<T: ::com::Interface>(self: &::std::pin::Pin<::std::boxed::Box<Self>>) -> Option<T> {
                let mut result = None;
                let hr = unsafe { self.query_interface(&T::IID, &mut result as *mut _ as _) };

                if ::com::sys::FAILED(hr) {
                    assert!(
                        hr == ::com::sys::E_NOINTERFACE || hr == ::com::sys::E_POINTER,
                        "QueryInterface returned non-standard error"
                    );
                    return None;
                }
                debug_assert!(result.is_some(), "Successful call to query_interface yielded a null pointer");
                result
            }
        }
    }
}

fn this_ptr_type() -> TokenStream {
    quote! {
        ::std::ptr::NonNull<::std::ptr::NonNull<<::com::interfaces::IUnknown as ::com::Interface>::VTable>>
    }
}
