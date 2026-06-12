use crate::assertion;
use crate::ast::{Spec, Type};
use crate::backend::Backend;
use crate::casing;
use crate::error::BackendError;
use crate::template_engine::TemplateEngine;
use serde::Serialize;
use tera::Context;

pub struct TypeScriptBackend {
    engine: TemplateEngine,
}

impl TypeScriptBackend {
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(TypeScriptBackend { engine })
    }

    pub(crate) fn to_typescript_type(typ: &Type) -> String {
        match typ {
            Type::Simple(s) => Self::translate_simple_type(s),
            Type::Parameterized { base, params } => {
                let ts_base = Self::translate_simple_type(base);
                let ts_params: Vec<String> = params.iter().map(Self::to_typescript_type).collect();
                format!("{}<{}>", ts_base, ts_params.join(", "))
            }
        }
    }

    pub(crate) fn translate_simple_type(s: &str) -> String {
        match s {
            "Option<T>" => "T | null".to_string(),
            "Vec<T>" => "T[]".to_string(),
            "HashMap<K,V>" => "Map<K, V>".to_string(),
            "&T" => "T".to_string(),
            "&mut [T]" => "T[]".to_string(),
            "usize" => "number".to_string(),
            "i32" => "number".to_string(),
            "bool" => "boolean".to_string(),
            "void" => "void".to_string(),
            s if s.starts_with("Option<") => {
                let inner = &s[7..s.len() - 1];
                format!("{} | null", Self::translate_simple_type(inner))
            }
            s if s.starts_with("Vec<") => {
                let inner = &s[4..s.len() - 1];
                let translated = Self::translate_simple_type(inner);
                if translated.contains('|') {
                    format!("({})[]", translated)
                } else {
                    format!("{}[]", translated)
                }
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

fn translate_assertion(a: &str) -> String {
    if let Some(expr) = assertion::parse_assert_bang(a) {
        format!("expect({}).toBe(true)", expr.trim())
    } else if let Some((left, right)) = assertion::parse_assert_eq(a) {
        format!("expect({}).toBe({})", left, right)
    } else {
        a.to_string()
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
                    constraints: g.constraints.join(" & "),
                })
                .collect(),
            fields: s
                .fields
                .iter()
                .map(|f| FieldContext {
                    name: casing::to_camel_case(&f.name),
                    ts_type: TypeScriptBackend::to_typescript_type(&f.field_type),
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
                name: casing::to_camel_case(&m.name),
                params: m
                    .params
                    .iter()
                    .map(|p| ParamContext {
                        name: casing::to_camel_case(&p.name),
                        ts_type: TypeScriptBackend::to_typescript_type(&p.param_type),
                    })
                    .collect(),
                returns: return_type
                    .as_ref()
                    .map(TypeScriptBackend::to_typescript_type),
                throws_exception: return_type
                    .as_ref()
                    .map(TypeScriptBackend::is_result_type)
                    .unwrap_or(false),
                preconditions: &m.preconditions,
                postconditions: &m.postconditions,
                injected_assertions: &m.injected_assertions,
            }
        })
        .collect();
    context.insert("methods", &methods);

    let translated_assertions: Vec<Vec<String>> = spec
        .verification
        .test_cases
        .iter()
        .map(|t| {
            t.assertions
                .iter()
                .map(|a| translate_assertion(a))
                .collect()
        })
        .collect();

    let tests: Vec<TestContext> = spec
        .verification
        .test_cases
        .iter()
        .enumerate()
        .map(|(i, t)| TestContext {
            name: &t.name,
            setup: t.setup.as_deref(),
            actions: &t.actions,
            assertions: &translated_assertions[i],
        })
        .collect();
    context.insert("verification", &VerificationContext { test_cases: tests });

    context
}

impl Backend for TypeScriptBackend {
    fn generate(&self, spec: &Spec) -> Result<String, BackendError> {
        let context = build_context(spec);
        self.engine.render("typescript.ts.tera", &context)
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
    fields: Vec<FieldContext>,
}

#[derive(Serialize)]
struct GenericParamContext<'a> {
    name: &'a str,
    constraints: String,
}

#[derive(Serialize)]
struct FieldContext {
    name: String,
    ts_type: String,
}

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

#[derive(Serialize)]
struct ParamContext {
    name: String,
    ts_type: String,
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
        Complexity, Contracts, FieldDef, Metadata, MethodDef, Spec, StructDef, Type, Verification,
    };

    fn make_basic_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Stack".into(),
                category: "linear".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![StructDef {
                name: "Stack".into(),
                generics: vec![],
                fields: vec![FieldDef {
                    name: "items".into(),
                    field_type: Type::Simple("Vec<T>".into()),
                }],
            }],
            methods: vec![],
            verification: Verification::default(),
        }
    }

    #[test]
    fn test_generates_interface_and_class() {
        let spec = make_basic_spec();
        let backend = TypeScriptBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("export interface Stack"));
        assert!(code.contains("export class StackImpl"));
        assert!(code.contains("items: T[]"));
    }

    #[test]
    fn test_option_to_union() {
        assert_eq!(
            TypeScriptBackend::to_typescript_type(&Type::Simple("Option<string>".into())),
            "string | null"
        );
        assert_eq!(
            TypeScriptBackend::to_typescript_type(&Type::Simple("Option<T>".into())),
            "T | null"
        );
    }

    #[test]
    fn test_method_stub_throws_error() {
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
                name: "doWork".into(),
                params: vec![],
                returns: Some("void".into()),
                preconditions: vec![],
                postconditions: vec![],
                injected_assertions: vec![],
            }],
            verification: Verification::default(),
        };
        let backend = TypeScriptBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        assert!(code.contains("throw new Error('Not implemented');"));
    }

    #[test]
    fn test_translate_hashmap() {
        assert_eq!(
            TypeScriptBackend::translate_simple_type("HashMap<K,V>"),
            "Map<K, V>"
        );
    }

    #[test]
    fn test_translate_reference() {
        assert_eq!(TypeScriptBackend::translate_simple_type("&T"), "T");
        assert_eq!(TypeScriptBackend::translate_simple_type("&mut [T]"), "T[]");
    }

    #[test]
    fn test_translate_primitives() {
        assert_eq!(TypeScriptBackend::translate_simple_type("usize"), "number");
        assert_eq!(TypeScriptBackend::translate_simple_type("i32"), "number");
        assert_eq!(TypeScriptBackend::translate_simple_type("bool"), "boolean");
        assert_eq!(TypeScriptBackend::translate_simple_type("void"), "void");
    }

    #[test]
    fn test_translate_box_unwrapping() {
        assert_eq!(TypeScriptBackend::translate_simple_type("Box<T>"), "T");
        assert_eq!(
            TypeScriptBackend::translate_simple_type("Box<BSTNode<T>>"),
            "BSTNode<T>"
        );
    }

    #[test]
    fn test_translate_result_type() {
        // Result<T,E> strips to T
        assert_eq!(
            TypeScriptBackend::translate_simple_type("Result<i32,String>"),
            "number"
        );
    }

    #[test]
    fn test_translate_nested_types() {
        assert_eq!(
            TypeScriptBackend::translate_simple_type("Vec<Option<string>>"),
            "(string | null)[]"
        );
        assert_eq!(
            TypeScriptBackend::translate_simple_type("Option<Box<Node<T>>>"),
            "Node<T> | null"
        );
    }

    #[test]
    fn test_to_typescript_type_parameterized() {
        let typ = Type::Parameterized {
            base: "Map".into(),
            params: vec![Type::Simple("K".into()), Type::Simple("V".into())],
        };
        assert_eq!(TypeScriptBackend::to_typescript_type(&typ), "Map<K, V>");
    }

    #[test]
    fn test_is_result_type_parameterized_returns_false() {
        let typ = Type::Parameterized {
            base: "Result".into(),
            params: vec![Type::Simple("T".into()), Type::Simple("E".into())],
        };
        assert!(!TypeScriptBackend::is_result_type(&typ));
    }

    #[test]
    fn test_translate_unknown_type_passthrough() {
        assert_eq!(TypeScriptBackend::translate_simple_type("MyType"), "MyType");
        assert_eq!(TypeScriptBackend::translate_simple_type(""), "");
    }

    #[test]
    fn test_contract_assertions_injected_in_typescript() {
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
            },
            structs: vec![StructDef {
                name: "Foo".into(),
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
        let backend = TypeScriptBackend::new("templates").unwrap();
        let code = backend.generate(&injected).unwrap();
        assert!(code.contains("// Contract: precondition: x > 0"));
        assert!(code.contains("console.assert(false, \"precondition: x > 0\");"));
        assert!(code.contains("// Contract: postcondition: result ok"));
        assert!(code.contains("// Contract: invariant: size >= 0"));
    }
}
