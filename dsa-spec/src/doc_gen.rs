use crate::ast::Spec;

/// Generates a markdown summary from a specification's metadata and contracts.
pub fn generate_doc(spec: &Spec) -> String {
    let mut doc = String::new();

    // Title
    doc.push_str(&format!("# {}\n\n", spec.metadata.name));
    doc.push_str(&format!("**Category:** {}\n\n", spec.metadata.category));

    // Complexity
    if let Some(time) = &spec.metadata.complexity.time {
        doc.push_str(&format!("- **Time complexity:** {time}\n"));
    }
    if let Some(space) = &spec.metadata.complexity.space {
        doc.push_str(&format!("- **Space complexity:** {space}\n"));
    }
    if spec.metadata.complexity.time.is_some() || spec.metadata.complexity.space.is_some() {
        doc.push('\n');
    }

    // Tags
    if !spec.metadata.tags.is_empty() {
        doc.push_str("**Tags:** ");
        let tags: Vec<&str> = spec.metadata.tags.iter().map(|s| s.as_str()).collect();
        doc.push_str(&tags.join(", "));
        doc.push_str("\n\n");
    }

    // Invariants
    if !spec.contracts.invariants.is_empty() {
        doc.push_str("## Invariants\n\n");
        for inv in &spec.contracts.invariants {
            doc.push_str(&format!("- {inv}\n"));
        }
        doc.push('\n');
    }

    // Structs
    if !spec.structs.is_empty() {
        doc.push_str("## Data Structures\n\n");
        for s in &spec.structs {
            doc.push_str(&format!("### `{}`\n\n", s.name));
            if !s.fields.is_empty() {
                doc.push_str("| Field | Type |\n|------|------|\n");
                for f in &s.fields {
                    doc.push_str(&format!("| `{}` | `{}` |\n", f.name, f.field_type));
                }
                doc.push('\n');
            }
        }
    }

    // Methods
    if !spec.methods.is_empty() {
        doc.push_str("## Methods\n\n");
        for m in &spec.methods {
            doc.push_str(&format!("### `{}`\n\n", m.name));
            if let Some(ret) = &m.returns {
                doc.push_str(&format!("**Returns:** `{ret}`\n\n"));
            }
            if !m.preconditions.is_empty() {
                doc.push_str("**Preconditions:**\n");
                for pre in &m.preconditions {
                    doc.push_str(&format!("- {pre}\n"));
                }
                doc.push('\n');
            }
            if !m.postconditions.is_empty() {
                doc.push_str("**Postconditions:**\n");
                for post in &m.postconditions {
                    doc.push_str(&format!("- {post}\n"));
                }
                doc.push('\n');
            }
        }
    }

    // Test cases
    if !spec.verification.test_cases.is_empty() {
        doc.push_str("## Test Cases\n\n");
        for tc in &spec.verification.test_cases {
            doc.push_str(&format!("### {}\n\n", tc.name));
            if let Some(setup) = &tc.setup {
                doc.push_str(&format!("**Setup:**\n```\n{setup}\n```\n\n"));
            }
            if !tc.actions.is_empty() {
                doc.push_str("**Actions:**\n");
                for action in &tc.actions {
                    doc.push_str(&format!("- `{action}`\n"));
                }
                doc.push('\n');
            }
            if !tc.assertions.is_empty() {
                doc.push_str("**Assertions:**\n");
                for assertion in &tc.assertions {
                    doc.push_str(&format!("- `{assertion}`\n"));
                }
                doc.push('\n');
            }
        }
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        Complexity, Contracts, FieldDef, Metadata, MethodDef, ParamDef, Spec, StructDef, TestCase,
        Type, Verification,
    };

    fn sample_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Stack".into(),
                category: "linear".into(),
                complexity: Complexity {
                    time: Some("O(1)".into()),
                    space: Some("O(n)".into()),
                },
                tags: vec!["lifo".into(), "generic".into()],
            },
            contracts: Contracts {
                invariants: vec!["size >= 0".into()],
            },
            structs: vec![StructDef {
                name: "Stack".into(),
                generics: vec![],
                fields: vec![FieldDef {
                    name: "items".into(),
                    field_type: Type::Simple("Vec<T>".into()),
                }],
            }],
            methods: vec![MethodDef {
                name: "push".into(),
                params: vec![ParamDef {
                    name: "item".into(),
                    param_type: Type::Simple("T".into()),
                }],
                returns: Some("void".into()),
                preconditions: vec!["stack not full".into()],
                postconditions: vec!["item on top".into()],
            }],
            verification: Verification {
                test_cases: vec![TestCase {
                    name: "push_works".into(),
                    setup: Some("let mut s = Stack::new();".into()),
                    actions: vec!["s.push(1)".into()],
                    assertions: vec!["assert_eq!(s.pop(), Some(1))".into()],
                }],
            },
        }
    }

    #[test]
    fn test_generates_markdown_title() {
        let doc = generate_doc(&sample_spec());
        assert!(doc.contains("# Stack"));
    }

    #[test]
    fn test_includes_complexity() {
        let doc = generate_doc(&sample_spec());
        assert!(doc.contains("O(1)"));
        assert!(doc.contains("O(n)"));
    }

    #[test]
    fn test_includes_invariants() {
        let doc = generate_doc(&sample_spec());
        assert!(doc.contains("size >= 0"));
    }

    #[test]
    fn test_includes_struct_fields() {
        let doc = generate_doc(&sample_spec());
        assert!(doc.contains("| `items` | `Vec<T>` |"));
    }

    #[test]
    fn test_includes_method_docs() {
        let doc = generate_doc(&sample_spec());
        assert!(doc.contains("### `push`"));
        assert!(doc.contains("**Returns:** `void`"));
        assert!(doc.contains("stack not full"));
        assert!(doc.contains("item on top"));
    }
}
