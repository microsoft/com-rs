use proc_macro2::{Ident, Span, TokenStream as HelperTokenStream};
use quote::{format_ident, quote};
use syn::{LitInt, LitStr};

pub struct IID {
    parts: [String; 5],
}

impl IID {
    pub fn parse(iid_string: &LitStr) -> syn::Result<Self> {
        let iid_value = iid_string.value();
        let mut delimited = iid_value.split('-').fuse();
        let parts = [
            ensure_length(delimited.next(), 0, 8, iid_string.span())?,
            ensure_length(delimited.next(), 1, 4, iid_string.span())?,
            ensure_length(delimited.next(), 2, 4, iid_string.span())?,
            ensure_length(delimited.next(), 3, 4, iid_string.span())?,
            ensure_length(delimited.next(), 4, 12, iid_string.span())?,
        ];

        Ok(Self { parts })
    }

    pub fn to_tokens(&self, interface_ident: &Ident) -> HelperTokenStream {
        let iid_ident = ident(interface_ident);
        let data1 = hex_lit(&self.parts[0]);
        let data2 = hex_lit(&self.parts[1]);
        let data3 = hex_lit(&self.parts[2]);
        let (data4_1, data4_2) = self.parts[3].split_at(2);
        let data4_1 = hex_lit(data4_1);
        let data4_2 = hex_lit(data4_2);
        let (data4_3, rest) = self.parts[4].split_at(2);
        let data4_3 = hex_lit(data4_3);

        let (data4_4, rest) = rest.split_at(2);
        let data4_4 = hex_lit(data4_4);

        let (data4_5, rest) = rest.split_at(2);
        let data4_5 = hex_lit(data4_5);

        let (data4_6, rest) = rest.split_at(2);
        let data4_6 = hex_lit(data4_6);

        let (data4_7, data4_8) = rest.split_at(2);
        let data4_7 = hex_lit(data4_7);
        let data4_8 = hex_lit(data4_8);
        quote!(
            #[allow(missing_docs)]
            pub const #iid_ident: com::sys::IID = com::sys::IID {
                data1: #data1,
                data2: #data2,
                data3: #data3,
                data4: [#data4_1, #data4_2, #data4_3, #data4_4, #data4_5, #data4_6, #data4_7, #data4_8]
            };
        )
    }
}

pub fn ident(interface_ident: &Ident) -> Ident {
    format_ident!(
        "IID_{}",
        crate::utils::camel_to_snake(&interface_ident.to_string()).to_uppercase()
    )
}

fn ensure_length<'a>(
    part: Option<&'a str>,
    index: usize,
    length: usize,
    span: proc_macro2::Span,
) -> syn::Result<String> {
    let part = match part {
        Some(p) => p,
        None => {
            return Err(syn::Error::new(
                span,
                format!("The IID missing part at index {}", index,),
            ))
        }
    };

    if part.len() != length {
        return Err(syn::Error::new(
            span,
            format!(
                "The IID part at index {} must be {} characters long but was {} characters",
                index,
                length,
                part.len()
            ),
        ));
    }

    Ok(part.to_owned())
}

fn hex_lit(num: &str) -> LitInt {
    LitInt::new(&format!("0x{}", num), Span::call_site())
}
