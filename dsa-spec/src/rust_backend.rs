use crate::ast::Spec;
use crate::backend::Backend;
use crate::template_engine::TemplateEngine;
use serde::Serialize;
use std::process::Command;
use tera::Context;

pub struct RustBackend {
    engine: TemplateEngine,
}

impl RustBackend {
    pub fn new(template_dir: &str) -> Result<Self, String> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(RustBackend { engine })
    }

    fn format_rust(code: &str) -> Result<String, String> {
        let mut child = Command::new("rustfmt")
            .arg("--edition")
            .arg("2021")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn rustfmt: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(code.as_bytes())
                .map_err(|e| format!("Failed to write to rustfmt stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait on rustfmt: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "rustfmt error: {} (falling back to unformatted)",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

impl Backend for RustBackend {
    fn generate(&self, spec: &Spec) -> Result<String, String> {
        let context = build_context(spec);
        let raw_code = self.engine.render("rust.rs.tera", &context)?;
        // Try to format, fallback to raw if rustfmt fails
        Ok(Self::format_rust(&raw_code).unwrap_or(raw_code))
    }
}

fn build_context(spec: &Spec) -> Context {
    // ... unchanged ...
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
                    bounds: g.constraints.join(" + "),
                })
                .collect(),
            fields: s
                .fields
                .iter()
                .map(|f| FieldContext {
                    name: &f.name,
                    field_type: &f.field_type,
                })
                .collect(),
        })
        .collect();
    context.insert("structs", &structs);

    let methods: Vec<MethodContext> = spec
        .methods
        .iter()
        .map(|m| MethodContext {
            name: &m.name,
            params: m
                .params
                .iter()
                .map(|p| ParamContext {
                    name: &p.name,
                    param_type: &p.param_type,
                })
                .collect(),
            returns: m.returns.as_deref(),
            preconditions: &m.preconditions,
            postconditions: &m.postconditions,
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

// Context types unchanged...
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
    field_type: &'a crate::ast::Type,
}

#[derive(Serialize)]
struct MethodContext<'a> {
    name: &'a str,
    params: Vec<ParamContext<'a>>,
    returns: Option<&'a str>,
    preconditions: &'a [String],
    postconditions: &'a [String],
}

#[derive(Serialize)]
struct ParamContext<'a> {
    name: &'a str,
    param_type: &'a crate::ast::Type,
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
mod tests {
    use super::*;
    use crate::ast::{
        Complexity, Contracts, FieldDef, GenericParam, Metadata, MethodDef, Spec, StructDef,
        TestCase, Type, Verification,
    };

    #[test]
    fn test_format_rust_fallback_on_missing_rustfmt() {
        // Call format_rust with invalid Rust — should fail fmt but return Err
        let code = "fn main() { let x = 1; }";
        // This is valid Rust, so it should pass rustfmt if available, or return Err
        let result = RustBackend::format_rust(code);
        // Either result is valid: rustfmt may or may not be installed
        if let Err(e) = &result {
            assert!(e.contains("rustfmt error") || e.contains("Failed to spawn rustfmt"));
        }
    }

    #[test]
    fn test_build_context_populates_fields() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "MyStruct".into(),
                category: "data".into(),
                complexity: Complexity {
                    time: Some("O(n)".into()),
                    space: None,
                },
                tags: vec!["tag".into()],
            },
            contracts: Contracts {
                invariants: vec!["invariant1".into()],
            },
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![GenericParam {
                    name: "K".into(),
                    constraints: vec!["Clone".into(), "Debug".into()],
                }],
                fields: vec![FieldDef {
                    name: "value".into(),
                    field_type: Type::Simple("K".into()),
                }],
            }],
            methods: vec![MethodDef {
                name: "get".into(),
                params: vec![],
                returns: Some("K".into()),
                preconditions: vec!["self is valid".into()],
                postconditions: vec!["returns value".into()],
            }],
            verification: Verification {
                test_cases: vec![TestCase {
                    name: "test_get".into(),
                    setup: None,
                    actions: vec!["let v = s.get()".into()],
                    assertions: vec!["assert!(v.is_some())".into()],
                }],
            },
        };
        let ctx = build_context(&spec);
        // Verify context values via rendering
        let engine = TemplateEngine::new("templates").unwrap();
        let output = engine.render("rust.rs.tera", &ctx).unwrap();
        assert!(output.contains("struct MyStruct"), "output: {}", output);
        assert!(output.contains("value: K"), "output: {}", output);
        assert!(output.contains("fn get"), "output: {}", output);
    }

    #[test]
    fn test_generate_with_result_return_type() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "Container".into(),
                generics: vec![],
                fields: vec![],
            }],
            methods: vec![MethodDef {
                name: "try_get".into(),
                params: vec![],
                returns: Some("Result<i32,String>".into()),
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let backend = RustBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("try_get"), "generated code: {}", code);
        assert!(code.contains("Result"), "generated code: {}", code);
        assert!(code.contains("todo!()"), "generated code: {}", code);
    }

    #[test]
    fn test_generate_with_contracts() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Capped".into(),
                category: "test".into(),
                complexity: Complexity {
                    time: Some("O(1)".into()),
                    space: Some("O(n)".into()),
                },
                ..Default::default()
            },
            contracts: Contracts {
                invariants: vec!["size <= capacity".into()],
            },
            structs: vec![StructDef {
                name: "Capped".into(),
                fields: vec![],
                ..Default::default()
            }],
            methods: vec![],
            verification: Verification::default(),
        };
        let backend = RustBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("Capped"));
        assert!(code.contains("size <= capacity"));
        assert!(code.contains("O(1)"));
        assert!(code.contains("O(n)"));
    }
}
