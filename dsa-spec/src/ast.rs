//! Language-agnostic AST types for DSA-SPEC specs.
//! These types are deserialized from YAML and consumed by language backends.

use serde::{Deserialize, Serialize};

/// Top-level specification.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Spec {
    /// Schema version string (e.g. `"1.0"`).
    pub spec_version: String,
    /// Name, category, complexity, tags.
    pub metadata: Metadata,
    /// Invariants that must always hold.
    #[serde(default)]
    pub contracts: Contracts,
    /// Data structure definitions.
    #[serde(default)]
    pub structs: Vec<StructDef>,
    /// Method signatures with pre/postconditions.
    #[serde(default)]
    pub methods: Vec<MethodDef>,
    /// Test cases: setup, actions, assertions.
    #[serde(default)]
    pub verification: Verification,
}

/// Metadata about a DSA specification.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// Human-readable name (e.g. `"BinarySearchTree"`).
    pub name: String,
    /// Category label (e.g. `"trees"`, `"sorting"`).
    pub category: String,
    /// Time and space complexity annotations.
    #[serde(default)]
    pub complexity: Complexity,
    /// Free-form tags for search and grouping.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Big-O complexity annotations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Complexity {
    /// Time complexity string (e.g. `"O(log n)"`).
    pub time: Option<String>,
    /// Space complexity string (e.g. `"O(n)"`).
    pub space: Option<String>,
}

/// Invariants that must hold for all instances of the data structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contracts {
    /// Invariant conditions expressed as strings.
    #[serde(default)]
    pub invariants: Vec<String>,
}

/// A struct or type definition in the spec.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructDef {
    pub name: String,
    /// Generic type parameters.
    #[serde(default)]
    pub generics: Vec<GenericParam>,
    /// Named fields with their types.
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

/// A generic type parameter with optional trait bounds.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenericParam {
    pub name: String,
    /// Constraint trait names (e.g. `"Clone"`, `"Ord"`).
    #[serde(default)]
    pub constraints: Vec<String>,
}

/// A named field with a type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldDef {
    pub name: String,
    /// The field's type (language-agnostic).
    #[serde(rename = "type")]
    pub field_type: Type,
}

/// A method signature with optional contracts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MethodDef {
    pub name: String,
    /// Ordered parameter list.
    #[serde(default)]
    pub params: Vec<ParamDef>,
    /// Return type string (e.g. `"bool"`, `"Option<T>"`).
    pub returns: Option<String>,
    /// Conditions that must hold before the method runs.
    #[serde(default)]
    pub preconditions: Vec<String>,
    /// Conditions guaranteed after the method returns.
    #[serde(default)]
    pub postconditions: Vec<String>,
    /// Assertion strings injected by the contracts module.
    #[serde(default)]
    pub injected_assertions: Vec<String>,
}

/// A method parameter.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParamDef {
    pub name: String,
    /// Parameter type.
    #[serde(rename = "type")]
    pub param_type: Type,
}

/// Test case collection for a spec.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Verification {
    /// Individual test scenarios.
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
}

/// A single test scenario with setup, actions, and assertions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestCase {
    pub name: String,
    /// Optional setup code (e.g. constructor call).
    pub setup: Option<String>,
    /// Sequence of method calls or operations.
    #[serde(default)]
    pub actions: Vec<String>,
    /// Assertions to verify after actions run.
    #[serde(default)]
    pub assertions: Vec<String>,
}

/// A language-agnostic type representation.
///
/// Supports simple strings (`"i32"`, `"Vec<T>"`) and parameterized
/// types with a base name and type parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Type {
    /// Simple type string. E.g. `"T"`, `"i32"`, `"Vec<T>"`.
    Simple(String),
    /// A parameterized type with explicit type arguments.
    /// E.g. `HashMap<K, V>` represented as `Parameterized { base: "HashMap", params: [K, V] }`.
    Parameterized { base: String, params: Vec<Type> },
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Simple(s) => write!(f, "{s}"),
            Type::Parameterized { base, params } => {
                write!(f, "{base}<")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ">")
            }
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Simple(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display_simple() {
        let t = Type::Simple("i32".into());
        assert_eq!(t.to_string(), "i32");
    }

    #[test]
    fn test_type_display_parameterized() {
        let t = Type::Parameterized {
            base: "Vec".into(),
            params: vec![Type::Simple("T".into())],
        };
        assert_eq!(t.to_string(), "Vec<T>");
    }

    #[test]
    fn test_type_display_nested_parameterized() {
        let t = Type::Parameterized {
            base: "Option".into(),
            params: vec![Type::Parameterized {
                base: "Box".into(),
                params: vec![Type::Parameterized {
                    base: "BSTNode".into(),
                    params: vec![Type::Simple("T".into())],
                }],
            }],
        };
        assert_eq!(t.to_string(), "Option<Box<BSTNode<T>>>");
    }

    #[test]
    fn test_type_display_multiple_params() {
        let t = Type::Parameterized {
            base: "HashMap".into(),
            params: vec![Type::Simple("K".into()), Type::Simple("V".into())],
        };
        assert_eq!(t.to_string(), "HashMap<K, V>");
    }

    #[test]
    fn test_type_default_is_empty_simple() {
        let t = Type::default();
        assert_eq!(t.to_string(), "");
    }

    #[test]
    fn test_spec_serde_roundtrip() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                complexity: Complexity {
                    time: Some("O(1)".into()),
                    space: Some("O(n)".into()),
                },
                tags: vec!["tag1".into()],
            },
            contracts: Contracts {
                invariants: vec!["invariant 1".into()],
            },
            structs: vec![StructDef {
                name: "Foo".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    constraints: vec!["Clone".into()],
                }],
                fields: vec![FieldDef {
                    name: "bar".into(),
                    field_type: Type::Simple("T".into()),
                }],
            }],
            methods: vec![MethodDef {
                name: "do_stuff".into(),
                params: vec![ParamDef {
                    name: "x".into(),
                    param_type: Type::Simple("i32".into()),
                }],
                returns: Some("bool".into()),
                preconditions: vec!["x > 0".into()],
                postconditions: vec!["result is valid".into()],
                injected_assertions: vec![],
            }],
            verification: Verification {
                test_cases: vec![TestCase {
                    name: "test_it".into(),
                    setup: Some("let f = Foo::new();".into()),
                    actions: vec!["f.do_stuff(1)".into()],
                    assertions: vec!["assert!(result)".into()],
                }],
            },
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let roundtrip: Spec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip.spec_version, "1.0");
        assert_eq!(roundtrip.metadata.name, "Test");
        assert_eq!(roundtrip.structs.len(), 1);
        assert_eq!(roundtrip.structs[0].fields[0].name, "bar");
        assert_eq!(roundtrip.methods[0].name, "do_stuff");
        assert_eq!(roundtrip.verification.test_cases[0].name, "test_it");
    }
}
