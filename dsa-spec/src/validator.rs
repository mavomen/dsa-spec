use crate::ast::Spec;
use crate::spec_schema::SPEC_JSON_SCHEMA;
use jsonschema::{Draft, JSONSchema, ValidationError};

pub fn validate(spec: &Spec) -> Result<(), Vec<String>> {
    let value = serde_json::to_value(spec)
        .map_err(|e| vec![format!("Internal serialization error: {}", e)])?;

    let schema_json: serde_json::Value = serde_json::from_str(SPEC_JSON_SCHEMA)
        .map_err(|e| vec![format!("Internal schema parse error: {}", e)])?;

    let schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| vec![format!("Schema compilation error: {}", e)])?;

    let errors: Vec<ValidationError> = match schema.validate(&value) {
        Ok(()) => return Ok(()),
        Err(errors) => errors.collect(),
    };

    let messages: Vec<String> = errors
        .iter()
        .map(|e| format!("{} (at {})", e, e.instance_path))
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
        assert!(errors.iter().any(|e| e.contains("name")));
        assert!(errors.iter().any(|e| e.contains("category")));
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
