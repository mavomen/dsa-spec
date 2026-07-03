//! TypeScript code generation backend with multi-file output.

use crate::assertion;
use crate::ast::{MethodDef, Spec, Type};
use crate::backend::Backend;
use crate::casing;
use crate::context::{add_metadata_and_contracts, add_test_cases_translated};
use crate::error::BackendError;
use crate::template_engine::{TemplateEngine, sanitize_filename};
use serde::Serialize;
use tera::Context;

/// TypeScript backend using Tera templates.
pub struct TypeScriptBackend {
    engine: TemplateEngine,
}

impl TypeScriptBackend {
    /// Create a new TypeScript backend loading templates from the given directory.
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let engine = TemplateEngine::new(template_dir)?;
        Ok(TypeScriptBackend { engine })
    }

    /// Convert an AST type to a TypeScript type string with union types.
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

    /// Translate a type name string to a TypeScript type expression.
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

    /// Return true if the type is a `Result<T, E>` (TypeScript uses exceptions).
    pub(crate) fn is_result_type(typ: &Type) -> bool {
        match typ {
            Type::Simple(s) => s.starts_with("Result<"),
            Type::Parameterized { base, .. } => base == "Result",
        }
    }
}

impl Backend for TypeScriptBackend {
    fn engine(&self) -> &TemplateEngine {
        &self.engine
    }

    fn name(&self) -> &'static str {
        "typescript"
    }

    fn file_extension(&self) -> &'static str {
        "ts"
    }

    fn format_code(&self, code: &str) -> Result<String, BackendError> {
        Ok(code.to_string())
    }

    fn monolithic_template(&self) -> &'static str {
        "typescript.ts.tera"
    }

    fn class_template(&self) -> &'static str {
        "typescript/class.ts.tera"
    }

    fn method_template(&self) -> &'static str {
        "typescript/method.ts.tera"
    }

    fn monolithic_filename(&self, spec: &Spec) -> String {
        format!("{}Methods.{}", spec.metadata.name, self.file_extension())
    }

    fn method_filename(&self, struct_name: &str, method_name: &str) -> String {
        format!(
            "{}_{}.{}",
            sanitize_filename(struct_name),
            casing::to_camel_case(method_name),
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
        ctx.insert("structs", &structs);

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
                            constraints: g.constraints.join(" & "),
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
                name: casing::to_camel_case(&method.name),
                params: method
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
                preconditions: &method.preconditions,
                postconditions: &method.postconditions,
                injected_assertions: &method.injected_assertions,
            },
        );

        add_test_cases_translated(&mut ctx, spec, translate_assertion);
        ctx
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

#[derive(Serialize)]
struct ClassStructContext<'a> {
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
        let files = backend.generate(&spec).unwrap();
        let class_file = files.iter().find(|(n, _)| n == "Stack.ts").unwrap();
        assert!(class_file.1.contains("export class Stack"));
        assert!(class_file.1.contains("items: T[]"));
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
        let files = backend.generate(&spec).unwrap();
        let method_file = files
            .iter()
            .find(|(n, _)| n == "TestClass_doWork.ts")
            .unwrap();
        assert!(
            method_file
                .1
                .contains("throw new Error('Not implemented');")
        );
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
    fn test_is_result_type_parameterized_returns_true() {
        let typ = Type::Parameterized {
            base: "Result".into(),
            params: vec![Type::Simple("T".into()), Type::Simple("E".into())],
        };
        assert!(TypeScriptBackend::is_result_type(&typ));
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
                ..Default::default()
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
        let files = backend.generate(&injected).unwrap();
        let method_file = files.iter().find(|(n, _)| n.contains("Foo_bar")).unwrap();
        assert!(method_file.1.contains("// Contract: precondition: x > 0"));
        assert!(
            method_file
                .1
                .contains("console.assert(false, \"precondition: x > 0\");")
        );
        assert!(
            method_file
                .1
                .contains("// Contract: postcondition: result ok")
        );
        assert!(method_file.1.contains("// Contract: invariant: size >= 0"));
    }
}
