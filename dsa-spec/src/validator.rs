//! JSON Schema validation for parsed specs.

use crate::ast::Spec;
use crate::error::SpecError;
use crate::spec_schema::SPEC_JSON_SCHEMA;
use jsonschema::{Draft, JSONSchema, ValidationError};

/// Validate a parsed spec against the JSON Schema.
///
/// Returns `Ok(())` on success or a list of `SpecError` values with
/// path information for each validation failure.
pub fn validate(spec: &Spec) -> Result<(), Vec<SpecError>> {
    let value = serde_json::to_value(spec).map_err(|e| {
        vec![SpecError::SchemaError {
            message: format!("Internal serialization error: {e}"),
        }]
    })?;

    let schema_json: serde_json::Value = serde_json::from_str(SPEC_JSON_SCHEMA).map_err(|e| {
        vec![SpecError::SchemaError {
            message: format!("Internal schema parse error: {e}"),
        }]
    })?;

    let schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| {
            vec![SpecError::SchemaError {
                message: format!("Schema compilation error: {e}"),
            }]
        })?;

    let errors: Vec<ValidationError> = match schema.validate(&value) {
        Ok(()) => return Ok(()),
        Err(errors) => errors.collect(),
    };

    let messages: Vec<SpecError> = errors
        .iter()
        .map(|e| {
            let path = e.instance_path.to_string();
            SpecError::ValidationError {
                message: format!("{e}"),
                path,
            }
        })
        .collect();

    Err(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        Complexity, Contracts, FieldDef, GenericParam, Metadata, MethodDef, ParamDef, Spec,
        StructDef, TestCase, Type, Verification,
    };

    fn make_valid_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Stack".into(),
                category: "linear".into(),
                complexity: Complexity {
                    time: Some("O(1)".into()),
                    space: Some("O(n)".into()),
                },
                tags: vec!["lifo".into()],
            },
            contracts: Contracts {
                invariants: vec!["size >= 0".into()],
            },
            structs: vec![StructDef {
                name: "Stack".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    constraints: vec!["Clone".into()],
                }],
                fields: vec![FieldDef {
                    name: "items".into(),
                    field_type: Type::Simple("Vec<T>".into()),
                }],
            }],
            methods: vec![MethodDef {
                name: "push".into(),
                params: vec![ParamDef {
                    name: "item".into(),
                    param_type: Type::Simple("T".into()),
                }],
                returns: Some("void".into()),
                preconditions: vec!["stack not full".into()],
                postconditions: vec!["item on top".into()],
                injected_assertions: vec![],
            }],
            verification: Verification {
                test_cases: vec![TestCase {
                    name: "push_works".into(),
                    setup: Some("let mut s = Stack::new();".into()),
                    actions: vec!["s.push(1)".into()],
                    assertions: vec!["assert_eq!(s.pop(), Some(1))".into()],
                }],
            },
        }
    }

    #[test]
    fn test_valid_spec_passes() {
        let spec = make_valid_spec();
        assert!(validate(&spec).is_ok());
    }

    #[test]
    fn test_missing_metadata_fails() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "".into(),
                category: "".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = validate(&spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.to_string().contains("name")));
        assert!(errors.iter().any(|e| e.to_string().contains("category")));
    }

    #[test]
    fn test_missing_struct_name_fails() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            structs: vec![StructDef {
                name: "".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(validate(&spec).is_err());
    }

    #[test]
    fn test_missing_method_name_fails() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            methods: vec![MethodDef {
                name: "".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(validate(&spec).is_err());
    }

    #[test]
    fn test_missing_metadata_name_fails() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "".into(),
                category: "real".into(),
                ..Default::default()
            },
            structs: vec![],
            methods: vec![],
            ..Default::default()
        };
        let result = validate(&spec);
        assert!(result.is_err(), "empty name should fail: {:?}", result);
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.to_string().contains("name")));
    }

    #[test]
    fn test_missing_field_type_fails() {
        // The schema requires field to have "type" key. If we omit it
        // via Default, the Type defaults to Simple("") which is still a
        // valid string. The schema doesn't enforce minLength on field type.
        // So this validates that a missing type key in YAML would fail.
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs:
  - name: "Foo"
    fields:
      - name: "bar"
methods: []
verification:
  test_cases: []
"#;
        // Missing "type" key in field fails at deserialization (serde requires it)
        let result = serde_yaml::from_str::<crate::ast::Spec>(yaml);
        assert!(
            result.is_err(),
            "missing field 'type' should fail deserialization"
        );
    }

    #[test]
    fn test_non_object_complexity_yaml_fails_parse() {
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
  complexity: "O(1)"
structs: []
methods: []
verification:
  test_cases: []
"#;
        // YAML type mismatch: complexity expects an object, got a string
        assert!(serde_yaml::from_str::<crate::ast::Spec>(yaml).is_err());
    }

    #[test]
    fn test_tags_as_string_fails() {
        // Should this pass? serde_yaml will fail to deserialize tags: "lifo"
        let yaml = r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
  tags: "lifo"
structs: []
methods: []
verification:
  test_cases: []
"#;
        // YAML type mismatch: tags is expected to be an array
        assert!(serde_yaml::from_str::<crate::ast::Spec>(yaml).is_err());
    }

    #[test]
    fn test_bst_invariants_pass_validation() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "BST".into(),
                category: "trees".into(),
                ..Default::default()
            },
            contracts: Contracts {
                invariants: vec![
                    "left.value < node.value < right.value".into(),
                    "no duplicate values".into(),
                ],
            },
            structs: vec![
                StructDef {
                    name: "BSTNode".into(),
                    generics: vec![GenericParam {
                        name: "T".into(),
                        constraints: vec!["Ord".into(), "Clone".into()],
                    }],
                    fields: vec![
                        FieldDef {
                            name: "value".into(),
                            field_type: Type::Simple("T".into()),
                        },
                        FieldDef {
                            name: "left".into(),
                            field_type: Type::Simple("Option<Box<BSTNode<T>>>".into()),
                        },
                        FieldDef {
                            name: "right".into(),
                            field_type: Type::Simple("Option<Box<BSTNode<T>>>".into()),
                        },
                    ],
                },
                StructDef {
                    name: "BinarySearchTree".into(),
                    generics: vec![GenericParam {
                        name: "T".into(),
                        constraints: vec!["Ord".into(), "Clone".into()],
                    }],
                    fields: vec![FieldDef {
                        name: "root".into(),
                        field_type: Type::Simple("Option<Box<BSTNode<T>>>".into()),
                    }],
                },
            ],
            methods: vec![MethodDef {
                name: "insert".into(),
                params: vec![ParamDef {
                    name: "value".into(),
                    param_type: Type::Simple("T".into()),
                }],
                returns: Some("bool".into()),
                postconditions: vec!["tree contains value".into()],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(validate(&spec).is_ok());
    }
}
