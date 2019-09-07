use quote::{format_ident,};
use syn::{
    Ident, Meta, NestedMeta, Fields, ItemStruct
};

use std::collections::HashMap;

pub fn get_class_factory_ident(class_ident: &Ident) -> Ident {
    format_ident!("{}ClassFactory", class_ident)
}

pub fn get_vtable_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VTable", trait_ident)
}

pub fn get_vptr_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VPtr", trait_ident)
}

pub fn get_non_del_unk_field_ident() -> Ident {
    format_ident!("__non_delegating_unk")
}

pub fn get_iunk_to_use_field_ident() -> Ident {
    format_ident!("__iunk_to_use")
}

pub fn get_ref_count_ident() -> Ident {
    format_ident!("__refcnt")
}

pub fn get_vptr_field_ident(trait_ident: &Ident) -> Ident {
    format_ident!("__{}vptr", trait_ident.to_string().to_lowercase())
}

pub fn get_real_ident(struct_ident: &Ident) -> Ident {
    if !struct_ident.to_string().starts_with("Init") {
        panic!("The target struct's name must begin with Init")
    }

    format_ident!("{}", &struct_ident.to_string()[4..])
}

pub fn get_inner_init_field_ident() -> Ident {
    format_ident!("__init_struct")
}

pub fn get_base_interface_idents(struct_item: &ItemStruct) -> Vec<Ident> {
    let mut base_itf_idents = Vec::new();

    for attr in &struct_item.attrs {
        if let Ok(Meta::List(ref attr)) = attr.parse_meta() {
            if attr.path.segments.last().unwrap().ident != "com_implements" {
                continue;
            }

            for item in &attr.nested {
                if let NestedMeta::Meta(Meta::Path(p)) = item {
                    assert!(p.segments.len() == 1, "Incapable of handling multiple path segments yet.");
                    base_itf_idents.push(p.segments.last().unwrap().ident.clone());
                }
            }
        }
    }

    base_itf_idents
}

/// Parse the arguments in helper attribute aggr. E.g. #[aggr(ICat, IAnimal)]
/// Returns a HashMap mapping each struct field ident to idents of the base
/// interfaces exposed by aggregate.
pub fn get_aggr_map(struct_item: &ItemStruct) -> HashMap<Ident, Vec<Ident>> {
    let mut aggr_map = HashMap::new();

    let fields = match &struct_item.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("Found field other than named fields in struct")
    };

    for field in fields {
        for attr in &field.attrs {
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
                let ident = field.ident.as_ref().unwrap().clone();
                aggr_map.insert(ident, aggr_interfaces_idents);
            }
        }
    }

    aggr_map
}