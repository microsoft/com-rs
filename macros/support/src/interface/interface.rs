use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Path, Visibility};

use super::iid::IID;

pub struct Interface {
    pub iid: IID,
    pub visibility: Visibility,
    pub name: Ident,
    pub parent: Option<Path>,
    pub methods: Vec<InterfaceMethod>,
    docs: Vec<Attribute>,
}

impl Interface {
    pub fn to_struct_tokens(&self) -> TokenStream {
        let vis = &self.visibility;
        let name = &self.name;
        let vptr = super::vptr::ident(name);
        let docs = &self.docs;
        let impl_block = self.to_impl_block();
        quote! {
            #(#docs)*
            #[repr(transparent)]
            #[derive(Debug)]
            #[derive(PartialEq, Eq)]
            #vis struct #name {
                inner: ::core::ptr::NonNull<#vptr>,
            }
            #impl_block
        }
    }

    pub fn to_iid_tokens(&self) -> TokenStream {
        self.iid.to_tokens(&self.name)
    }

    pub fn is_iunknown(&self) -> bool {
        self.parent.is_none()
    }

    fn to_impl_block(&self) -> TokenStream {
        let interface_name = &self.name;

        let methods = self.methods.iter().map(|m| m.to_tokens());

        let deref = self.deref_impl();
        let drop = self.drop_impl();
        let clone = self.clone_impl();

        quote! {
            impl #interface_name {
                #(#methods)*
            }
            #deref
            #drop
            #clone
        }
    }

    fn deref_impl(&self) -> TokenStream {
        if self.is_iunknown() {
            return quote! {};
        }

        let name = &self.name;

        quote! {
            impl ::core::ops::Deref for #name {
                type Target = <#name as ::com::Interface>::Super;
                fn deref(&self) -> &Self::Target {
                    unsafe { ::core::mem::transmute(self) }
                }
            }
        }
    }

    fn drop_impl(&self) -> TokenStream {
        let name = &self.name;

        quote! {
            impl Drop for #name {
                fn drop(&mut self) {
                    unsafe { <Self as ::com::Interface>::as_iunknown(self).Release(); }
                }
            }
        }
    }

    fn clone_impl(&self) -> TokenStream {
        let name = &self.name;

        quote! {
            impl ::core::clone::Clone for #name {
                fn clone(&self) -> Self {
                    unsafe {
                        <Self as ::com::Interface>::as_iunknown(self).AddRef();
                    }
                    Self {
                        inner: self.inner
                    }
                }
            }
        }
    }
}

impl syn::parse::Parse for Interface {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let mut iid = None;
        let mut docs = Vec::new();
        for attr in attributes.into_iter() {
            let path = &attr.path;
            let tokens = &attr.tokens;
            if path.is_ident("doc") {
                docs.push(attr);
            } else if path.is_ident("uuid") {
                let iid_str: ParenthsizedStr = syn::parse2(tokens.clone())?;

                iid = Some(IID::parse(&iid_str.lit)?);
            } else {
                return Err(syn::Error::new(
                    path.span(),
                    format!("Unrecognized attribute '{}'", path.to_token_stream()),
                ));
            }
        }

        let visibility = input.parse::<syn::Visibility>()?;
        let _ = input.parse::<syn::Token![unsafe]>()?;
        let interface = input.parse::<keywords::interface>()?;
        let iid = match iid {
            Some(iid) => iid,
            None => {
                return Err(syn::Error::new(
                    interface.span(),
                    "Interfaces must have a '#[uuid(\"$IID\")]' attribute",
                ))
            }
        };
        let name = input.parse::<Ident>()?;
        let mut parent = None;
        if name != "IUnknown" {
            let _ = input.parse::<syn::Token![:]>().map_err(|_| {
                syn::Error::new(
                    name.span(),
                    format!("Interfaces must inherit from another interface like so: `interface {}: IParentInterface`", name),
                )
            })?;
            parent = Some(input.parse::<Path>()?);
        }
        let content;
        syn::braced!(content in input);
        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(content.parse::<InterfaceMethod>()?);
        }
        Ok(Self {
            iid,
            visibility,
            methods,
            name,
            parent,
            docs,
        })
    }
}

mod keywords {
    syn::custom_keyword!(interface);
}

struct ParenthsizedStr {
    lit: syn::LitStr,
}

