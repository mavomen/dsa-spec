//! Python code generation backend with multi-file output.

use crate::assertion;
use crate::ast::{MethodDef, Spec, Type};
use crate::backend::Backend;
use crate::context::{add_metadata_and_contracts, add_test_cases_translated};
use crate::error::BackendError;
use crate::template_engine::TemplateEngine;
use serde::Serialize;
use std::io::Write;
use std::process::Command;
use tera::Context;

/// Python backend using Tera templates with black formatting.
pub struct PythonBackend {
    engine: TemplateEngine,
}

impl PythonBackend {
    /// Create a new Python backend loading templates from the given directory.
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(PythonBackend { engine })
    }
}

impl Backend for PythonBackend {
    fn engine(&self) -> &TemplateEngine {
        &self.engine
    }

    fn name(&self) -> &'static str {
        "python"
    }

    fn file_extension(&self) -> &'static str {
        "py"
    }

    fn format_code(&self, code: &str) -> Result<String, BackendError> {
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

    fn monolithic_template(&self) -> &'static str {
        "python.py.tera"
    }

    fn class_template(&self) -> &'static str {
        "python/class.py.tera"
    }

    fn method_template(&self) -> &'static str {
        "python/method.py.tera"
    }

    fn monolithic_filename(&self, spec: &Spec) -> String {
        format!("{}Methods.{}", spec.metadata.name, self.file_extension())
    }

    fn build_monolithic_context(&self, spec: &Spec) -> Context {
        let mut ctx = Context::new();
        add_metadata_and_contracts(&mut ctx, spec);

        let structs: Vec<ClassStructContext> = spec
            .structs
            .iter()
            .map(|s| ClassStructContext {
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
                        name: f.name.clone(),
                        python_type: to_python_type(&f.field_type),
                    })
                    .collect(),
            })
            .collect();
        ctx.insert("structs", &structs);

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
                            name: p.name.clone(),
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
        ctx.insert("methods", &methods);

        add_test_cases_translated(&mut ctx, spec, translate_assertion);
        ctx
    }

    fn build_class_context(&self, spec: &Spec) -> Context {
        let mut ctx = Context::new();
        add_metadata_and_contracts(&mut ctx, spec);

        if let Some(s) = spec.structs.first() {
            ctx.insert(
                "struct",
                &ClassStructContext {
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
                            name: f.name.clone(),
                            python_type: to_python_type(&f.field_type),
                        })
                        .collect(),
                },
            );
        }

        ctx
    }

    fn build_method_context(&self, spec: &Spec, method: &MethodDef) -> Context {
        let mut ctx = Context::new();
        add_metadata_and_contracts(&mut ctx, spec);

        if let Some(s) = spec.structs.first() {
            ctx.insert(
                "struct",
                &ClassStructContext {
                    name: &s.name,
                    generics: s
                        .generics
                        .iter()
                        .map(|g| GenericParamContext {
                            name: &g.name,
                            bounds: g.constraints.join(", "),
                        })
                        .collect(),
                    fields: vec![],
                },
            );
        }

        let return_type = method
            .returns
            .as_deref()
            .map(|r| Type::Simple(r.to_string()));
        ctx.insert(
            "method",
            &MethodContext {
                name: &method.name,
                params: method
                    .params
                    .iter()
                    .map(|p| ParamContext {
                        name: p.name.clone(),
                        python_type: to_python_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type.as_ref().map(to_python_type),
                raises_exception: return_type.as_ref().map(is_result_type).unwrap_or(false),
                preconditions: &method.preconditions,
                postconditions: &method.postconditions,
                injected_assertions: &method.injected_assertions,
            },
        );

        add_test_cases_translated(&mut ctx, spec, translate_assertion);
        ctx
    }
}

/// Convert an AST type to a Python type string.
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

/// Translate a type name string to a Python type expression.
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

/// Return true if the type is a `Result<T, E>` (which Python handles via exceptions).
pub(crate) fn is_result_type(typ: &Type) -> bool {
    match typ {
        Type::Simple(s) => s.starts_with("Result<"),
        Type::Parameterized { base, .. } => base == "Result",
    }
}

/// Convert a Rust-style assertion string to Python `assert` syntax.
fn translate_assertion(a: &str) -> String {
    if let Some(expr) = assertion::parse_assert_bang(a) {
        format!("assert {}", expr.trim())
    } else if let Some((left, right)) = assertion::parse_assert_eq(a) {
        format!("assert {} == {}", left, right)
    } else {
        a.to_string()
    }
}

/// Template context for a Python class definition.
#[derive(Serialize)]
struct ClassStructContext<'a> {
    name: &'a str,
    generics: Vec<GenericParamContext<'a>>,
    fields: Vec<FieldContext>,
}

/// Template context for a generic type parameter.
#[derive(Serialize)]
struct GenericParamContext<'a> {
    name: &'a str,
    bounds: String,
}

/// Template context for a Python class field.
#[derive(Serialize)]
struct FieldContext {
    name: String,
    python_type: String,
}

/// Template context for a Python method with exception awareness.
#[derive(Serialize)]
struct MethodContext<'a> {
    name: &'a str,
    params: Vec<ParamContext>,
    returns: Option<String>,
    raises_exception: bool,
    preconditions: &'a [String],
    postconditions: &'a [String],
    injected_assertions: &'a [String],
}

/// Template context for a Python method parameter.
#[derive(Serialize)]
struct ParamContext {
    name: String,
    python_type: String,
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
        assert_eq!(to_python_type(&typ), "Vec[Option[int]]");
    }

    #[test]
    fn test_translate_unknown_type_passthrough() {
        assert_eq!(translate_simple_type("CustomType"), "CustomType");
        assert_eq!(translate_simple_type(""), "");
    }

    #[test]
    fn test_is_result_type_with_parameterized_returns_true() {
        let typ = Type::Parameterized {
            base: "Result".into(),
            params: vec![Type::Simple("T".into()), Type::Simple("E".into())],
        };
        assert!(is_result_type(&typ));
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
                ..Default::default()
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
        let backend = PythonBackend::new("templates").unwrap();
        let code = backend.build_monolithic_context(&injected);
        let engine = crate::template_engine::TemplateEngine::new("templates").unwrap();
        let output = engine.render("python.py.tera", &code).unwrap();
        assert!(output.contains("# Contract: precondition: x > 0"));
        assert!(output.contains("assert False, \"precondition: x > 0\""));
        assert!(output.contains("# Contract: postcondition: result ok"));
        assert!(output.contains("# Contract: invariant: size >= 0"));
    }
}
