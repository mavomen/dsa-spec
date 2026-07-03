//! Go code generation backend with multi-file output.

use crate::assertion;
use crate::ast::{MethodDef, Spec, Type};
use crate::backend::Backend;
use crate::casing;
use crate::error::BackendError;
use crate::template_engine::{
    TemplateEngine, format_code, sanitize_filename, validate_unique_names,
};
use serde::Serialize;
use tera::Context;

/// Go backend using Tera templates with gofmt formatting.
pub struct GoBackend {
    engine: TemplateEngine,
}

impl GoBackend {
    /// Create a new Go backend loading templates from the given directory.
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(GoBackend { engine })
    }

    fn file_extension() -> &'static str {
        "go"
    }

    fn class_filename(struct_name: &str) -> String {
        format!(
            "{}.{}",
            sanitize_filename(struct_name),
            Self::file_extension()
        )
    }

    fn method_filename(struct_name: &str, method_name: &str) -> String {
        format!(
            "{}_{}.{}",
            sanitize_filename(struct_name),
            sanitize_filename(method_name),
            Self::file_extension()
        )
    }

    fn format_go(code: &str) -> Result<String, BackendError> {
        format_code(code, "gofmt", &[])
    }

    /// Convert an AST type to a Go type string with pointer-based optionality.
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

    /// Translate a type name string to a Go type expression.
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

    /// Map spec constraint strings to Go generic constraints.
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
        if constraints.contains(&"Eq".to_string()) || constraints.contains(&"PartialEq".to_string())
        {
            return "comparable".to_string();
        }
        // Clone, Copy, Debug, Hash, Default have no Go equivalent
        "any".to_string()
    }

    /// Return true if the type is a `Result<T, E>` (Go uses `(T, error)` tuples).
    pub(crate) fn is_result_type(typ: &Type) -> bool {
        match typ {
            Type::Simple(s) => s.starts_with("Result<"),
            Type::Parameterized { base, .. } => base == "Result",
        }
    }

    fn build_monolithic_context(spec: &Spec) -> Context {
        let mut context = Context::new();

        let metadata = &spec.metadata;
        let pkg = Self::sanitize_package_name(&metadata.name);

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
                        bound: GoBackend::go_constraint(&g.constraints),
                    })
                    .collect(),
                fields: s
                    .fields
                    .iter()
                    .map(|f| FieldContext {
                        name: casing::to_pascal_case(&f.name),
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
                    name: casing::to_pascal_case(&m.name),
                    params: m
                        .params
                        .iter()
                        .map(|p| ParamContext {
                            name: casing::to_camel_case(&p.name),
                            go_type: GoBackend::to_go_type(&p.param_type),
                        })
                        .collect(),
                    returns: return_type.as_ref().map(GoBackend::to_go_type),
                    returns_error: return_type
                        .as_ref()
                        .map(GoBackend::is_result_type)
                        .unwrap_or(false),
                    preconditions: m.preconditions.clone(),
                    postconditions: m.postconditions.clone(),
                    injected_assertions: m.injected_assertions.clone(),
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
                assertions: t
                    .assertions
                    .iter()
                    .map(|a| translate_assertion(a))
                    .collect(),
            })
            .collect();
        context.insert("verification", &VerificationContext { test_cases: tests });

        context
    }

    fn sanitize_package_name(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_lowercase()
    }

    fn build_class_context(spec: &Spec) -> Context {
        let mut ctx = Context::new();

        let metadata = &spec.metadata;
        let pkg = Self::sanitize_package_name(&metadata.name);
        ctx.insert(
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

        ctx.insert(
            "contracts",
            &ContractsContext {
                invariants: spec.contracts.invariants.clone(),
            },
        );

        if let Some(s) = spec.structs.first() {
            ctx.insert(
                "struct",
                &StructContext {
                    name: s.name.clone(),
                    generics: s
                        .generics
                        .iter()
                        .map(|g| GenericParamContext {
                            name: g.name.clone(),
                            constraint: GoBackend::go_constraint(&g.constraints),
                            bound: GoBackend::go_constraint(&g.constraints),
                        })
                        .collect(),
                    fields: s
                        .fields
                        .iter()
                        .map(|f| FieldContext {
                            name: casing::to_pascal_case(&f.name),
                            go_type: GoBackend::to_go_type(&f.field_type),
                        })
                        .collect(),
                },
            );
        }

        ctx
    }

    fn build_method_context(spec: &Spec, method: &MethodDef) -> Context {
        let mut ctx = Context::new();

        let pkg = Self::sanitize_package_name(&spec.metadata.name);
        ctx.insert(
            "metadata",
            &MetadataContext {
                name: spec.metadata.name.clone(),
                complexity: ComplexityContext {
                    time: spec.metadata.complexity.time.clone(),
                    space: spec.metadata.complexity.space.clone(),
                },
                package_name: pkg,
            },
        );

        if let Some(s) = spec.structs.first() {
            ctx.insert(
                "struct",
                &StructContext {
                    name: s.name.clone(),
                    generics: s
                        .generics
                        .iter()
                        .map(|g| GenericParamContext {
                            name: g.name.clone(),
                            constraint: String::new(),
                            bound: String::new(),
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
                        go_type: GoBackend::to_go_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type.as_ref().map(GoBackend::to_go_type),
                returns_error: return_type
                    .as_ref()
                    .map(GoBackend::is_result_type)
                    .unwrap_or(false),
                preconditions: method.preconditions.clone(),
                postconditions: method.postconditions.clone(),
                injected_assertions: method.injected_assertions.clone(),
            },
        );

        let tests: Vec<TestContext> = spec
            .verification
            .test_cases
            .iter()
            .map(|t| TestContext {
                name: t.name.clone(),
                setup: t.setup.clone(),
                actions: t.actions.clone(),
                assertions: t
                    .assertions
                    .iter()
                    .map(|a| translate_assertion(a))
                    .collect(),
            })
            .collect();
        ctx.insert("verification", &VerificationContext { test_cases: tests });

        ctx
    }
}

fn translate_assertion(a: &str) -> String {
    if let Some(expr) = assertion::parse_assert_bang(a) {
        format!("if !({expr}) {{ t.Errorf(\"assertion failed: {expr}\") }}")
    } else if let Some((left, right)) = assertion::parse_assert_eq(a) {
        format!("if {left} != {right} {{ t.Errorf(\"got %v, want %v\", {left}, {right}) }}")
    } else {
        a.to_string()
    }
}

impl Backend for GoBackend {
    fn generate(&self, spec: &Spec) -> Result<Vec<(String, String)>, BackendError> {
        validate_unique_names(spec)?;
        if spec.structs.is_empty() {
            let ctx = Self::build_monolithic_context(spec);
            let raw_code = self.engine.render("go.go.tera", &ctx)?;
            let code = Self::format_go(&raw_code).unwrap_or(raw_code);
            return Ok(vec![(
                format!("{}Methods.{}", spec.metadata.name, Self::file_extension()),
                code,
            )]);
        }

        let mut files = Vec::new();
        let s = spec.structs.first().unwrap();

        let class_ctx = Self::build_class_context(spec);
        let raw = self.engine.render("go/class.go.tera", &class_ctx)?;
        let code = Self::format_go(&raw).unwrap_or(raw);
        files.push((Self::class_filename(&s.name), code));

        for m in &spec.methods {
            let method_ctx = Self::build_method_context(spec, m);
            let raw = self.engine.render("go/method.go.tera", &method_ctx)?;
            let code = Self::format_go(&raw).unwrap_or(raw);
            files.push((Self::method_filename(&s.name, &m.name), code));
        }

        Ok(files)
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
    bound: String,
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
    injected_assertions: Vec<String>,
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
        Complexity, Contracts, FieldDef, GenericParam, Metadata, MethodDef, ParamDef, Spec,
        StructDef, Type, Verification,
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
        let files = backend.generate(&spec).unwrap();
        let class_file = files.iter().find(|(n, _)| n == "MyStruct.go").unwrap();
        assert!(
            class_file
                .1
                .contains("type MyStruct[T comparable] struct {")
        );
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
                injected_assertions: vec![],
            }],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        let method_file = files
            .iter()
            .find(|(n, _)| n == "MyStruct_DoWork.go")
            .unwrap();
        assert!(
            method_file
                .1
                .contains("func (s *MyStruct[T]) DoWork() (int32, error) {")
        );
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
                injected_assertions: vec![],
            }],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        let method_file = files
            .iter()
            .find(|(n, _)| n == "MyStruct_DoWork.go")
            .unwrap();
        assert!(method_file.1.contains("panic(\"not implemented\")"));
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
        let files = backend.generate(&spec).unwrap();
        assert!(files[0].1.contains("package binarysearchtree"));
    }

    #[test]
    fn test_translate_hashmap() {
        assert_eq!(GoBackend::translate_simple_type("HashMap<K,V>"), "map[K]V");
    }

    #[test]
    fn test_translate_reference() {
        assert_eq!(GoBackend::translate_simple_type("&T"), "T");
        assert_eq!(GoBackend::translate_simple_type("&mut [T]"), "[]T");
    }

    #[test]
    fn test_translate_primitives() {
        assert_eq!(GoBackend::translate_simple_type("usize"), "int");
        assert_eq!(GoBackend::translate_simple_type("i32"), "int32");
        assert_eq!(GoBackend::translate_simple_type("bool"), "bool");
        assert_eq!(GoBackend::translate_simple_type("void"), "");
    }

    #[test]
    fn test_translate_box_unwrapping() {
        assert_eq!(GoBackend::translate_simple_type("Box<T>"), "T");
        assert_eq!(
            GoBackend::translate_simple_type("Box<BSTNode<T>>"),
            "BSTNode<T>"
        );
    }

    #[test]
    fn test_translate_nested_types() {
        assert_eq!(
            GoBackend::translate_simple_type("Vec<Option<string>>"),
            "[]*string"
        );
        assert_eq!(
            GoBackend::translate_simple_type("Option<Box<Node<T>>>"),
            "*Node<T>"
        );
    }

    #[test]
    fn test_to_go_type_parameterized() {
        let typ = Type::Parameterized {
            base: "map".into(),
            params: vec![Type::Simple("K".into()), Type::Simple("V".into())],
        };
        assert_eq!(GoBackend::to_go_type(&typ), "map[K, V]");
    }

    #[test]
    fn test_go_constraint_ord() {
        assert_eq!(
            GoBackend::go_constraint(&["Ord".into()]),
            "constraints.Ordered"
        );
    }

    #[test]
    fn test_is_result_type_parameterized_returns_true() {
        let typ = Type::Parameterized {
            base: "Result".into(),
            params: vec![Type::Simple("T".into()), Type::Simple("E".into())],
        };
        assert!(GoBackend::is_result_type(&typ));
    }

    #[test]
    fn test_translate_unknown_type_passthrough() {
        assert_eq!(GoBackend::translate_simple_type("MyType"), "MyType");
        assert_eq!(GoBackend::translate_simple_type(""), "");
    }

    #[test]
    fn test_go_field_names_are_pascal_case() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![],
                fields: vec![
                    FieldDef {
                        name: "first_name".into(),
                        field_type: Type::Simple("string".into()),
                    },
                    FieldDef {
                        name: "item_count".into(),
                        field_type: Type::Simple("int".into()),
                    },
                ],
            }],
            methods: vec![],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        let class_file = files.iter().find(|(n, _)| n == "MyStruct.go").unwrap();
        assert!(
            class_file.1.contains("FirstName string"),
            "expected PascalCase field, got: {}",
            class_file.1
        );
        assert!(
            class_file.1.contains("ItemCount int"),
            "expected PascalCase field, got: {}",
            class_file.1
        );
    }

    #[test]
    fn test_go_param_names_are_camel_case() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![],
                fields: vec![],
            }],
            methods: vec![MethodDef {
                name: "DoWork".into(),
                params: vec![
                    ParamDef {
                        name: "first_name".into(),
                        param_type: Type::Simple("string".into()),
                    },
                    ParamDef {
                        name: "item_count".into(),
                        param_type: Type::Simple("int".into()),
                    },
                ],
                returns: Some("void".into()),
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let files = backend.generate(&spec).unwrap();
        let method_file = files
            .iter()
            .find(|(n, _)| n == "MyStruct_DoWork.go")
            .unwrap();
        assert!(
            method_file.1.contains("firstName string"),
            "expected camelCase param, got: {}",
            method_file.1
        );
        assert!(
            method_file.1.contains("itemCount int"),
            "expected camelCase param, got: {}",
            method_file.1
        );
    }

    #[test]
    fn test_go_casing_with_contract_assertions() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "MyStruct".into(),
                generics: vec![],
                fields: vec![FieldDef {
                    name: "my_field".into(),
                    field_type: Type::Simple("int".into()),
                }],
            }],
            methods: vec![MethodDef {
                name: "DoStuff".into(),
                params: vec![ParamDef {
                    name: "my_param".into(),
                    param_type: Type::Simple("string".into()),
                }],
                returns: Some("bool".into()),
                preconditions: vec!["my_param != \"\"".into()],
                postconditions: vec!["result ok".into()],
                injected_assertions: vec!["precondition: my_param != \"\"".into()],
            }],
            verification: Verification::default(),
        };
        let injected = crate::contracts::inject_assertions(&spec);
        let backend = GoBackend::new("templates").unwrap();
        let files = backend.generate(&injected).unwrap();
        let class_file = files.iter().find(|(n, _)| n == "MyStruct.go").unwrap();
        assert!(
            class_file.1.contains("MyField int"),
            "field should be PascalCase"
        );
        let method_file = files
            .iter()
            .find(|(n, _)| n == "MyStruct_DoStuff.go")
            .unwrap();
        assert!(
            method_file.1.contains("myParam string"),
            "param should be camelCase"
        );
        assert!(
            method_file.1.contains("DoStuff"),
            "method should be PascalCase"
        );
    }

    #[test]
    fn test_contract_assertions_injected_in_go() {
        use crate::ast::{Contracts, Metadata, MethodDef, Spec, StructDef, Verification};
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
        let backend = GoBackend::new("templates").unwrap();
        let files = backend.generate(&injected).unwrap();
        let method_file = files.iter().find(|(n, _)| n == "Foo_Bar.go").unwrap();
        assert!(method_file.1.contains("// Contract: precondition: x > 0"));
        assert!(
            method_file
                .1
                .contains("// Contract: postcondition: result ok")
        );
        assert!(method_file.1.contains("// Contract: invariant: size >= 0"));
    }
}
