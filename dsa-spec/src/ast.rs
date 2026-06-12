use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Spec {
    pub spec_version: String,
    pub metadata: Metadata,
    #[serde(default)]
    pub contracts: Contracts,
    #[serde(default)]
    pub structs: Vec<StructDef>,
    #[serde(default)]
    pub methods: Vec<MethodDef>,
    #[serde(default)]
    pub verification: Verification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub complexity: Complexity,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Complexity {
    pub time: Option<String>,
    pub space: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contracts {
    #[serde(default)]
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructDef {
    pub name: String,
    #[serde(default)]
    pub generics: Vec<GenericParam>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenericParam {
    pub name: String,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MethodDef {
    pub name: String,
    #[serde(default)]
    pub params: Vec<ParamDef>,
    #[serde(default)]
    pub returns: Option<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub postconditions: Vec<String>,
    #[serde(default)]
    pub injected_assertions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParamDef {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Verification {
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestCase {
    pub name: String,
    #[serde(default)]
    pub setup: Option<String>,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub assertions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Type {
    Simple(String),
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
