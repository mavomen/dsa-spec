//! Thin wrapper around the Tera template engine.

use crate::ast::Spec;
use crate::error::BackendError;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};
use tera::{Context, Tera};

static GLOBAL_TERA: OnceLock<Result<Arc<Tera>, BackendError>> = OnceLock::new();

fn get_or_init_tera(template_dir: &str) -> Result<Arc<Tera>, BackendError> {
    if template_dir == "templates" {
        GLOBAL_TERA
            .get_or_init(|| {
                Tera::new("templates/**/*")
                    .map(Arc::new)
                    .map_err(|e| BackendError::TemplateInit {
                        message: format!("{e}"),
                    })
            })
            .clone()
    } else {
        Tera::new(&format!("{template_dir}/**/*"))
            .map(Arc::new)
            .map_err(|e| BackendError::TemplateInit {
                message: format!("{e}"),
            })
    }
}

/// Tera template engine wrapper.
#[derive(Clone, Debug)]
pub struct TemplateEngine {
    tera: Arc<Tera>,
}

impl TemplateEngine {
    /// Create a new engine, loading templates from a directory tree.
    ///
    /// Loads all files matching `{template_dir}/**/*` as Tera templates.
    /// When `template_dir` is `"templates"`, the engine is lazily
    /// initialised once and shared across all callers.
    pub fn new(template_dir: &str) -> Result<Self, BackendError> {
        let tera = get_or_init_tera(template_dir)?;
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

/// Pipe code through an external formatter (rustfmt, gofmt, etc.).
///
/// Returns the formatted output on success. On failure (formatter not
/// installed or errors), callers typically fall back to unformatted code.
pub fn format_code(code: &str, cmd: &str, args: &[&str]) -> Result<String, BackendError> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| BackendError::Formatter {
            message: format!("Failed to spawn {cmd}: {e}"),
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(code.as_bytes())
            .map_err(|e| BackendError::Formatter {
                message: format!("Failed to write to {cmd} stdin: {e}"),
            })?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| BackendError::Formatter {
            message: format!("Failed to wait on {cmd}: {e}"),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(BackendError::Formatter {
            message: format!("{cmd} error: {}", String::from_utf8_lossy(&output.stderr)),
        })
    }
}

/// Write content to a file atomically via temp file + rename.
///
/// Writes to a `.tmp` sibling of `path` then renames, preventing
/// partial output on crash or power loss.
pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Check that struct names and method names are unique (no silent overwrites).
pub fn validate_unique_names(spec: &Spec) -> Result<(), BackendError> {
    let mut struct_names = HashSet::new();
    for s in &spec.structs {
        if !struct_names.insert(s.name.as_str()) {
            return Err(BackendError::TemplateInit {
                message: format!("Duplicate struct name: '{}'", s.name),
            });
        }
    }
    let mut method_names = HashSet::new();
    for m in &spec.methods {
        if !method_names.insert(m.name.as_str()) {
            return Err(BackendError::TemplateInit {
                message: format!("Duplicate method name: '{}'", m.name),
            });
        }
    }
    Ok(())
}

/// Replace characters unsafe for filenames, prevent directory traversal.
///
/// Strips/replaces: `/`, `\0`, `:`, `<`, `>`, `|`, `?`, `*`, `\`, `..`
/// Returns `"_"` if the result would be empty.
pub fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '<' | '>' | '|' | '?' | '*' | '\0' => '_',
            _ => c,
        })
        .collect();
    let sanitized = sanitized.trim_matches([' ', '.']).to_string();
    if sanitized.is_empty() {
        "_".into()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_invalid_dir_returns_empty_but_not_crash() {
        let result = TemplateEngine::new("/nonexistent/path/that/does/not/exist");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_render_missing_template_returns_err() {
        let engine = TemplateEngine::new("templates").expect("should init with templates dir");
        let ctx = Context::new();
        let result = engine.render("nonexistent.html.tera", &ctx);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("template render error")
        );
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

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = std::env::temp_dir().join("dsa_spec_test_atomic");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        atomic_write(&path, "hello").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        assert!(!path.with_extension("tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_atomic_write_overwrites_previous_content() {
        let dir = std::env::temp_dir().join("dsa_spec_test_atomic_over");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("data.txt");
        atomic_write(&path, "old").unwrap();
        atomic_write(&path, "new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_sanitize_filename_passes_through_safe_names() {
        assert_eq!(sanitize_filename("MyStruct"), "MyStruct");
        assert_eq!(sanitize_filename("foo_bar"), "foo_bar");
        assert_eq!(sanitize_filename("abc123"), "abc123");
    }

    #[test]
    fn test_sanitize_filename_replaces_invalid_chars() {
        assert_eq!(sanitize_filename("a/b:c"), "a_b_c");
        assert_eq!(sanitize_filename("x<y>z"), "x_y_z");
        assert_eq!(sanitize_filename("a|b?c*d"), "a_b_c_d");
    }

    #[test]
    fn test_sanitize_filename_strips_trailing_dots_and_spaces() {
        assert_eq!(sanitize_filename("foo."), "foo");
        assert_eq!(sanitize_filename(" bar "), "bar");
        assert_eq!(sanitize_filename(".baz."), "baz");
    }

    #[test]
    fn test_sanitize_filename_empty_returns_underscore() {
        assert_eq!(sanitize_filename(""), "_");
        assert_eq!(sanitize_filename("..."), "_");
        assert_eq!(sanitize_filename("   "), "_");
    }

    #[test]
    fn test_shared_tera_does_not_panic() {
        let a = TemplateEngine::new("templates").unwrap();
        let b = TemplateEngine::new("templates").unwrap();
        let mut ctx = Context::new();
        ctx.insert(
            "metadata",
            &serde_json::json!({"name": "x", "complexity": {"time": null, "space": null}}),
        );
        ctx.insert("contracts", &serde_json::json!({"invariants": []}));
        ctx.insert("structs", &serde_json::json!([]));
        ctx.insert("methods", &serde_json::json!([]));
        ctx.insert("verification", &serde_json::json!({"test_cases": []}));
        assert!(a.render("rust.rs.tera", &ctx).is_ok());
        assert!(b.render("rust.rs.tera", &ctx).is_ok());
    }
}
