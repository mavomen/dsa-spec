use crate::ast::{Spec, Type};
use crate::backend::Backend;
use crate::template_engine::TemplateEngine;
use serde::Serialize;
use std::process::Command;
use tera::Context;

pub struct CSharpBackend {
    engine: TemplateEngine,
}

impl CSharpBackend {
    pub fn new(template_dir: &str) -> Result<Self, String> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(CSharpBackend { engine })
    }

    fn format_csharp(code: &str) -> Result<String, String> {
        let mut child = Command::new("dotnet")
            .arg("format")
            .arg("--no-restore")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn dotnet format: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(code.as_bytes())
                .map_err(|e| format!("Failed to write to dotnet format stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait on dotnet format: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // Fallback: return original code
            Err(format!(
                "dotnet format error: {} (falling back to unformatted)",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub(crate) fn to_csharp_type(typ: &Type) -> String {
        match typ {
            Type::Simple(s) => Self::translate_simple_type(s),
            Type::Parameterized { base, params } => {
                let cs_base = Self::translate_simple_type(base);
                let cs_params: Vec<String> = params.iter().map(Self::to_csharp_type).collect();
                format!("{}<{}>", cs_base, cs_params.join(", "))
            }
        }
    }

    pub(crate) fn translate_simple_type(s: &str) -> String {
        match s {
            "Option<T>" => "T?".to_string(),
            "Vec<T>" => "List<T>".to_string(),
            "HashMap<K,V>" => "Dictionary<K,V>".to_string(),
            "&T" => "T".to_string(),
            "&mut [T]" => "T[]".to_string(),
            "usize" => "int".to_string(),
            "i32" => "int".to_string(),
            "bool" => "bool".to_string(),
            "void" => "void".to_string(),
            s if s.starts_with("Option<") => {
                let inner = &s[7..s.len() - 1];
                format!("{}?", Self::translate_simple_type(inner))
            }
            s if s.starts_with("Vec<") => {
                let inner = &s[4..s.len() - 1];
                format!("List<{}>", Self::translate_simple_type(inner))
            }
            s if s.starts_with("Result<") => {
                let inner = &s[7..s.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
                if parts.len() == 2 {
                    Self::translate_simple_type(parts[0])
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
                    bounds: if g.constraints.is_empty() {
                        String::new()
                    } else {
                        format!("where {} : {}", g.name, g.constraints.join(", "))
                    },
                })
                .collect(),
            fields: s
                .fields
                .iter()
                .map(|f| FieldContext {
                    name: &f.name,
                    csharp_type: CSharpBackend::to_csharp_type(&f.field_type),
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
                        csharp_type: CSharpBackend::to_csharp_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type
                    .as_ref()
                    .map(|t| CSharpBackend::to_csharp_type(t)),
                throws_exception: return_type
                    .as_ref()
                    .map(|t| CSharpBackend::is_result_type(t))
                    .unwrap_or(false),
                preconditions: &m.preconditions,
                postconditions: &m.postconditions,
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

impl Backend for CSharpBackend {
    fn generate(&self, spec: &Spec) -> Result<String, String> {
        let context = build_context(spec);
        let raw_code = self.engine.render("csharp.cs.tera", &context)?;
        Ok(Self::format_csharp(&raw_code).unwrap_or(raw_code))
    }
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
    csharp_type: String,
}

#[derive(Serialize)]
struct MethodContext<'a> {
    name: &'a str,
    params: Vec<ParamContext<'a>>,
    returns: Option<String>,
    throws_exception: bool,
    preconditions: &'a [String],
    postconditions: &'a [String],
}

#[derive(Serialize)]
struct ParamContext<'a> {
    name: &'a str,
    csharp_type: String,
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
        Complexity, Contracts, FieldDef, Metadata, MethodDef, ParamDef, Spec, StructDef,
        Verification,
    };

    fn make_minimal_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "Person".into(),
                generics: vec![],
                fields: vec![
                    FieldDef {
                        name: "Name".into(),
                        field_type: Type::Simple("string".into()),
                    },
                    FieldDef {
                        name: "Age".into(),
                        field_type: Type::Simple("i32".into()),
                    },
                ],
            }],
            methods: vec![],
            verification: Verification::default(),
        }
    }

    #[test]
    fn test_option_to_nullable() {
        assert_eq!(
            CSharpBackend::to_csharp_type(&Type::Simple("Option<i32>".to_string())),
            "int?"
        );
        assert_eq!(
            CSharpBackend::to_csharp_type(&Type::Simple("Option<string>".to_string())),
            "string?"
        );
        assert_eq!(
            CSharpBackend::to_csharp_type(&Type::Simple("Option<T>".to_string())),
            "T?"
        );
    }

    #[test]
    fn test_vec_to_list() {
        assert_eq!(
            CSharpBackend::to_csharp_type(&Type::Simple("Vec<T>".to_string())),
            "List<T>"
        );
        assert_eq!(
            CSharpBackend::to_csharp_type(&Type::Simple("Vec<i32>".to_string())),
            "List<int>"
        );
    }

    #[test]
    fn test_property_generation() {
        let spec = make_minimal_spec();
        let backend = CSharpBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("public string Name { get; set; }"));
        assert!(code.contains("public int Age { get; set; }"));
    }

    #[test]
    fn test_not_implemented_exception_stub() {
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
                name: "TestClass".into(),
                generics: vec![],
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
        let backend = CSharpBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("throw new NotImplementedException();"));
    }
}
