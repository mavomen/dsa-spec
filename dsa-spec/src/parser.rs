use crate::ast::Spec;

pub fn parse(spec_text: &str) -> Result<Spec, String> {
    serde_yaml::from_str::<Spec>(spec_text)
        .map_err(|e| format!("YAML parse error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_spec() {
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "Stack"
  category: "linear"
structs:
  - name: "Stack"
    generics:
      - name: "T"
    fields:
      - name: "items"
        type: "Vec<T>"
methods:
  - name: "push"
    params:
      - name: "item"
        type: "T"
    returns: "void"
verification:
  test_cases: []
"#;
        assert!(parse(yaml).is_ok());
    }

    #[test]
    fn test_parse_malformed_yaml() {
        let yaml = "invalid: [unclosed";
        assert!(parse(yaml).is_err());
    }

    #[test]
    fn test_malformed_yaml_error_message() {
        let yaml = "invalid: [unclosed";
        let err = parse(yaml).unwrap_err();
        // serde_yaml errors contain the location (line, column)
        assert!(err.contains("YAML parse error"));
        assert!(err.contains("line") || err.contains("column"));
    }
}
