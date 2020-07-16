use proc_macro2::{Ident, TokenStream};

use std::iter::FromIterator;

pub mod class_factory;
pub mod co_class_impl;
pub mod com_struct;
pub mod com_struct_impl;
pub mod iunknown_impl;

pub struct CoClass {
    name: Ident,
    docs: Vec<syn::Attribute>,
    visibility: syn::Visibility,
    interfaces: Vec<syn::Path>,
    fields: Vec<syn::Field>,
}

impl syn::parse::Parse for CoClass {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        while !input.is_empty() {
            let docs = input.call(syn::Attribute::parse_outer)?;
            //TODO: ensure only docs attributes
            if !input.peek(syn::Token!(impl)) {
                let visibility = input.parse::<syn::Visibility>()?;
                let _ = input.parse::<keywords::coclass>()?;
                let name = input.parse::<Ident>()?;
                let _ = input.parse::<syn::Token!(:)>()?;
                let mut interfaces = Vec::new();
                while !input.peek(syn::token::Brace) {
                    let path = input.parse::<syn::Path>()?;
                    interfaces.push(path);
                    if !input.peek(syn::token::Brace) {
                        let _ = input.parse::<syn::Token!(,)>()?;
                    }
                }
                let fields;
                syn::braced!(fields in input);
                let fields =
                    syn::punctuated::Punctuated::<syn::Field, syn::Token!(,)>::parse_terminated_with(
                        &fields,
                        syn::Field::parse_named
                    )?;
                let fields = fields.into_iter().collect();
                return Ok(CoClass {
                    name,
                    docs,
                    visibility,
                    interfaces,
                    fields,
                });
            }
        }
        todo!()
    }
}

impl CoClass {
    pub fn to_tokens(&self) -> TokenStream {
        // let base_interface_idents = crate::utils::base_interface_idents(attr_args);

        let mut out: Vec<TokenStream> = Vec::new();
        out.push(com_struct::generate(self));

        out.push(com_struct_impl::generate(self));

        out.push(co_class_impl::generate(self));

        out.push(iunknown_impl::generate(self));
        // out.push(class_factory::generate(input).into());

        TokenStream::from_iter(out)
    }
}

mod keywords {
    syn::custom_keyword!(coclass);
}
