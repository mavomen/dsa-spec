use crate::ast::Spec;
use serde_yaml;

pub fn parse(spec_text: &str) -> Result<Spec, String> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(spec_text).map_err(|e| format!("YAML syntax error: {}", e))?;
    serde_yaml::from_value::<Spec>(value).map_err(|e| format!("Spec validation error: {}", e))
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
        // The error should mention YAML syntax (from the first step) and contain location info
        assert!(err.contains("YAML syntax error"));
        // serde_yaml's error typically includes line/column numbers, e.g. "line 1, column 14"
        assert!(err.contains("line") || err.contains("column"));
    }
}
