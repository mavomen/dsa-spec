use crate::ast::{Spec, Type};
use crate::backend::Backend;
use crate::template_engine::TemplateEngine;
use serde::Serialize;
use std::process::Command;
use tera::Context;

pub struct GoBackend {
    engine: TemplateEngine,
}

impl GoBackend {
    pub fn new(template_dir: &str) -> Result<Self, String> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(GoBackend { engine })
    }

    fn format_go(code: &str) -> Result<String, String> {
        let mut child = Command::new("gofmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn gofmt: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(code.as_bytes())
                .map_err(|e| format!("Failed to write to gofmt stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait on gofmt: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "gofmt error: {} (falling back to unformatted)",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub(crate) fn to_go_type(typ: &Type) -> String {
        match typ {
            Type::Simple(s) => Self::translate_simple_type(s),
            Type::Parameterized { base, params } => {
                let go_base = Self::translate_simple_type(base);
                let go_params: Vec<String> = params.iter().map(Self::to_go_type).collect();
                format!("{}[{}]", go_base, go_params.join(", "))
            }
        }
    }

    pub(crate) fn translate_simple_type(s: &str) -> String {
        match s {
            "Option<T>" => "*T".to_string(),
            "Vec<T>" => "[]T".to_string(),
            "HashMap<K,V>" => "map[K]V".to_string(),
            "&T" => "T".to_string(),
            "&mut [T]" => "[]T".to_string(),
            "usize" => "int".to_string(),
            "i32" => "int32".to_string(),
            "bool" => "bool".to_string(),
            "void" => "".to_string(),
            s if s.starts_with("Option<") => {
                let inner = &s[7..s.len() - 1];
                format!("*{}", Self::translate_simple_type(inner))
            }
            s if s.starts_with("Vec<") => {
                let inner = &s[4..s.len() - 1];
                format!("[]{}", Self::translate_simple_type(inner))
            }
            s if s.starts_with("Result<") => {
                let inner = &s[7..s.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
                if parts.len() == 2 {
                    format!("({}, error)", Self::translate_simple_type(parts[0]))
                } else {
                    s.to_string()
                }
            }
            s if s.starts_with("Box<") => {
                let inner = &s[4..s.len() - 1];
                Self::translate_simple_type(inner)
            }
            _ => s.to_string(),
        }
    }

    pub(crate) fn go_constraint(constraints: &[String]) -> String {
        if constraints.is_empty() {
            return "any".to_string();
        }
        if constraints.contains(&"Ord".to_string()) {
            return "constraints.Ordered".to_string();
        }
        if constraints.contains(&"comparable".to_string()) {
            return "comparable".to_string();
        }
        "any".to_string()
    }

    pub(crate) fn is_result_type(typ: &Type) -> bool {
        match typ {
            Type::Simple(s) => s.starts_with("Result<"),
            _ => false,
        }
    }
}

fn build_context(spec: &Spec) -> Context {
    let mut context = Context::new();

    let metadata = &spec.metadata;
    let pkg = metadata
        .name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase();

    context.insert(
        "metadata",
        &MetadataContext {
            name: metadata.name.clone(),
            complexity: ComplexityContext {
                time: metadata.complexity.time.clone(),
                space: metadata.complexity.space.clone(),
            },
            package_name: pkg,
        },
    );

    let contracts = &spec.contracts;
    context.insert(
        "contracts",
        &ContractsContext {
            invariants: contracts.invariants.clone(),
        },
    );

    let structs: Vec<StructContext> = spec
        .structs
        .iter()
        .map(|s| StructContext {
            name: s.name.clone(),
            generics: s
                .generics
                .iter()
                .map(|g| GenericParamContext {
                    name: g.name.clone(),
                    constraint: GoBackend::go_constraint(&g.constraints),
                })
                .collect(),
            fields: s
                .fields
                .iter()
                .map(|f| FieldContext {
                    name: f.name.clone(),
                    go_type: GoBackend::to_go_type(&f.field_type),
                })
                .collect(),
        })
        .collect();
    context.insert("structs", &structs);

    let methods: Vec<MethodContext> = spec
        .methods
        .iter()
        .map(|m| {
            let return_type = m.returns.as_deref().map(|r| Type::Simple(r.to_string()));
            MethodContext {
                name: m.name.clone(),
                params: m
                    .params
                    .iter()
                    .map(|p| ParamContext {
                        name: p.name.clone(),
                        go_type: GoBackend::to_go_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type.as_ref().map(|t| GoBackend::to_go_type(t)),
                returns_error: return_type
                    .as_ref()
                    .map(|t| GoBackend::is_result_type(t))
                    .unwrap_or(false),
                preconditions: m.preconditions.clone(),
                postconditions: m.postconditions.clone(),
            }
        })
        .collect();
    context.insert("methods", &methods);

    let tests: Vec<TestContext> = spec
        .verification
        .test_cases
        .iter()
        .map(|t| TestContext {
            name: t.name.clone(),
            setup: t.setup.clone(),
            actions: t.actions.clone(),
            assertions: t.assertions.clone(),
        })
        .collect();
    context.insert("verification", &VerificationContext { test_cases: tests });

    context
}

impl Backend for GoBackend {
    fn generate(&self, spec: &Spec) -> Result<String, String> {
        let context = build_context(spec);
        let raw_code = self.engine.render("go.go.tera", &context)?;
        Ok(Self::format_go(&raw_code).unwrap_or(raw_code))
    }
}

#[derive(Serialize)]
struct MetadataContext {
    name: String,
    complexity: ComplexityContext,
    package_name: String,
}

#[derive(Serialize)]
struct ComplexityContext {
    time: Option<String>,
    space: Option<String>,
}

#[derive(Serialize)]
struct ContractsContext {
    invariants: Vec<String>,
}

#[derive(Serialize)]
struct StructContext {
    name: String,
    generics: Vec<GenericParamContext>,
    fields: Vec<FieldContext>,
}

#[derive(Serialize)]
struct GenericParamContext {
    name: String,
    constraint: String,
}

#[derive(Serialize)]
struct FieldContext {
    name: String,
    go_type: String,
}

#[derive(Serialize)]
struct MethodContext {
    name: String,
    params: Vec<ParamContext>,
    returns: Option<String>,
    returns_error: bool,
    preconditions: Vec<String>,
    postconditions: Vec<String>,
}

#[derive(Serialize)]
struct ParamContext {
    name: String,
    go_type: String,
}

#[derive(Serialize)]
struct VerificationContext {
    test_cases: Vec<TestContext>,
}

#[derive(Serialize)]
struct TestContext {
    name: String,
    setup: Option<String>,
    actions: Vec<String>,
    assertions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        Complexity, Contracts, GenericParam, Metadata, MethodDef, Spec, StructDef, Verification,
    };

    #[test]
    fn test_go_constraint_comparable() {
        assert_eq!(
            GoBackend::go_constraint(&["comparable".into()]),
            "comparable"
        );
    }

    #[test]
    fn test_go_constraint_defaults_to_any() {
        assert_eq!(GoBackend::go_constraint(&[]), "any");
        assert_eq!(GoBackend::go_constraint(&["unknown".into()]), "any");
    }

    #[test]
    fn test_generated_struct_uses_generic_constraint() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    constraints: vec!["comparable".into()],
                }],
                fields: vec![],
            }],
            methods: vec![],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("type MyStruct[T comparable] struct {"));
    }

    #[test]
    fn test_result_to_tuple_error() {
        assert_eq!(
            GoBackend::to_go_type(&Type::Simple("Result<i32,String>".into())),
            "(int32, error)"
        );
        assert_eq!(
            GoBackend::to_go_type(&Type::Simple("Result<T,E>".into())),
            "(T, error)"
        );
    }

    #[test]
    fn test_generated_method_returns_tuple_error() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    constraints: vec![],
                }],
                fields: vec![],
            }],
            methods: vec![MethodDef {
                name: "DoWork".into(),
                params: vec![],
                returns: Some("Result<i32,String>".into()),
                preconditions: vec![],
                postconditions: vec![],
            }],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("func (s *MyStruct[T]) DoWork() (int32, error) {"));
    }

    #[test]
    fn test_option_to_pointer() {
        assert_eq!(
            GoBackend::to_go_type(&Type::Simple("Option<string>".into())),
            "*string"
        );
        assert_eq!(
            GoBackend::to_go_type(&Type::Simple("Option<T>".into())),
            "*T"
        );
    }

    #[test]
    fn test_method_stub_panics_not_implemented() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![GenericParam {
                    name: "T".into(),
                    constraints: vec![],
                }],
                fields: vec![],
            }],
            methods: vec![MethodDef {
                name: "DoWork".into(),
                params: vec![],
                returns: Some("void".into()),
                preconditions: vec![],
                postconditions: vec![],
            }],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("panic(\"not implemented\")"));
    }

    #[test]
    fn test_package_name_sanitization() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Binary Search Tree".into(),
                category: "trees".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("package binarysearchtree"));
    }
}
