//! YAML spec deserialization into the AST.
//! Returns `SpecError` with line/column information on failure.

use crate::ast::Spec;
use crate::error::SpecError;

/// Parse a YAML string into a `Spec` AST.
///
/// Returns parse errors with line and column numbers when available.
pub fn parse(spec_text: &str) -> Result<Spec, SpecError> {
    serde_yml::from_str::<Spec>(spec_text).map_err(|e| SpecError::ParseError {
        message: format!("YAML parse error: {e}"),
        line: e.location().map(|loc| loc.line()),
        column: e.location().map(|loc| loc.column()),
    })
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
        let msg = err.to_string();
        assert!(msg.contains("parse error"));
        assert!(msg.contains("YAML parse error"));
        assert!(msg.contains("line") || msg.contains("column"));
    }

    #[test]
    fn test_parse_empty_yaml_fails() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn test_parse_empty_structs_and_methods() {
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "Empty"
  category: "test"
structs: []
methods: []
verification:
  test_cases: []
"#;
        let spec = parse(yaml).expect("should parse");
        assert!(spec.structs.is_empty());
        assert!(spec.methods.is_empty());
        assert_eq!(spec.metadata.name, "Empty");
    }

    #[test]
    fn test_parse_unknown_fields_are_ignored() {
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
  extra_field: "ignored"
unknown_section:
  - foo: bar
structs: []
methods: []
verification:
  test_cases: []
"#;
        // serde ignores unknown fields by default
        assert!(parse(yaml).is_ok());
    }

    #[test]
    fn test_parse_parameterized_type() {
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "ParamTest"
  category: "test"
structs:
  - name: "Container"
    fields:
      - name: "items"
        type:
          base: "Vec"
          params:
            - base: "Option"
              params:
                - "T"
methods: []
verification:
  test_cases: []
"#;
        let spec = parse(yaml).expect("should parse parameterized type");
        let field = &spec.structs[0].fields[0];
        match &field.field_type {
            crate::ast::Type::Parameterized { base, params } => {
                assert_eq!(base, "Vec");
                assert_eq!(params.len(), 1);
            }
            _ => panic!("expected Parameterized type"),
        }
    }

    #[test]
    fn test_parse_no_verification() {
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "NoVerification"
  category: "test"
structs: []
methods: []
"#;
        let spec = parse(yaml).expect("spec without verification should parse");
        assert!(spec.verification.test_cases.is_empty());
    }
}
