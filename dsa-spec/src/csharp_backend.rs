//! C# code generation backend with multi-file partial class output.

use crate::assertion;
use crate::ast::{MethodDef, Spec, Type};
use crate::backend::Backend;
use crate::casing;
use crate::context::{add_metadata_and_contracts, add_test_cases_translated};
use crate::error::BackendError;
use crate::template_engine::{TemplateEngine, sanitize_filename};
use serde::Serialize;
use tera::Context;

/// C# backend using Tera templates with optional dotnet format.
pub struct CSharpBackend {
    engine: TemplateEngine,
}

impl CSharpBackend {
    /// Create a new C# backend loading templates from the given directory.
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(CSharpBackend { engine })
    }

    /// Convert an AST type to a C# type string with nullable and collection translations.
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

    /// Translate a type name string to a C# type expression.
    /// `Option<T>` → `T?`, `Vec<T>` → `List<T>`, `i32` → `int`
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

    /// Return true if the type is a `Result<T, E>` (C# uses exceptions).
    pub(crate) fn is_result_type(typ: &Type) -> bool {
        match typ {
            Type::Simple(s) => s.starts_with("Result<"),
            Type::Parameterized { base, .. } => base == "Result",
        }
    }
}

impl Backend for CSharpBackend {
    fn engine(&self) -> &TemplateEngine {
        &self.engine
    }

    fn name(&self) -> &'static str {
        "csharp"
    }

    fn file_extension(&self) -> &'static str {
        "cs"
    }

    fn format_code(&self, _code: &str) -> Result<String, BackendError> {
        Err(BackendError::Formatter {
            message: "C# formatting skipped: dotnet format requires a project file".into(),
        })
    }

    fn monolithic_template(&self) -> &'static str {
        "csharp.cs.tera"
    }

    fn class_template(&self) -> &'static str {
        "csharp/class.cs.tera"
    }

    fn method_template(&self) -> &'static str {
        "csharp/method.cs.tera"
    }

    fn monolithic_filename(&self, spec: &Spec) -> String {
        format!("{}Methods.{}", spec.metadata.name, self.file_extension())
    }

    fn method_filename(&self, struct_name: &str, method_name: &str) -> String {
        format!(
            "{}.{}.{}",
            sanitize_filename(struct_name),
            casing::to_pascal_case(method_name),
            self.file_extension()
        )
    }

    fn build_monolithic_context(&self, spec: &Spec) -> Context {
        let mut ctx = Context::new();
        add_metadata_and_contracts(&mut ctx, spec);

        let structs: Vec<ClassStructContext> = spec
            .structs
            .iter()
            .map(|s| ClassStructContext {
                name: &s.name,
                fields: s
                    .fields
                    .iter()
                    .map(|f| FieldContext {
                        name: casing::to_pascal_case(&f.name),
                        csharp_type: CSharpBackend::to_csharp_type(&f.field_type),
                    })
                    .collect(),
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
            })
            .collect();
        ctx.insert("structs", &structs);

        let methods: Vec<MethodContext> = spec
            .methods
            .iter()
            .map(|m| {
                let return_type = m.returns.as_deref().map(|r| Type::Simple(r.to_string()));
                MethodContext {
                    name: casing::to_pascal_case(&m.name),
                    params: m
                        .params
                        .iter()
                        .map(|p| ParamContext {
                            name: casing::to_camel_case(&p.name),
                            csharp_type: CSharpBackend::to_csharp_type(&p.param_type),
                        })
                        .collect(),
                    returns: return_type.as_ref().map(CSharpBackend::to_csharp_type),
                    throws_exception: return_type
                        .as_ref()
                        .map(CSharpBackend::is_result_type)
                        .unwrap_or(false),
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
                            name: casing::to_pascal_case(&f.name),
                            csharp_type: CSharpBackend::to_csharp_type(&f.field_type),
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
                                format!("where {} : {}", g.name, g.constraints.join(", "))
                            },
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
                name: casing::to_pascal_case(&method.name),
                params: method
                    .params
                    .iter()
                    .map(|p| ParamContext {
                        name: casing::to_camel_case(&p.name),
                        csharp_type: CSharpBackend::to_csharp_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type.as_ref().map(CSharpBackend::to_csharp_type),
                throws_exception: return_type
                    .as_ref()
                    .map(CSharpBackend::is_result_type)
                    .unwrap_or(false),
                preconditions: &method.preconditions,
                postconditions: &method.postconditions,
                injected_assertions: &method.injected_assertions,
            },
        );

        add_test_cases_translated(&mut ctx, spec, translate_assertion);
        ctx
    }
}

/// Convert a Rust-style assertion string to C# `Assert.Xxx` syntax.
fn translate_assertion(a: &str) -> String {
    if let Some(expr) = assertion::parse_assert_bang(a) {
        format!("Assert.IsTrue({})", expr.trim())
    } else if let Some((left, right)) = assertion::parse_assert_eq(a) {
        format!("Assert.AreEqual({}, {})", left, right)
    } else {
        a.to_string()
    }
}

/// Template context for a C# class definition.
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

/// Template context for a C# class field/property.
#[derive(Serialize)]
struct FieldContext {
    name: String,
    csharp_type: String,
}

/// Template context for a C# method with nullable awareness.
#[derive(Serialize)]
struct MethodContext<'a> {
    name: String,
    params: Vec<ParamContext>,
    returns: Option<String>,
    throws_exception: bool,
    preconditions: &'a [String],
    postconditions: &'a [String],
    injected_assertions: &'a [String],
}

