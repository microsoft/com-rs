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

    quote! {
        impl #interface_name {
            #(#impl_methods)*
        }
        #deref
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

fn gen_impl_method(method: &TraitItemMethod) -> TokenStream {
    let method_sig = &method.sig;
    let inner_method_ident = format_ident!(
        "{}",
        crate::utils::snake_to_camel(&method.sig.ident.to_string())
    );
    let interface_ptr_ident = format_ident!("interface_ptr");

    let mut params = Vec::new();
    let mut args = Vec::new();
    for param in method.sig.inputs.iter() {
        match param {
            FnArg::Receiver(_n) => params.push(quote!(#interface_ptr_ident)),
            FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                args.push(quote! { #pat: #ty });
                params.push(pat.to_token_stream());
            }
        }
    }

    let outer_method_ident = &method_sig.ident;
    let return_type = &method_sig.output;

    quote!(
        #[allow(missing_docs)]
        pub unsafe fn #outer_method_ident(&self, #(#args),*) #return_type {
            let #interface_ptr_ident = <Self as ::com::ComInterface>::as_raw(self);
            ((**#interface_ptr_ident.as_ref()).#inner_method_ident)(#(#params),*)
        }
    )
}