impl syn::parse::Parse for ParenthsizedStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit;
        syn::parenthesized!(lit in input);
        let lit = lit
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "uuids must be string literals".to_string()))?;

        Ok(Self { lit })
    }
}

pub struct InterfaceMethod {
    pub name: Ident,
    pub visibility: Visibility,
    pub args: Vec<InterfaceMethodArg>,
    pub ret: syn::ReturnType,
    pub docs: Vec<syn::Attribute>,
}

pub struct InterfaceMethodArg {
    pub ty: Box<syn::Type>,
    pub pat: Box<syn::Pat>,
    pub pass_through: bool,
}

macro_rules! bail {
    ($item:expr, $($msg:tt),*) => {
        return Err(syn::Error::new($item.span(), std::fmt::format(format_args!($($msg),*))));
    };

}

macro_rules! unexpected_token {
    ($item:expr, $msg:expr) => {
        if let Some(i) = $item {
            bail!(i, "unexpected {}", $msg);
        }
    };
}
macro_rules! expected_token {
    ($sig:tt.$item:tt(), $msg:expr) => {
        if let None = $sig.$item() {
            bail!($sig, "expected {}", $msg);
        }
    };
}

impl syn::parse::Parse for InterfaceMethod {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let docs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse::<syn::Visibility>()?;
        let method = input.parse::<syn::TraitItemMethod>()?;
        unexpected_token!(docs.iter().find(|a| !a.path.is_ident("doc")), "attribute");
        unexpected_token!(method.default, "default method implementation");
        let sig = method.sig;
        unexpected_token!(sig.abi, "abi declaration");
        unexpected_token!(sig.asyncness, "async declaration");
        unexpected_token!(sig.generics.params.iter().next(), "generics declaration");
        unexpected_token!(sig.constness, "const declaration");
        expected_token!(
            sig.receiver(),
            "the method to have &self as its first argument"
        );
        unexpected_token!(sig.variadic, "variadic args");
        let args = sig
            .inputs
            .into_iter()
            .filter_map(|a| match a {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(p) => Some(p),
            })
            .map(|p| {
                let mut filter = p
                    .attrs
                    .iter()
                    .filter(|a| a.path.is_ident("pass_through"))
                    .fuse();
                let pass_through = filter.next().is_some();

                unexpected_token!(filter.next(), "function attribute");
                Ok(InterfaceMethodArg {
                    ty: p.ty,
                    pat: p.pat,
                    pass_through,
                })
            })
            .collect::<Result<Vec<InterfaceMethodArg>, syn::Error>>()?;

        let ret = sig.output;
        Ok(InterfaceMethod {
            name: sig.ident,
            visibility,
            args,
            ret,
            docs,
        })
    }
}

impl InterfaceMethod {
    fn to_tokens(&self) -> TokenStream {
        let inner_method_ident =
            format_ident!("{}", crate::utils::snake_to_camel(&self.name.to_string()));
        let interface_ptr_ident = format_ident!("interface_ptr");

        let outer_method_ident = &self.name;
        let return_type = &self.ret;

        let mut generics = Vec::new();
        if !self.args.is_empty() {
            generics.push(quote! { 'a })
        }
        let mut params = vec![quote!(#interface_ptr_ident)];
        let mut args = Vec::new();
        let mut into = Vec::new();
        for (index, arg) in self.args.iter().enumerate() {
            let pat = &arg.pat;
            let ty = &arg.ty;
            if arg.pass_through {
                args.push(quote! { #pat: #ty });
            } else {
                let generic = quote::format_ident!("__{}", index);
                args.push(quote! { #pat: #generic });
                generics.push(quote! { #generic: ::core::convert::Into<::com::Param<'a, #ty>> });

                // note: we separate the call to `into` and `get_abi` so that the `param`
                // binding lives to the end of the method.
                into.push(quote! {
                    let mut param = #pat.into();
                    let #pat = param.get_abi();
                });
            }
            params.push(pat.to_token_stream());
        }

        let docs = &self.docs;
        let vis = &self.visibility;
        return quote! {
            #[allow(non_snake_case)]
            #[allow(clippy::from_over_into)]
            #(#docs)*
            #vis unsafe fn #outer_method_ident<#(#generics),*>(&self, #(#args),*) #return_type {
                #(#into)*
                let #interface_ptr_ident = <Self as ::com::AbiTransferable>::get_abi(self);
                (#interface_ptr_ident.as_ref().as_ref().#inner_method_ident)(#(#params),*)
            }
        };
    }
}
