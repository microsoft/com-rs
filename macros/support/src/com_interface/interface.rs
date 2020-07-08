use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, TraitItemMethod, Visibility};

use super::iid::IID;

pub struct Interface {
    iid: IID,
    visibility: Visibility,
    name: Ident,
    parent: Option<Ident>,
    items: Vec<TraitItemMethod>,
}

impl Interface {
    pub fn to_struct_tokens(&self) -> TokenStream {
        let vis = &self.visibility;
        let name = &self.name;
        quote! {
            #vis struct #name {}
        }
    }

    pub fn to_iid_tokens(&self) -> TokenStream {
        self.iid.to_tokens(&self.name)
    }
}

impl syn::parse::Parse for Interface {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let mut iid = None;
        let mut docs = Vec::new();
        for attr in attributes.iter() {
            let path = &attr.path;
            let tokens = &attr.tokens;
            if path.is_ident("doc") {
                docs.push(attr);
            } else if path.is_ident("uuid") {
                let iid_str: ParenthsizedStr = syn::parse2(tokens.clone())?;

                iid = Some(IID::parse(&iid_str.lit)?);
            } else {
                return Err(syn::Error::new(
                    path.span().clone(),
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
        if name.to_string() != "IUnknown" {
            let _ = input.parse::<syn::Token![:]>()?;
            parent = Some(input.parse::<Ident>()?);
        }
        let content;
        syn::braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self {
            iid,
            visibility,
            items,
            name,
            parent,
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
        let lit = lit.parse()?;
        Ok(Self { lit })
    }
}
