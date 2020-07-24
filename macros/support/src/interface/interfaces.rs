use super::Interface;
use std::collections::HashMap;

pub struct Interfaces {
    pub inner: Vec<Interface>,
    pub parents: HashMap<proc_macro2::Ident, syn::Path>,
}

impl syn::parse::Parse for Interfaces {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut interfaces = Vec::new();
        let mut parents = HashMap::new();
        while !input.is_empty() {
            let interface: Interface = input.parse()?;
            if let Some(parent) = interface.parent.clone() {
                parents.insert(interface.name.clone(), parent);
            }
            interfaces.push(interface);
        }
        Ok(Self {
            inner: interfaces,
            parents,
        })
    }
}
