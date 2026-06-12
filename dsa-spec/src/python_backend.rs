use crate::ast::{Spec, Type};
use crate::backend::Backend;
use crate::error::BackendError;
use crate::template_engine::TemplateEngine;
use serde::Serialize;
use std::process::Command;
use tera::Context;

pub struct PythonBackend {
    engine: TemplateEngine,
}

impl PythonBackend {
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(PythonBackend { engine })
    }

    fn format_python(code: &str) -> Result<String, BackendError> {
        let mut child = Command::new("black")
            .arg("-c")
            .arg(code)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| BackendError::Formatter {
                message: format!("Failed to spawn black: {e}"),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(code.as_bytes())
                .map_err(|e| BackendError::Formatter {
                    message: format!("Failed to write to black stdin: {e}"),
                })?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| BackendError::Formatter {
                message: format!("Failed to wait on black: {e}"),
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(BackendError::Formatter {
                message: format!(
                    "black error: {} (falling back to unformatted)",
                    String::from_utf8_lossy(&output.stderr)
                ),
            })
        }
    }
}

impl Backend for PythonBackend {
    fn generate(&self, spec: &Spec) -> Result<String, BackendError> {
        let context = build_context(spec);
        let raw_code = self.engine.render("python.py.tera", &context)?;
        Ok(Self::format_python(&raw_code).unwrap_or(raw_code))
    }
}

pub(crate) fn to_python_type(typ: &Type) -> String {
    match typ {
        Type::Simple(s) => translate_simple_type(s),
        Type::Parameterized { base, params } => {
            let py_base = translate_simple_type(base);
            let py_params: Vec<String> = params.iter().map(to_python_type).collect();
            format!("{}[{}]", py_base, py_params.join(", "))
        }
    }
}

pub(crate) fn translate_simple_type(s: &str) -> String {
    match s {
        "Option<T>" => "Optional[T]".to_string(),
        "Vec<T>" => "List[T]".to_string(),
        "HashMap<K,V>" => "Dict[K, V]".to_string(),
        "&T" => "T".to_string(),
        "&mut [T]" => "List[T]".to_string(),
        "usize" => "int".to_string(),
        "i32" => "int".to_string(),
        "bool" => "bool".to_string(),
        "void" => "None".to_string(),
        s if s.starts_with("Option<") => {
            let inner = &s[7..s.len() - 1];
            format!("Optional[{}]", translate_simple_type(inner))
        }
        s if s.starts_with("Vec<") => {
            let inner = &s[4..s.len() - 1];
            format!("List[{}]", translate_simple_type(inner))
        }
        s if s.starts_with("HashMap<") => {
            let inner = &s[8..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 2 {
                format!(
                    "Dict[{}, {}]",
                    translate_simple_type(parts[0]),
                    translate_simple_type(parts[1])
                )
            } else {
                s.to_string()
            }
        }
        s if s.starts_with("Result<") => {
            let inner = &s[7..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 2 {
                translate_simple_type(parts[0])
            } else {
                s.to_string()
            }
        }
        s if s.starts_with("Box<") => {
            let inner = &s[4..s.len() - 1];
            translate_simple_type(inner)
        }
        _ => s.to_string(),
    }
}

pub(crate) fn is_result_type(typ: &Type) -> bool {
    match typ {
        Type::Simple(s) => s.starts_with("Result<"),
        _ => false,
    }
}

fn build_context(spec: &Spec) -> Context {
    let mut context = Context::new();

    let metadata = &spec.metadata;
    context.insert(
        "metadata",
        &MetadataContext {
            name: &metadata.name,
            complexity: ComplexityContext {
                time: metadata.complexity.time.as_deref(),
                space: metadata.complexity.space.as_deref(),
            },
        },
    );

    let contracts = &spec.contracts;
    context.insert(
        "contracts",
        &ContractsContext {
            invariants: &contracts.invariants,
        },
    );

    let structs: Vec<StructContext> = spec
        .structs
        .iter()
        .map(|s| StructContext {
            name: &s.name,
            generics: s
                .generics
                .iter()
                .map(|g| GenericParamContext {
                    name: &g.name,
                    bounds: g.constraints.join(", "),
                })
                .collect(),
            fields: s
                .fields
                .iter()
                .map(|f| FieldContext {
                    name: &f.name,
                    python_type: to_python_type(&f.field_type),
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
                name: &m.name,
                params: m
                    .params
                    .iter()
                    .map(|p| ParamContext {
                        name: &p.name,
                        python_type: to_python_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type.as_ref().map(to_python_type),
                raises_exception: return_type.as_ref().map(is_result_type).unwrap_or(false),
                preconditions: &m.preconditions,
                postconditions: &m.postconditions,
                injected_assertions: &m.injected_assertions,
            }
        })
        .collect();
    context.insert("methods", &methods);

    let tests: Vec<TestContext> = spec
        .verification
        .test_cases
        .iter()
        .map(|t| TestContext {
            name: &t.name,
            setup: t.setup.as_deref(),
            actions: &t.actions,
            assertions: &t.assertions,
        })
        .collect();
    context.insert("verification", &VerificationContext { test_cases: tests });

    context
}

#[derive(Serialize)]
struct MetadataContext<'a> {
    name: &'a str,
    complexity: ComplexityContext<'a>,
}

#[derive(Serialize)]
struct ComplexityContext<'a> {
    time: Option<&'a str>,
    space: Option<&'a str>,
}

#[derive(Serialize)]
struct ContractsContext<'a> {
    invariants: &'a [String],
}

#[derive(Serialize)]
struct StructContext<'a> {
    name: &'a str,
    generics: Vec<GenericParamContext<'a>>,
    fields: Vec<FieldContext<'a>>,
}

#[derive(Serialize)]
struct GenericParamContext<'a> {
    name: &'a str,
    bounds: String,
}

