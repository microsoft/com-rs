use quote::format_ident;
use syn::{AttributeArgs, Ident, Meta, NestedMeta};

pub fn class_factory_ident(class_ident: &Ident) -> Ident {
    format_ident!("{}ClassFactory", class_ident)
}

pub fn ref_count_ident() -> Ident {
    format_ident!("__refcnt")
}

pub fn vptr_field_ident(interface_ident: &Ident) -> Ident {
    format_ident!("__{}vptr", interface_ident.to_string().to_lowercase())
}

pub fn base_interface_idents(attr_args: &AttributeArgs) -> Vec<Ident> {
    let mut base_interface_idents = Vec::new();

    for attr_arg in attr_args {
        if let NestedMeta::Meta(Meta::List(ref attr)) = attr_arg {
            if attr
                .path
                .segments
                .last()
                .expect("Invalid attribute syntax")
                .ident
                != "implements"
            {
                continue;
            }

            for item in &attr.nested {
                if let NestedMeta::Meta(Meta::Path(p)) = item {
                    assert!(
                        p.segments.len() == 1,
                        "Incapable of handling multiple path segments yet."
                    );
                    base_interface_idents.push(
                        p.segments
                            .last()
                            .expect("Implemented interface is empty path")
                            .ident
                            .clone(),
                    );
                }
            }
        }
    }

    base_interface_idents
}
