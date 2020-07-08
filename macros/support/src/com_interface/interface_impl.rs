use super::Interface;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, TraitItemMethod};

pub fn generate(interface: &Interface) -> TokenStream {
    let interface_name = &interface.name;
    let mut impl_methods = Vec::new();

    for m in &interface.items {
        impl_methods.push(gen_impl_method(m));
    }

    let deref = deref_impl(interface);
    let convert = convert_impl(interface);

    quote! {
        impl #interface_name {
            #(#impl_methods)*
        }
        #deref
        #convert
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
                unsafe { ::std::mem::transmute_copy(self) }
            }
        }
    }
}

fn convert_impl(interface: &Interface) -> TokenStream {
    if interface.is_iunknown() {
        return quote! {};
    }

    let name = &interface.name;

    quote! {
        impl ::std::convert::From<#name> for <#name as ::com::ComInterface>::Super {
            fn from(this: #name) -> Self {
                unsafe { ::std::mem::transmute(this) }
            }
        }
    }
}

fn gen_impl_method(method: &TraitItemMethod) -> TokenStream {
    let method_sig = &method.sig;
    let method_ident = format_ident!(
        "{}",
        crate::utils::snake_to_camel(&method.sig.ident.to_string())
    );
    let interface_ptr_ident = format_ident!("interface_ptr");

    let mut params = Vec::new();
    for param in method.sig.inputs.iter() {
        match param {
            FnArg::Receiver(_n) => params.push(quote!(#interface_ptr_ident)),
            FnArg::Typed(n) => params.push(n.pat.to_token_stream()),
        }
    }

    quote!(
        #[allow(missing_docs)]
        pub #method_sig {
            let #interface_ptr_ident = <Self as ::com::ComInterface>::as_raw(self);
            ((**#interface_ptr_ident.as_ref()).#method_ident)(#(#params),*)
        }
    )
}
