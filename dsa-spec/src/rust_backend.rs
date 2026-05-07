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
