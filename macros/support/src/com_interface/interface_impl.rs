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

fn gen_impl_method(method: &InterfaceMethod) -> TokenStream {
    let inner_method_ident =
        format_ident!("{}", crate::utils::snake_to_camel(&method.name.to_string()));
    let interface_ptr_ident = format_ident!("interface_ptr");

    let outer_method_ident = &method.name;
    let return_type = &method.ret;

    let mut generics = Vec::new();
    let mut params = vec![quote!(#interface_ptr_ident)];
    let mut args = Vec::new();
    let mut into = Vec::new();
    for (index, syn::PatType { pat, ty, .. }) in method.args.iter().enumerate() {
        let generic = quote::format_ident!("__{}", index);
        args.push(quote! { #pat: #generic });
        generics.push(quote! { #generic: ::com::ComInterfaceParam<#ty> });
        into.push(quote! { let #pat = unsafe { #pat.into() }; });
        params.push(pat.to_token_stream());
    }

    let docs = &method.docs;
    return quote! {
        #(#docs)*
        pub unsafe fn #outer_method_ident<#(#generics),*>(&self, #(#args),*) #return_type {
            #(#into)*
            let #interface_ptr_ident = <Self as ::com::ComInterface>::as_raw(self);
            ((**#interface_ptr_ident.as_ptr()).#inner_method_ident)(#(#params),*)
        }
    };
}
