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
    // ── required: engine & identity ────────────────────────────────

    fn engine(&self) -> &TemplateEngine;
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;

    // ── required: formatting ──────────────────────────────────────

    fn format_code(&self, code: &str) -> Result<String, BackendError>;

    // ── required: template paths ──────────────────────────────────

    fn monolithic_template(&self) -> &'static str;
    fn class_template(&self) -> &'static str;
    fn method_template(&self) -> &'static str;

    // ── required: filenames (no defaults — vary per backend) ──────

    fn monolithic_filename(&self, spec: &Spec) -> String;

    fn class_filename(&self, struct_name: &str) -> String {
        format!(
            "{}.{}",
            sanitize_filename(struct_name),
            self.file_extension()
        )
    }

    fn method_filename(&self, struct_name: &str, method_name: &str) -> String {
        format!(
            "{}_{}.{}",
            sanitize_filename(struct_name),
            sanitize_filename(method_name),
            self.file_extension()
        )
    }

    // ── required: context builders ────────────────────────────────

    fn build_monolithic_context(&self, spec: &Spec) -> Context;
    fn build_class_context(&self, spec: &Spec) -> Context;
    fn build_method_context(&self, spec: &Spec, method: &MethodDef) -> Context;

    // ── default generate() ────────────────────────────────────────

    /// Generate code from a spec.
    ///
    /// Returns a list of `(filename, source_code)` pairs. Each pair
    /// represents a separate output file (e.g. one per method for
    /// partial class backends).
    fn generate(&self, spec: &Spec) -> Result<Vec<(String, String)>, BackendError> {
        validate_unique_names(spec)?;
        if spec.structs.is_empty() {
            let ctx = self.build_monolithic_context(spec);
            let raw_code = self.engine().render(self.monolithic_template(), &ctx)?;
            let code = self.format_code(&raw_code).unwrap_or(raw_code);
            return Ok(vec![(self.monolithic_filename(spec), code)]);
        }

        let mut files = Vec::new();
        let s = spec.structs.first().unwrap();

        let class_ctx = self.build_class_context(spec);
        let raw = self.engine().render(self.class_template(), &class_ctx)?;
        let code = self.format_code(&raw).unwrap_or(raw);
        files.push((self.class_filename(&s.name), code));

        for m in &spec.methods {
            let method_ctx = self.build_method_context(spec, m);
            let raw = self.engine().render(self.method_template(), &method_ctx)?;
            let code = self.format_code(&raw).unwrap_or(raw);
            files.push((self.method_filename(&s.name, &m.name), code));
        }

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
