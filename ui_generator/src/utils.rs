use std::collections::HashSet;

pub(super) fn to_pascal_case(input: &str) -> String {
    input
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

pub(crate) trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for &str {
    fn to_snake_case(&self) -> String {
        let mut result = String::new();

        for (i, ch) in self.chars().enumerate() {
            if ch.is_uppercase() {
                if i != 0 {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
            } else {
                result.push(ch);
            }
        }

        result
    }
}

impl ToSnakeCase for String {
    fn to_snake_case(&self) -> String {
        self.as_str().to_snake_case()
    }
}

pub(super) fn join_usings(usings: &HashSet<String>) -> String {
    let mut output: String = String::new();
    usings.iter().for_each(|using| {
        output.push_str("use ");
        output.push_str(using);
        output.push(';');
    });
    output
}