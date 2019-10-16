mod idents;
pub use idents::*;

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_camel_to_snake() {
        let result = camel_to_snake("IAnimalVTable".into());
        assert_eq!(result, "ianimal_vtable".to_owned());
    }
    use super::*;
}