#[derive(Serialize)]
struct FieldContext<'a> {
    name: &'a str,
    python_type: String,
}

#[derive(Serialize)]
struct MethodContext<'a> {
    name: &'a str,
    params: Vec<ParamContext<'a>>,
    returns: Option<String>,
    raises_exception: bool,
    preconditions: &'a [String],
    postconditions: &'a [String],
    injected_assertions: &'a [String],
}

#[derive(Serialize)]
struct ParamContext<'a> {
    name: &'a str,
    python_type: String,
}

#[derive(Serialize)]
struct VerificationContext<'a> {
    test_cases: Vec<TestContext<'a>>,
}

#[derive(Serialize)]
struct TestContext<'a> {
    name: &'a str,
    setup: Option<&'a str>,
    actions: &'a [String],
    assertions: &'a [String],
}

#[cfg(test)]
mod python_type_tests {
    use super::*;
    use crate::ast::Type;

    #[test]
    fn test_translate_hashmap() {
        assert_eq!(translate_simple_type("HashMap<K,V>"), "Dict[K, V]");
    }

    #[test]
    fn test_translate_reference() {
        assert_eq!(translate_simple_type("&T"), "T");
        assert_eq!(translate_simple_type("&mut [T]"), "List[T]");
    }

    #[test]
    fn test_translate_primitives() {
        assert_eq!(translate_simple_type("usize"), "int");
        assert_eq!(translate_simple_type("i32"), "int");
        assert_eq!(translate_simple_type("bool"), "bool");
        assert_eq!(translate_simple_type("void"), "None");
    }

    #[test]
    fn test_translate_box_unwrapping() {
        assert_eq!(translate_simple_type("Box<T>"), "T");
        assert_eq!(translate_simple_type("Box<BSTNode<T>>"), "BSTNode<T>");
    }

    #[test]
    fn test_translate_nested_types() {
        assert_eq!(
            translate_simple_type("Vec<Option<i32>>"),
            "List[Optional[int]]"
        );
        assert_eq!(
            translate_simple_type("Option<Box<Node<T>>>"),
            "Optional[Node<T>]"
        );
    }

    #[test]
    fn test_to_python_type_parameterized() {
        let typ = Type::Parameterized {
            base: "Vec".into(),
            params: vec![Type::Parameterized {
                base: "Option".into(),
                params: vec![Type::Simple("i32".into())],
            }],
        };
        // Note: Parameterized base types like "Vec" don't match
        // the "Vec<T>" pattern; they pass through as-is via to_python_type
        assert_eq!(to_python_type(&typ), "Vec[Option[int]]");
    }

    #[test]
    fn test_translate_unknown_type_passthrough() {
        assert_eq!(translate_simple_type("CustomType"), "CustomType");
        assert_eq!(translate_simple_type(""), "");
    }

    #[test]
    fn test_is_result_type_with_parameterized_returns_false() {
        let typ = Type::Parameterized {
            base: "Result".into(),
            params: vec![Type::Simple("T".into()), Type::Simple("E".into())],
        };
        // is_result_type checks Type::Simple with starts_with("Result<")
        // Parameterized won't match that pattern
        assert!(!is_result_type(&typ));
    }

    #[test]
    fn test_contract_assertions_injected_in_python() {
        let spec = crate::ast::Spec {
            spec_version: "1.0".into(),
            metadata: crate::ast::Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: crate::ast::Contracts {
                invariants: vec!["size >= 0".into()],
            },
            structs: vec![],
            methods: vec![crate::ast::MethodDef {
                name: "bar".into(),
                preconditions: vec!["x > 0".into()],
                postconditions: vec!["result ok".into()],
                ..Default::default()
            }],
            verification: crate::ast::Verification::default(),
        };
        let injected = crate::contracts::inject_assertions(&spec);
        let code = super::build_context(&injected);
        // Verify template renders assertions
        let engine = crate::template_engine::TemplateEngine::new("templates").unwrap();
        let output = engine.render("python.py.tera", &code).unwrap();
        assert!(output.contains("# Contract: precondition: x > 0"));
        assert!(output.contains("assert False, \"precondition: x > 0\""));
        assert!(output.contains("# Contract: postcondition: result ok"));
        assert!(output.contains("# Contract: invariant: size >= 0"));
    }
}
