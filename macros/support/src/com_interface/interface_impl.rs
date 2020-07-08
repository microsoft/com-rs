use super::Interface;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, TraitItemMethod};

pub fn generate(interface: &Interface) -> TokenStream {
    let interface_name = &interface.name;
    let mut impl_methods = Vec::new();

    for m in &interface.items {
        impl_methods.push(gen_impl_method(m, interface_name));
    }

    quote! {
        impl #interface_name {
            #(#impl_methods)*
        }
    }
}

fn gen_impl_method(method: &TraitItemMethod, interface_name: &Ident) -> TokenStream {
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
