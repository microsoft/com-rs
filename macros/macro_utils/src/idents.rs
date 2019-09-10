use quote::{format_ident,};
use syn::{
    Ident, Meta, NestedMeta, ItemStruct,
};

use std::collections::HashMap;

pub fn class_factory_ident(class_ident: &Ident) -> Ident {
    format_ident!("{}ClassFactory", class_ident)
}

pub fn vtable_ident(interface_ident: &Ident) -> Ident {
    format_ident!("{}VTable", interface_ident)
}

pub fn vptr_ident(interface_ident: &Ident) -> Ident {
    format_ident!("{}VPtr", interface_ident)
}

pub fn non_delegating_iunknown_field_ident() -> Ident {
    format_ident!("__non_delegatingegating_iunknown")
}

pub fn iunknown_to_use_field_ident() -> Ident {
    format_ident!("__iunknown_to_use")
}

pub fn ref_count_ident() -> Ident {
    format_ident!("__refcnt")
}

pub fn vptr_field_ident(interface_ident: &Ident) -> Ident {
    format_ident!("__{}vptr", interface_ident.to_string().to_lowercase())
}

pub fn set_aggregate_fn_ident(base: &Ident) -> Ident {
    format_ident!("set_aggregate_{}", crate::camel_to_snake(&base.to_string()))
}

pub fn base_interface_idents(struct_item: &ItemStruct) -> Vec<Ident> {
    let mut base_interface_idents = Vec::new();

    for attr in &struct_item.attrs {
        if let Ok(Meta::List(ref attr)) = attr.parse_meta() {
            if attr.path.segments.last().unwrap().ident != "com_implements" {
                continue;
            }

            for item in &attr.nested {
                if let NestedMeta::Meta(Meta::Path(p)) = item {
                    assert!(p.segments.len() == 1, "Incapable of handling multiple path segments yet.");
                    base_interface_idents.push(p.segments.last().unwrap().ident.clone());
                }
            }
        }
    }

    base_interface_idents
}

/// Parse the arguments in helper attribute aggr. E.g. #[aggr(ICat, IAnimal)]
/// Returns a HashMap mapping each struct field ident to idents of the base
/// interfaces exposed by aggregate.
pub fn get_aggr_map(struct_item: &ItemStruct) -> HashMap<Ident, Vec<Ident>> {
    let mut aggr_map = HashMap::new();

    for attr in &struct_item.attrs {
        if let Ok(Meta::List(ref attr)) = attr.parse_meta() {
            if attr.path.segments.last().unwrap().ident != "aggr" {
                continue;
            }

            let mut aggr_interfaces_idents = Vec::new();


            assert!(attr.nested.len() > 0, "Need to expose at least one interface from aggregated COM object.");

            for item in &attr.nested {
                if let NestedMeta::Meta(Meta::Path(p)) = item {
                    assert!(p.segments.len() == 1, "Incapable of handling multiple path segments yet.");
                    aggr_interfaces_idents.push(p.segments.last().unwrap().ident.clone());
                }
            }
            let ident = aggr_interfaces_idents.iter()
            .map(|base| {
                crate::camel_to_snake(&base.to_string())
            })
            .fold("aggr".to_owned(), |acc, base| {
                format!("{}_{}", acc, base)
            });
            aggr_map.insert(format_ident!("{}", ident), aggr_interfaces_idents);
        }
    }

    aggr_map
}