/// Convert snake_case to PascalCase.
/// `is_empty` → `IsEmpty`, `new` → `New`
pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert snake_case to camelCase.
/// `is_empty` → `isEmpty`, `pop_front` → `popFront`
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut upper = false;
    for ch in s.chars() {
        if ch == '_' {
            upper = true;
        } else if upper {
            result.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case_simple() {
        assert_eq!(to_pascal_case("insert"), "Insert");
    }

    #[test]
    fn test_to_pascal_case_snake() {
        assert_eq!(to_pascal_case("is_empty"), "IsEmpty");
        assert_eq!(to_pascal_case("pop_front"), "PopFront");
    }

    #[test]
    fn test_to_pascal_case_already_pascal() {
        assert_eq!(to_pascal_case("AlreadyPascal"), "AlreadyPascal");
    }

    #[test]
    fn test_to_pascal_case_empty() {
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_camel_case_simple() {
        assert_eq!(to_camel_case("insert"), "insert");
    }

    #[test]
    fn test_to_camel_case_snake() {
        assert_eq!(to_camel_case("is_empty"), "isEmpty");
        assert_eq!(to_camel_case("pop_front"), "popFront");
    }

    #[test]
    fn test_to_camel_case_single_word() {
        assert_eq!(to_camel_case("len"), "len");
    }

    #[test]
    fn test_to_camel_case_empty() {
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn test_to_camel_case_trailing_underscore() {
        assert_eq!(to_camel_case("foo_"), "foo");
    }
}
