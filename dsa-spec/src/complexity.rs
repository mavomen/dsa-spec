use crate::ast::Spec;
use crate::error::SpecError;
use std::fs;
use std::path::Path;

/// Generates a markdown comparison table of complexity across specs.
pub fn generate_report(specs: &[Spec]) -> String {
    let mut lines = Vec::new();
    lines.push("| DSA | Category | Time Complexity | Space Complexity |".into());
    lines.push("|---|---|---|---|".into());

    let mut sorted: Vec<&Spec> = specs.iter().collect();
    sorted.sort_by(|a, b| {
        a.metadata
            .category
            .cmp(&b.metadata.category)
            .then(a.metadata.name.cmp(&b.metadata.name))
    });

    for spec in &sorted {
        let time = spec.metadata.complexity.time.as_deref().unwrap_or("N/A");
        let space = spec.metadata.complexity.space.as_deref().unwrap_or("N/A");
        lines.push(format!(
            "| {} | {} | {} | {} |",
            spec.metadata.name, spec.metadata.category, time, space
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

/// Generates a JSON report of complexity across specs.
pub fn generate_json_report(specs: &[Spec]) -> String {
    let mut entries: Vec<serde_json::Value> = Vec::new();
    for spec in specs {
        let entry = serde_json::json!({
            "name": spec.metadata.name,
            "category": spec.metadata.category,
            "time": spec.metadata.complexity.time,
            "space": spec.metadata.complexity.space,
        });
        entries.push(entry);
    }
    serde_json::to_string_pretty(&entries).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

/// Parse a Big-O string to a numeric ordinal for chart positioning.
/// Returns a value in [0.0, 1.0] where higher = slower/more memory.
fn complexity_to_ordinal(s: &str) -> Option<f64> {
    let s = s.trim().to_lowercase();
    if s.contains("2^n") || s.contains("2ⁿ") || s.contains("exponential") {
        return Some(0.95);
    }
    if s.contains("n^2") || s.contains("n²") || s.contains("quadratic") {
        return Some(0.8);
    }
    if s.contains("n log n") || s.contains("nlogn") {
        return Some(0.6);
    }
    if s.contains("v+e") || s.contains("v + e") {
        return Some(0.55);
    }
    if s.contains("v") && s.contains("e") {
        return Some(0.55);
    }
    if s.contains("(n)") || s.contains("linear") {
        return Some(0.4);
    }
    if s.contains("log n") || s.contains("logn") || s.contains("logarithmic") {
        return Some(0.2);
    }
    if s.contains("(1)") || s.contains("constant") {
        return Some(0.05);
    }
    None
}

/// Generates a Mermaid quadrant chart visualizing time vs space tradeoffs.
pub fn generate_tradeoff_chart(specs: &[Spec]) -> String {
    let mut lines = vec![
        "```mermaid".into(),
        "quadrantChart".into(),
        "    title Time vs Space Complexity".into(),
        r#"    x-axis "Fast Time" --> "Slow Time""#.into(),
        r#"    y-axis "Low Memory" --> "High Memory""#.into(),
        r#"    quadrant-1 "Ideal""#.into(),
        r#"    quadrant-2 "Memory Heavy""#.into(),
        r#"    quadrant-3 "Slow & Heavy""#.into(),
        r#"    quadrant-4 "Time Heavy""#.into(),
    ];

    for spec in specs {
        let time_ord = spec
            .metadata
            .complexity
            .time
            .as_deref()
            .and_then(complexity_to_ordinal);
        let space_ord = spec
            .metadata
            .complexity
            .space
            .as_deref()
            .and_then(complexity_to_ordinal);
        if let (Some(t), Some(s)) = (time_ord, space_ord) {
            lines.push(format!(r#"    "{}": [{t}, {s}]"#, spec.metadata.name));
        }
    }
    lines.push("```".into());
    lines.join("\n")
}

/// Load all spec YAML files from a directory.
pub fn load_specs_from_dir(dir: &str) -> Result<Vec<Spec>, Vec<SpecError>> {
    let dir_path = Path::new(dir);
    let mut specs = Vec::new();
    let mut errors = Vec::new();

    if !dir_path.is_dir() {
        return Err(vec![SpecError::IoError {
            message: format!("not a directory: {dir}"),
        }]);
    }

    let entries = match fs::read_dir(dir_path) {
        Ok(e) => e,
        Err(e) => {
            return Err(vec![SpecError::IoError {
                message: format!("failed to read directory {dir}: {e}"),
            }]);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(SpecError::IoError {
                    message: format!("failed to read entry: {e}"),
                });
                continue;
            }
        };
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            match fs::read_to_string(&path) {
                Ok(yaml) => match crate::parser::parse(&yaml) {
                    Ok(spec) => specs.push(spec),
                    Err(e) => errors.push(e),
                },
                Err(e) => {
                    errors.push(SpecError::IoError {
                        message: format!("failed to read {}: {e}", path.display()),
                    });
                }
            }
        }
    }

    if specs.is_empty() && !errors.is_empty() {
        return Err(errors);
    }
    Ok(specs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Complexity, Metadata, Spec};

    fn make_spec(name: &str, category: &str, time: Option<&str>, space: Option<&str>) -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: name.into(),
                category: category.into(),
                complexity: Complexity {
                    time: time.map(String::from),
                    space: space.map(String::from),
                },
                tags: vec![],
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_report_empty() {
        let report = generate_report(&[]);
        assert!(report.contains("| DSA | Category |"));
        assert!(report.contains("|---|---|---|"));
        // Header + separator (trailing newline doesn't add a line)
        assert_eq!(report.lines().count(), 2);
    }

    #[test]
    fn test_generate_report_single_spec() {
        let spec = make_spec("BST", "trees", Some("O(log n)"), Some("O(n)"));
        let report = generate_report(&[spec]);
        assert!(report.contains("| BST | trees | O(log n) | O(n) |"));
    }

    #[test]
    fn test_generate_report_multiple_categories() {
        let specs = vec![
            make_spec("Quicksort", "sorting", Some("O(n log n)"), Some("O(log n)")),
            make_spec("BST", "trees", Some("O(log n)"), Some("O(n)")),
        ];
        let report = generate_report(&specs);
        let bst_pos = report.find("BST").unwrap();
        let qsort_pos = report.find("Quicksort").unwrap();
        // Quicksort (sorting) should appear before BST (trees) alphabetically: 's' < 't'
        assert!(qsort_pos < bst_pos);
    }

    #[test]
    fn test_generate_report_missing_complexity() {
        let spec = make_spec("Unknown", "misc", None, None);
        let report = generate_report(&[spec]);
        assert!(report.contains("| Unknown | misc | N/A | N/A |"));
    }

    #[test]
    fn test_generate_json() {
        let specs = vec![make_spec("BST", "trees", Some("O(log n)"), Some("O(n)"))];
        let json = generate_json_report(&specs);
        assert!(json.contains("\"name\": \"BST\""));
        assert!(json.contains("\"time\": \"O(log n)\""));
        // A JSON array should start with [
        assert!(json.trim_start().starts_with('['));
    }

    #[test]
    fn test_complexity_to_ordinal_known_values() {
        assert_eq!(complexity_to_ordinal("O(1)"), Some(0.05));
        assert_eq!(complexity_to_ordinal("O(log n)"), Some(0.2));
        assert_eq!(complexity_to_ordinal("O(n)"), Some(0.4));
        assert_eq!(complexity_to_ordinal("O(n log n)"), Some(0.6));
        assert_eq!(complexity_to_ordinal("O(n^2)"), Some(0.8));
        assert_eq!(complexity_to_ordinal("O(2^n)"), Some(0.95));
        assert_eq!(complexity_to_ordinal("O(V+E)"), Some(0.55));
    }

    #[test]
    fn test_complexity_to_ordinal_unknown_returns_none() {
        assert_eq!(complexity_to_ordinal(""), None);
        assert_eq!(complexity_to_ordinal("unknown"), None);
        assert_eq!(complexity_to_ordinal("O(foo)"), None);
    }

    #[test]
    fn test_generate_chart_basic() {
        let specs = vec![make_spec("BST", "trees", Some("O(log n)"), Some("O(n)"))];
        let chart = generate_tradeoff_chart(&specs);
        assert!(chart.contains("quadrantChart"));
        assert!(chart.contains("BST"));
        // Should contain numeric coordinates
        assert!(chart.contains("0.2"));
        assert!(chart.contains("0.4"));
    }

    #[test]
    fn test_generate_chart_skips_unparseable() {
        let specs = vec![
            make_spec("Valid", "a", Some("O(1)"), Some("O(1)")),
            make_spec("SkipMe", "b", Some("???"), Some("???")),
        ];
        let chart = generate_tradeoff_chart(&specs);
        assert!(chart.contains("Valid"));
        assert!(!chart.contains("SkipMe"));
    }

    #[test]
    fn test_load_specs_from_dir_nonexistent() {
        let result = load_specs_from_dir("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_json_empty() {
        let json = generate_json_report(&[]);
        assert_eq!(json, "[]");
    }
}
