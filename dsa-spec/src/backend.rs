//! Trait that all language backends must implement.

use crate::ast::{MethodDef, Spec};
use crate::error::BackendError;
use crate::template_engine::{TemplateEngine, sanitize_filename, validate_unique_names};
use tera::Context;

/// Interface for a language code generator.
///
/// Each backend reads a language-agnostic AST and produces idiomatic
/// source code for its target language. The output consists of one or
/// more `(filename, source_code)` pairs to support method-by-file
/// partitioning and partial classes.
///
/// The trait provides a default [`generate`](Backend::generate)
/// implementation that delegates the context-building, template-path,
/// filename and formatting decisions to the individual methods below.
/// Language backends override only what varies per language.
pub trait Backend {
    /// Reference to the shared template engine instance.
    fn engine(&self) -> &TemplateEngine;
    /// Short identifier for the target language. `"rust"`, `"go"`, etc.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    /// File extension including the leading dot. `"rs"`, `"py"`, `"go"`, ...
    fn file_extension(&self) -> &'static str;

    /// Format generated source code with the language's formatter.
    ///
    /// Returns the formatted string on success. Callers typically fall
    /// back to unformatted code when formatting fails.
    fn format_code(&self, code: &str) -> Result<String, BackendError>;

    /// Format all generated files in a single batch pass.
    ///
    /// The default implementation calls [`format_code`](Backend::format_code)
    /// on each file individually. Backends whose formatter supports
    /// multi-file input (e.g. rustfmt) should override this to spawn
    /// the formatter once for all files.
    fn format_all(&self, files: &mut [(String, String)]) -> Result<(), BackendError> {
        for (_, code) in files.iter_mut() {
            if let Ok(formatted) = self.format_code(code) {
                *code = formatted;
            }
        }
        Ok(())
    }

    /// Template filename for the monolithic (no-struct) output variant.
    /// `"rust.rs.tera"`, `"python.py.tera"`, ...
    fn monolithic_template(&self) -> &'static str;
    /// Template filename for the class definition output.
    /// `"rust/class.rs.tera"`, `"go/class.go.tera"`, ...
    fn class_template(&self) -> &'static str;
    /// Template filename for the per-method output.
    /// `"rust/method.rs.tera"`, `"typescript/method.ts.tera"`, ...
    fn method_template(&self) -> &'static str;

    /// Output filename for the monolithic variant.
    /// E.g. `"StackMethods.rs"`, `"StackMethods.py"`.
    fn monolithic_filename(&self, spec: &Spec) -> String;

    /// Output filename for a class definition.
    ///
    /// Defaults to `"{struct_name}.{ext}"`. Override for backends that
    /// need a different convention (e.g. C# partial classes).
    fn class_filename(&self, struct_name: &str) -> String {
        format!(
            "{}.{}",
            sanitize_filename(struct_name),
            self.file_extension()
        )
    }

    /// Output filename for a per-method file.
    ///
    /// Defaults to `"{struct_name}_{method_name}.{ext}"`. Override for
    /// backends that use a different scheme (e.g. C#'s `Struct.Method.cs`).
    fn method_filename(&self, struct_name: &str, method_name: &str) -> String {
        format!(
            "{}_{}.{}",
            sanitize_filename(struct_name),
            sanitize_filename(method_name),
            self.file_extension()
        )
    }

    /// Build the Tera context for the monolithic template.
    fn build_monolithic_context(&self, spec: &Spec) -> Context;
    /// Build the Tera context for the class template.
    fn build_class_context(&self, spec: &Spec) -> Context;
    /// Build the Tera context for a single-method template.
    fn build_method_context(&self, spec: &Spec, method: &MethodDef) -> Context;

    /// Generate code from a spec.
    ///
    /// Returns a list of `(filename, source_code)` pairs. Each pair
    /// represents a separate output file (e.g. one per method for
    /// partial class backends).
    fn generate(&self, spec: &Spec) -> Result<Vec<(String, String)>, BackendError> {
        validate_unique_names(spec)?;
        let mut files: Vec<(String, String)> = if spec.structs.is_empty() {
            let ctx = self.build_monolithic_context(spec);
            let raw_code = self.engine().render(self.monolithic_template(), &ctx)?;
            vec![(self.monolithic_filename(spec), raw_code)]
        } else {
            let s = spec.structs.first().unwrap();
            let class_ctx = self.build_class_context(spec);
            let raw = self.engine().render(self.class_template(), &class_ctx)?;
            let mut result = vec![(self.class_filename(&s.name), raw)];
            for m in &spec.methods {
                let method_ctx = self.build_method_context(spec, m);
                let raw = self.engine().render(self.method_template(), &method_ctx)?;
                result.push((self.method_filename(&s.name, &m.name), raw));
            }
            result
        };

        self.format_all(&mut files)?;
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Contracts, Metadata, Spec, Verification};
    use crate::csharp_backend::CSharpBackend;
    use crate::go_backend::GoBackend;
    use crate::python_backend::PythonBackend;
    use crate::rust_backend::RustBackend;
    use crate::typescript_backend::TypeScriptBackend;

    fn minimal_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Minimal".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![],
            verification: Verification::default(),
        }
    }

    #[test]
    fn test_all_backends_implement_trait() {
        let spec = minimal_spec();
        let backends: Vec<(&str, Box<dyn Backend>)> = vec![
            ("rust", Box::new(RustBackend::new("templates").unwrap())),
            ("python", Box::new(PythonBackend::new("templates").unwrap())),
            ("csharp", Box::new(CSharpBackend::new("templates").unwrap())),
            (
                "typescript",
                Box::new(TypeScriptBackend::new("templates").unwrap()),
            ),
            ("go", Box::new(GoBackend::new("templates").unwrap())),
        ];

        for (name, backend) in &backends {
            let result = backend.generate(&spec);
            assert!(
                result.is_ok(),
                "Backend {} failed: {}",
                name,
                result.unwrap_err()
            );
            let files = result.unwrap();
            assert!(
                !files.is_empty(),
                "Backend {} produced empty file list",
                name
            );
            for (filename, code) in &files {
                assert!(
                    !code.is_empty(),
                    "Backend {} produced empty code for {}",
                    name,
                    filename
                );
            }
        }
    }
}
