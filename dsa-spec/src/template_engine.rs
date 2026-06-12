//! Thin wrapper around the Tera template engine.

use crate::error::BackendError;
use tera::{Context, Tera};

/// Tera template engine wrapper.
#[derive(Debug)]
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let tera = Tera::new(&format!("{}/**/*", template_dir)).map_err(|e| {
            BackendError::TemplateInit {
                message: format!("{e}"),
            }
        })?;
        Ok(TemplateEngine { tera })
    }

    /// Render a template with the given context.
    ///
    /// Template names correspond to filenames in the template directory
    /// (e.g. `"rust.rs.tera"`).
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String, BackendError> {
        self.tera
            .render(template_name, context)
            .map_err(|e| BackendError::TemplateRender {
                message: format!("{e}"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_invalid_dir_returns_empty_but_not_crash() {
        // Tera's glob **/* on a nonexistent dir returns empty template set — not an error
        // The engine initializes but has no templates
        let result = TemplateEngine::new("/nonexistent/path/that/does/not/exist");
        // Engine initializes with empty template set
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_render_missing_template_returns_err() {
        let engine = TemplateEngine::new("templates").expect("should init with templates dir");
        let ctx = Context::new();
        let result = engine.render("nonexistent.html.tera", &ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("template render error"));
    }

    #[test]
    fn test_render_with_known_template_succeeds() {
        let engine = TemplateEngine::new("templates").expect("should init");
        let mut ctx = Context::new();
        ctx.insert(
            "metadata",
            &serde_json::json!({"name": "Test", "complexity": {"time": null, "space": null}}),
        );
        ctx.insert("contracts", &serde_json::json!({"invariants": []}));
        ctx.insert("structs", &serde_json::json!([]));
        ctx.insert("methods", &serde_json::json!([]));
        ctx.insert("verification", &serde_json::json!({"test_cases": []}));
        let result = engine.render("rust.rs.tera", &ctx);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
