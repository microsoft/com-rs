use super::{interface::MethodDecl, Interface, InterfaceMethod};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

pub fn generate(interface: &Interface) -> TokenStream {
    let interface_name = &interface.name;
    let mut impl_methods = Vec::new();

    for m in &interface.methods {
        if let MethodDecl::Method(m) = m {
            impl_methods.push(gen_impl_method(m));
        }
    }

    let deref = deref_impl(interface);
    let drop = drop_impl(interface);
    let clone = clone_impl(interface);
    let abi = abi_impl(interface);

    quote! {
        impl #interface_name {
            #(#impl_methods)*
        }
        #deref
        #drop
        #clone
        #abi
    }
}

fn deref_impl(interface: &Interface) -> TokenStream {
    if interface.is_iunknown() {
        return quote! {};
    }

    let name = &interface.name;

    quote! {
        impl ::std::ops::Deref for #name {
            type Target = <#name as ::com::ComInterface>::Super;
            fn deref(&self) -> &Self::Target {
                unsafe { ::std::mem::transmute(self) }
            }
        }
    }
}

fn drop_impl(interface: &Interface) -> TokenStream {
    let name = &interface.name;

    quote! {
        impl Drop for #name {
            fn drop(&mut self) {
                unsafe {
                    self.as_iunknown().release();
                }
            }
        }
    }
}

fn clone_impl(interface: &Interface) -> TokenStream {
    let name = &interface.name;

    quote! {
        impl ::std::clone::Clone for #name {
            fn clone(&self) -> Self {
                unsafe {
                    self.as_iunknown().add_ref();
                }
                Self {
                    inner: self.inner
                }
            }
        }
    }
}

fn abi_impl(interface: &Interface) -> TokenStream {
    let name = &interface.name;

    quote! {
        unsafe impl ::com::AbiTransferable for #name {
            type Abi = std::ptr::NonNull<std::ptr::NonNull<<#name as ::com::ComInterface>::VTable>>;
            fn get_abi(&self) -> Self::Abi {
                self.inner
            }
            fn set_abi(&mut self) -> *mut Self::Abi {
                &mut self.inner
            }
        }
    }
}
fn gen_impl_method(method: &InterfaceMethod) -> TokenStream {
    let inner_method_ident =
        format_ident!("{}", crate::utils::snake_to_camel(&method.name.to_string()));
    let interface_ptr_ident = format_ident!("interface_ptr");

    let outer_method_ident = &method.name;
    let return_type = &method.ret;

    let mut generics = Vec::new();
    if method.args.len() > 0 {
        generics.push(quote! { 'a })
    }
    let mut params = vec![quote!(#interface_ptr_ident)];
    let mut args = Vec::new();
    let mut into = Vec::new();
    for (index, syn::PatType { pat, ty, .. }) in method.args.iter().enumerate() {
        let generic = quote::format_ident!("__{}", index);
        args.push(quote! { #pat: #generic });
        generics.push(quote! { #generic: ::std::convert::Into<::com::Param<'a, #ty>> });
        into.push(quote! {
            let #pat = #pat.into().get_abi();
        });
        params.push(pat.to_token_stream());
    }

    let docs = &method.docs;
    return quote! {
        #(#docs)*
        pub unsafe fn #outer_method_ident<#(#generics),*>(&self, #(#args),*) #return_type {
            #(#into)*
            let #interface_ptr_ident = <Self as ::com::AbiTransferable>::get_abi(self);
            (#interface_ptr_ident.as_ref().as_ref().#inner_method_ident)(#(#params),*)
        }
    };
}
