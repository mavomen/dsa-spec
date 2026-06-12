/// Extract inner expression from `assert!(expr)`.
pub fn parse_assert_bang(s: &str) -> Option<&str> {
    s.strip_prefix("assert!(")?.strip_suffix(')')
}

/// Split `assert_eq!` arguments at the top-level comma (depth 0).
pub fn split_assert_eq_args(s: &str) -> Option<(&str, &str)> {
    let mut depth_paren = 0u32;
    let mut depth_bracket = 0u32;
    let mut depth_brace = 0u32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            ',' if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 => {
                return Some((&s[..i], &s[i + 1..]));
            }
            _ => {}
        }
    }
    None
}

/// Extract left and right from `assert_eq!(a, b)`.
pub fn parse_assert_eq(s: &str) -> Option<(&str, &str)> {
    let inner = s.strip_prefix("assert_eq!(")?.strip_suffix(')')?;
    let (l, r) = split_assert_eq_args(inner)?;
    Some((l.trim(), r.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assert_bang_simple() {
        assert_eq!(parse_assert_bang("assert!(true)"), Some("true"));
    }

    #[test]
    fn test_parse_assert_bang_with_expr() {
        assert_eq!(
            parse_assert_bang("assert!(avl.contains(2))"),
            Some("avl.contains(2)")
        );
    }

    #[test]
    fn test_parse_assert_bang_no_match() {
        assert_eq!(parse_assert_bang("assert_eq!(a, b)"), None);
        assert_eq!(parse_assert_bang("x = 1"), None);
    }

    #[test]
    fn test_parse_assert_eq_simple() {
        assert_eq!(parse_assert_eq("assert_eq!(a, b)"), Some(("a", "b")));
    }

    #[test]
    fn test_parse_assert_eq_with_parens() {
        assert_eq!(
            parse_assert_eq("assert_eq!(avl.height(), 2)"),
            Some(("avl.height()", "2"))
        );
    }

    #[test]
    fn test_parse_assert_eq_with_vec() {
        assert_eq!(
            parse_assert_eq("assert_eq!(avl.inorder(), vec![1, 2, 3])"),
            Some(("avl.inorder()", "vec![1, 2, 3]"))
        );
    }

    #[test]
    fn test_parse_assert_eq_no_match() {
        assert_eq!(parse_assert_eq("assert!(x)"), None);
        assert_eq!(parse_assert_eq("x == 1"), None);
    }

    #[test]
    fn test_split_assert_eq_args_no_comma() {
        assert_eq!(split_assert_eq_args("single"), None);
    }

    #[test]
    fn test_split_assert_eq_args_nested_parens() {
        assert_eq!(
            split_assert_eq_args("f(a, b), g(c, d)"),
            Some(("f(a, b)", " g(c, d)"))
        );
    }
}
