pub fn snake_to_camel(input: &str) -> String {
    let mut new = String::new();

    let tokens: Vec<&str> = input.split('_').collect();
    for token in &tokens {
        let mut chars = token.chars();
        let title_string = match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        };

        new.push_str(title_string.as_str());
    }

    new
}

pub fn camel_to_snake(input: &str) -> String {
    let mut new = String::new();
    let mut seen_lowercase = false;

    for c in input.chars() {
        if c.is_uppercase() {
            if seen_lowercase {
                seen_lowercase = false;
                new.push_str("_");
            }
            new.push_str(&c.to_lowercase().to_string());
        } else {
            seen_lowercase = true;
            new.push_str(&c.to_string())
        }
    }

    new
}

pub fn get_iid_ident(trait_ident: &Ident) -> Ident {
    format_ident!(
        "IID_{}",
        camel_to_snake(trait_ident.to_string()).to_uppercase()
    )
}

pub fn get_vtable_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VTable", trait_ident)
}

pub fn get_vptr_ident(trait_ident: &Ident) -> Ident {
    format_ident!("{}VPtr", trait_ident)
}

pub fn get_vtable_macro_ident(struct_ident: &Ident) -> Ident {
    format_ident!(
        "{}_gen_vtable",
        camel_to_snake(struct_ident.to_string().replace("VTable", ""))
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_camel_to_snake() {
        let result = camel_to_snake("IAnimalVTable".into());
        assert_eq!(result, "ianimal_vtable".to_owned());
    }
    use super::*;
}
