//! Rust code generation backend with multi-file output.

use crate::ast::{MethodDef, Spec};
use crate::backend::Backend;
use crate::context::add_metadata_and_contracts;
use crate::error::BackendError;
use crate::template_engine::{TemplateEngine, format_code, sanitize_filename};
use serde::Serialize;
use tera::Context;

/// Rust backend using Tera templates with rustfmt formatting.
pub struct RustBackend {
    engine: TemplateEngine,
}

impl RustBackend {
    /// Create a new Rust backend loading templates from the given directory.
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(RustBackend { engine })
    }
}

impl Backend for RustBackend {
    fn engine(&self) -> &TemplateEngine {
        &self.engine
    }

    fn name(&self) -> &'static str {
        "rust"
    }

    fn file_extension(&self) -> &'static str {
        "rs"
    }

    fn format_code(&self, code: &str) -> Result<String, BackendError> {
        format_code(code, "rustfmt", &["--edition", "2024"])
    }

    fn monolithic_template(&self) -> &'static str {
        "rust.rs.tera"
    }

    fn class_template(&self) -> &'static str {
        "rust/class.rs.tera"
    }

    fn method_template(&self) -> &'static str {
        "rust/method.rs.tera"
    }

    fn monolithic_filename(&self, spec: &Spec) -> String {
        format!("{}Methods.rs", sanitize_filename(&spec.metadata.name))
    }

    fn build_monolithic_context(&self, spec: &Spec) -> Context {
        let mut ctx = Context::new();
        add_metadata_and_contracts(&mut ctx, spec);

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
        ctx.insert("structs", &structs);

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
                injected_assertions: &m.injected_assertions,
            })
            .collect();
        ctx.insert("methods", &methods);

        crate::context::add_test_cases_raw(&mut ctx, spec);
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
                            bounds: if g.constraints.is_empty() {
                                String::new()
                            } else {
                                g.constraints.join(" + ")
                            },
                        })
                        .collect(),
                    fields: s
                        .fields
                        .iter()
                        .map(|f| ClassFieldContext {
                            name: f.name.clone(),
                            rust_type: f.field_type.to_string(),
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
                            bounds: if g.constraints.is_empty() {
                                String::new()
                            } else {
                                g.constraints.join(" + ")
                            },
                        })
                        .collect(),
                    fields: vec![],
                },
            );
        }

        ctx.insert(
            "method",
            &MethodFileContext {
                name: method.name.clone(),
                params: method
                    .params
                    .iter()
                    .map(|p| MethodParamContext {
                        name: p.name.clone(),
                        rust_type: p.param_type.to_string(),
                    })
                    .collect(),
                returns: method.returns.clone(),
                preconditions: &method.preconditions,
                postconditions: &method.postconditions,
                injected_assertions: &method.injected_assertions,
            },
        );

        crate::context::add_test_cases_raw(&mut ctx, spec);
        ctx
    }
}

// === Context structs for monolithic template ===

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
struct StructContext<'a> {
    name: &'a str,
    generics: Vec<GenericParamContext<'a>>,
    fields: Vec<FieldContext<'a>>,
}

#[derive(Serialize)]
struct ParamContext<'a> {
    name: &'a str,
    param_type: &'a crate::ast::Type,
}

#[derive(Serialize)]
struct MethodContext<'a> {
    name: &'a str,
    params: Vec<ParamContext<'a>>,
    returns: Option<&'a str>,
    preconditions: &'a [String],
    postconditions: &'a [String],
    injected_assertions: &'a [String],
}

// === Context structs for multi-file (class/method) templates ===

#[derive(Serialize)]
struct ClassFieldContext {
    name: String,
    rust_type: String,
}

#[derive(Serialize)]
struct ClassStructContext<'a> {
    name: &'a str,
    generics: Vec<GenericParamContext<'a>>,
    fields: Vec<ClassFieldContext>,
}

#[derive(Serialize)]
struct MethodParamContext {
    name: String,
    rust_type: String,
}

#[derive(Serialize)]
struct MethodFileContext<'a> {
    name: String,
    params: Vec<MethodParamContext>,
    returns: Option<String>,
    preconditions: &'a [String],
    postconditions: &'a [String],
    injected_assertions: &'a [String],
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
        let backend = RustBackend::new("templates").unwrap();
        let code = "fn main() { let x = 1; }";
        let result = backend.format_code(code);
        if let Err(e) = &result {
            let msg = e.to_string();
            assert!(msg.contains("rustfmt error") || msg.contains("Failed to spawn rustfmt"));
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
                ..Default::default()
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
                injected_assertions: vec![],
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
        let backend = RustBackend::new("templates").unwrap();
        let ctx = backend.build_monolithic_context(&spec);
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
        let files = backend.generate(&spec).unwrap();
        let method_file = files
            .iter()
            .find(|(n, _)| n == "Container_try_get.rs")
            .unwrap();
        assert!(
            method_file.1.contains("try_get"),
            "generated code: {}",
            method_file.1
        );
        assert!(
            method_file.1.contains("Result"),
            "generated code: {}",
            method_file.1
        );
        assert!(
            method_file.1.contains("todo!()"),
            "generated code: {}",
            method_file.1
        );
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
                ..Default::default()
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
        let files = backend.generate(&spec).unwrap();
        let class_file = files.iter().find(|(n, _)| n == "Capped.rs").unwrap();
        assert!(class_file.1.contains("Capped"));
        assert!(class_file.1.contains("size <= capacity"));
        assert!(class_file.1.contains("O(1)"));
        assert!(class_file.1.contains("O(n)"));
    }

    #[test]
    fn test_contract_assertions_injected_in_rust() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts {
                invariants: vec!["size >= 0".into()],
                ..Default::default()
            },
            structs: vec![StructDef {
                name: "Foo".into(),
                fields: vec![],
                ..Default::default()
            }],
            methods: vec![MethodDef {
                name: "bar".into(),
                preconditions: vec!["x > 0".into()],
                postconditions: vec!["result ok".into()],
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let injected = crate::contracts::inject_assertions(&spec);
        let backend = RustBackend::new("templates").unwrap();
        let files = backend.generate(&injected).unwrap();
        let method_file = files.iter().find(|(n, _)| n == "Foo_bar.rs").unwrap();
        assert!(method_file.1.contains("// Contract: precondition: x > 0"));
        assert!(
            method_file
                .1
                .contains("// Contract: postcondition: result ok")
        );
        assert!(method_file.1.contains("// Contract: invariant: size >= 0"));
    }
}
