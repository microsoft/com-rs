use super::vptr;

use proc_macro2::{Ident, TokenStream as HelperTokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, ItemTrait, TraitItem, TraitItemMethod};

pub fn generate(interface: &ItemTrait) -> HelperTokenStream {
    let interface_ident = &interface.ident;
    let mut impl_methods = Vec::new();

    for trait_item in &interface.items {
        match trait_item {
            TraitItem::Method(n) => {
                impl_methods.push(gen_impl_method(&interface.ident, n));
            }
            _ => panic!("COM interfaces may only contain methods"),
        }
    }

    quote! {
        impl <T: #interface_ident + com::ComInterface + ?Sized> #interface_ident for com::ComRc<T> {
            #(#impl_methods)*
        }

        impl <T: #interface_ident + com::ComInterface + ?Sized> #interface_ident for com::ComPtr<T> {
            #(#impl_methods)*
        }
    }
}

fn gen_impl_method(interface_ident: &Ident, method: &TraitItemMethod) -> HelperTokenStream {
    let method_sig = &method.sig;
    let vptr_ident = vptr::ident(&interface_ident.to_string());
    let method_ident = format_ident!(
        "{}",
        crate::utils::snake_to_camel(&method.sig.ident.to_string())
    );
    let interface_ptr_ident = format_ident!("interface_ptr");

    let mut params = Vec::new();
    for param in method.sig.inputs.iter() {
        match param {
            FnArg::Receiver(_n) => params.push(quote!(#interface_ptr_ident)),
            // TODO: This may go wrong, I am using everything on the LHS.
            FnArg::Typed(n) => params.push(n.pat.to_token_stream()),
        }
    }

    quote!(
        #[allow(missing_docs)]
        #method_sig {
            let #interface_ptr_ident = self.as_raw() as *mut #vptr_ident;
            ((**#interface_ptr_ident).#method_ident)(#(#params),*)
        }
    )
}
