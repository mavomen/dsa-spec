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
