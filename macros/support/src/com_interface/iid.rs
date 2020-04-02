use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as HelperTokenStream};
use quote::{format_ident, quote};
use syn::LitInt;

pub fn generate(macro_attr: &TokenStream, interface_ident: &Ident) -> HelperTokenStream {
    let iid_string: syn::LitStr =
        syn::parse(macro_attr.clone()).expect("[com_interface] parameter must be a GUID string");
    let iid_value = iid_string.value();
    assert!(
        iid_value.len() == 36,
        "IIDs must be exactly 36 characters long"
    );

    let iid_ident = ident(interface_ident);
    let iid_value = iid_value.as_str();
    let delimited: Vec<&str> = iid_value.split('-').collect();
    assert!(
        delimited.len() == 5,
        "IIDs must have 5 parts separate by '-'s"
    );

    assert!(
        delimited[0].len() == 8,
        "The first part of the IID must be 8 characters long, but it is {} characters long",
        delimited[0].len()
    );
    let data1 = LitInt::new(format!("0x{}", delimited[0]).as_str(), Span::call_site());

    assert!(
        delimited[1].len() == 4,
        "The second part of the IID must be 4 characters long, but it is {} characters long",
        delimited[1].len()
    );
    let data2 = LitInt::new(format!("0x{}", delimited[1]).as_str(), Span::call_site());

    assert!(
        delimited[2].len() == 4,
        "The third part of the IID must be 4 characters long, but it is {} characters long",
        delimited[2].len()
    );
    let data3 = LitInt::new(format!("0x{}", delimited[2]).as_str(), Span::call_site());

    assert!(
        delimited[3].len() == 4,
        "The fourth part of the IID must be 4 characters long, but it is {} characters long",
        delimited[3].len()
    );
    let (data4_1, data4_2) = delimited[3].split_at(2);
    let data4_1 = LitInt::new(format!("0x{}", data4_1).as_str(), Span::call_site());
    let data4_2 = LitInt::new(format!("0x{}", data4_2).as_str(), Span::call_site());

    assert!(
        delimited[4].len() == 12,
        "The fifth part of the IID must be 12 characters long, but it is {} characters long",
        delimited[4].len()
    );
    let (data4_3, rest) = delimited[4].split_at(2);
    let data4_3 = LitInt::new(format!("0x{}", data4_3).as_str(), Span::call_site());

    let (data4_4, rest) = rest.split_at(2);
    let data4_4 = LitInt::new(format!("0x{}", data4_4).as_str(), Span::call_site());

    let (data4_5, rest) = rest.split_at(2);
    let data4_5 = LitInt::new(format!("0x{}", data4_5).as_str(), Span::call_site());

    let (data4_6, rest) = rest.split_at(2);
    let data4_6 = LitInt::new(format!("0x{}", data4_6).as_str(), Span::call_site());

    let (data4_7, data4_8) = rest.split_at(2);
    let data4_7 = LitInt::new(format!("0x{}", data4_7).as_str(), Span::call_site());
    let data4_8 = LitInt::new(format!("0x{}", data4_8).as_str(), Span::call_site());

    quote!(
        #[allow(non_upper_case_globals, missing_docs)]
        pub const #iid_ident: com::sys::IID = com::sys::IID {
            data1: #data1,
            data2: #data2,
            data3: #data3,
            data4: [#data4_1, #data4_2, #data4_3, #data4_4, #data4_5, #data4_6, #data4_7, #data4_8]
        };
    )
}

pub fn ident(interface_ident: &Ident) -> Ident {
    format_ident!(
        "IID_{}",
        crate::utils::camel_to_snake(&interface_ident.to_string()).to_uppercase()
    )
}