/// Template context for a C# method parameter.
#[derive(Serialize)]
struct ParamContext {
    name: String,
    csharp_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        Complexity, Contracts, FieldDef, Metadata, MethodDef, Spec, StructDef, Verification,
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

    fn spec_with_method() -> Spec {
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
                injected_assertions: vec![],
            }],
            verification: Verification::default(),
        }
    }

    #[test]
    fn test_generate_returns_class_file() {
        let spec = make_minimal_spec();
        let backend = CSharpBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        assert!(files.iter().any(|(n, _)| n == "Person.cs"));
    }

    #[test]
    fn test_generate_returns_method_file() {
        let spec = spec_with_method();
        let backend = CSharpBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        assert!(files.iter().any(|(n, _)| n == "TestClass.DoWork.cs"));
    }

    #[test]
    fn test_property_generation() {
        let spec = make_minimal_spec();
        let backend = CSharpBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        let class_file = files.iter().find(|(n, _)| n == "Person.cs").unwrap();
        assert!(class_file.1.contains("public string Name { get; set; }"));
        assert!(class_file.1.contains("public int Age { get; set; }"));
    }

    #[test]
    fn test_not_implemented_exception_stub() {
        let spec = spec_with_method();
        let backend = CSharpBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        let method_file = files
            .iter()
            .find(|(n, _)| n == "TestClass.DoWork.cs")
            .unwrap();
        assert!(
            method_file
                .1
                .contains("throw new NotImplementedException();")
        );
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
    fn test_translate_hashmap() {
        assert_eq!(
            CSharpBackend::translate_simple_type("HashMap<K,V>"),
            "Dictionary<K,V>"
        );
    }

    #[test]
    fn test_translate_reference() {
        assert_eq!(CSharpBackend::translate_simple_type("&T"), "T");
        assert_eq!(CSharpBackend::translate_simple_type("&mut [T]"), "T[]");
    }

    #[test]
    fn test_translate_primitives() {
        assert_eq!(CSharpBackend::translate_simple_type("usize"), "int");
        assert_eq!(CSharpBackend::translate_simple_type("i32"), "int");
        assert_eq!(CSharpBackend::translate_simple_type("bool"), "bool");
        assert_eq!(CSharpBackend::translate_simple_type("void"), "void");
    }

    #[test]
    fn test_translate_box_unwrapping() {
        assert_eq!(CSharpBackend::translate_simple_type("Box<T>"), "T");
        assert_eq!(
            CSharpBackend::translate_simple_type("Box<BSTNode<T>>"),
            "BSTNode<T>"
        );
    }

    #[test]
    fn test_translate_nested_types() {
        assert_eq!(
            CSharpBackend::translate_simple_type("Vec<Option<i32>>"),
            "List<int?>"
        );
        assert_eq!(
            CSharpBackend::translate_simple_type("Option<Box<Node<T>>>"),
            "Node<T>?"
        );
    }

    #[test]
    fn test_to_csharp_type_parameterized() {
        let typ = Type::Parameterized {
            base: "Dictionary".into(),
            params: vec![Type::Simple("K".into()), Type::Simple("V".into())],
        };
        assert_eq!(CSharpBackend::to_csharp_type(&typ), "Dictionary<K, V>");
    }

    #[test]
    fn test_contract_assertions_injected_in_csharp() {
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
                ..Default::default()
            }],
            methods: vec![MethodDef {
                name: "Bar".into(),
                preconditions: vec!["x > 0".into()],
                postconditions: vec!["result ok".into()],
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let injected = crate::contracts::inject_assertions(&spec);
        let backend = CSharpBackend::new("templates").unwrap();
        let files = backend.generate(&injected).unwrap();
        let method_file = files.iter().find(|(n, _)| n.contains("Foo.Bar")).unwrap();
        assert!(method_file.1.contains("// Contract: precondition: x > 0"));
        assert!(
            method_file
                .1
                .contains("System.Diagnostics.Debug.Assert(false, \"precondition: x > 0\");")
        );
        assert!(
            method_file
                .1
                .contains("// Contract: postcondition: result ok")
        );
    }
}
