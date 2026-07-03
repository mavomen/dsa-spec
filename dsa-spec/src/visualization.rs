//! Visualization generators for Graphviz DOT, Mermaid class diagrams,
//! and Mermaid sequence diagrams.

use crate::ast::Spec;
use std::collections::HashSet;

/// Generate a visualization for a spec in the given format.
pub fn generate(spec: &Spec, format: &str) -> String {
    match format.to_lowercase().as_str() {
        "dot" | "graphviz" => generate_dot(spec),
        "mermaid" => generate_mermaid(spec),
        "sequence" => generate_sequence(spec),
        _ => generate_dot(spec),
    }
}

/// Generate a Graphviz DOT diagram for a spec's data structures.
pub fn generate_dot(spec: &Spec) -> String {
    let struct_names: HashSet<String> = spec.structs.iter().map(|s| s.name.clone()).collect();
    let mut dot = String::new();

    dot.push_str(&format!("digraph \"{}\" {{\n", spec.metadata.name));
    dot.push_str("    rankdir=TB;\n");
    dot.push_str("    ranksep=0.6;\n");
    dot.push_str("    nodesep=0.4;\n");
    dot.push_str("    node [shape=record, fontname=\"monospace\", fontsize=10];\n");
    dot.push_str("    edge [fontname=\"monospace\", fontsize=9, arrowsize=0.7];\n");
    dot.push_str("    splines=true;\n\n");

    if spec.structs.is_empty() {
        dot.push_str(&format!("    \"{}\" [label=\"{{", spec.metadata.name));
        for (i, m) in spec.methods.iter().enumerate() {
            if i > 0 {
                dot.push('|');
            }
            let params: Vec<String> = m
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.param_type))
                .collect();
            let sig = if params.is_empty() {
                format!("{}() -> {}", m.name, m.returns.as_deref().unwrap_or("()"))
            } else {
                format!(
                    "{}({}) -> {}",
                    m.name,
                    params.join(", "),
                    m.returns.as_deref().unwrap_or("()")
                )
            };
            dot.push_str(&format!("<{}> {}", m.name, sig));
        }
        dot.push_str("}\"];\n");
    } else {
        for s in &spec.structs {
            dot.push_str("    ");
            dot.push_str(&s.name);
            dot.push_str(" [label=\"{");
            for (i, f) in s.fields.iter().enumerate() {
                if i > 0 {
                    dot.push_str(" | ");
                }
                dot.push_str(&format!("<{}> {}: {}", f.name, f.name, f.field_type));
            }
            dot.push_str("}\"];\n");
        }
    }

    dot.push('\n');

    for s in &spec.structs {
        for f in &s.fields {
            let type_str = f.field_type.to_string();
            let targets = find_edge_targets(&type_str, &struct_names);
            for target in &targets {
                let sname = &s.name;
                let edge = if sname == target {
                    // Self-reference: use constraint=false to avoid forcing layout rank
                    format!(
                        "    {}:{} -> {} [label=\"{}\", dir=both, constraint=false];\n",
                        sname, f.name, target, f.name
                    )
                } else {
                    format!(
                        "    {}:{} -> {} [label=\"{}\"];\n",
                        sname, f.name, target, f.name
                    )
                };
                dot.push_str(&edge);
            }
        }
    }

    dot.push_str("}\n");
    dot
}

/// Generate a Mermaid class diagram for a spec.
pub fn generate_mermaid(spec: &Spec) -> String {
    let struct_names: HashSet<String> = spec.structs.iter().map(|s| s.name.clone()).collect();
    let mut mermaid = String::new();

    mermaid.push_str("classDiagram\n");

    if spec.structs.is_empty() {
        mermaid.push_str(&format!("    class {} {{\n", spec.metadata.name));
        for m in &spec.methods {
            let params: Vec<String> = m
                .params
                .iter()
                .map(|p| {
                    format!(
                        "{}: {}",
                        p.name,
                        escape_mermaid_type(&p.param_type.to_string())
                    )
                })
                .collect();
            let ret = m
                .returns
                .as_deref()
                .map(escape_mermaid_type)
                .unwrap_or_else(|| "void".into());
            let sig = if params.is_empty() {
                format!("+{}() {}", m.name, ret)
            } else {
                format!("+{}({}) {}", m.name, params.join(", "), ret)
            };
            mermaid.push_str(&format!("        {}\n", sig));
        }
        mermaid.push_str("    }\n");
    } else {
        for s in &spec.structs {
            mermaid.push_str(&format!("    class {} {{\n", s.name));
            for f in &s.fields {
                let escaped_type = escape_mermaid_type(&f.field_type.to_string());
                mermaid.push_str(&format!("        +{}: {}\n", f.name, escaped_type));
            }
            mermaid.push_str("    }\n");
        }
    }

    mermaid.push('\n');

    for s in &spec.structs {
        for f in &s.fields {
            let type_str = f.field_type.to_string();
            let targets = find_edge_targets(&type_str, &struct_names);
            for target in &targets {
                mermaid.push_str(&format!("    {} --> {} : {}\n", s.name, target, f.name));
            }
        }
    }

    mermaid.push('\n');
    mermaid
}

fn find_edge_targets(type_str: &str, struct_names: &HashSet<String>) -> Vec<String> {
    let tokens: Vec<&str> = type_str
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .collect();
    let mut result: Vec<String> = Vec::new();
    for name in struct_names {
        if tokens.contains(&name.as_str()) {
            result.push(name.clone());
        }
    }
    result
}

fn escape_mermaid_type(s: &str) -> String {
    s.replace(['<', '>'], "~").replace('&', "&amp;")
}

/// Generate a Mermaid sequence diagram from test cases.
///
/// Each test case is rendered as a separate sequenceDiagram fragment.
pub fn generate_sequence(spec: &Spec) -> String {
    let actor = &spec.metadata.name;
    let mut out = String::new();

    if spec.verification.test_cases.is_empty() {
        return out;
    }

    for tc in &spec.verification.test_cases {
        out.push_str("sequenceDiagram\n");
        out.push_str(&format!(
            "    participant Client\n    participant {actor}\n\n"
        ));

        if let Some(setup) = &tc.setup {
            out.push_str(&format!("    Note over {actor}: setup: {setup}\n"));
        }

        for action in &tc.actions {
            out.push_str(&format!("    Client->>{actor}: {action}\n"));
        }

        for assertion in &tc.assertions {
            out.push_str(&format!("    Note over {actor}: assert: {assertion}\n"));
        }

        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn make_spec_with_structs(
        name: &str,
        structs: Vec<StructDef>,
        methods: Vec<MethodDef>,
    ) -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: name.into(),
                category: "test".into(),
                ..Default::default()
            },
            structs,
            methods,
            ..Default::default()
        }
    }

    fn make_struct(name: &str, fields: Vec<FieldDef>) -> StructDef {
        StructDef {
            name: name.into(),
            fields,
            ..Default::default()
        }
    }

    fn field(name: &str, type_str: &str) -> FieldDef {
        FieldDef {
            name: name.into(),
            field_type: Type::Simple(type_str.into()),
        }
    }

    #[test]
    fn test_dot_empty_structs_shows_methods() {
        let spec = make_spec_with_structs(
            "Quicksort",
            vec![],
            vec![MethodDef {
                name: "quicksort".into(),
                params: vec![ParamDef {
                    name: "arr".into(),
                    param_type: Type::Simple("&mut [T]".into()),
                }],
                returns: Some("()".into()),
                ..Default::default()
            }],
        );
        let dot = generate_dot(&spec);
        assert!(dot.contains("digraph \"Quicksort\""));
        assert!(dot.contains("quicksort"));
        assert!(dot.contains("&mut [T]"));
    }

    #[test]
    fn test_dot_single_struct_no_edges() {
        let s = make_struct("DynamicArray", vec![field("len", "usize")]);
        let spec = make_spec_with_structs("DynamicArray", vec![s], vec![]);
        let dot = generate_dot(&spec);
        assert!(dot.contains("digraph \"DynamicArray\""));
        assert!(dot.contains("DynamicArray"));
        assert!(dot.contains("len: usize"));
        assert!(!dot.contains("->"));
    }

    #[test]
    fn test_dot_self_referencing_struct() {
        let s = make_struct(
            "Node",
            vec![field("value", "T"), field("next", "Option<Box<Node<T>>>")],
        );
        let spec = make_spec_with_structs("SinglyLinkedList", vec![s], vec![]);
        let dot = generate_dot(&spec);
        assert!(dot.contains("digraph \"SinglyLinkedList\""));
        assert!(dot.contains("Node"));
        assert!(dot.contains("Option<Box<Node<T>>>"));
        assert!(dot.contains("Node:next -> Node"));
        // Self-referencing edge includes label, dir=both, and constraint=false
        assert!(dot.contains(r#"label="next""#));
        assert!(dot.contains("dir=both"));
        assert!(dot.contains("constraint=false"));
    }

    #[test]
    fn test_dot_multiple_structs_with_edges() {
        let node = make_struct(
            "BSTNode",
            vec![
                field("value", "T"),
                field("left", "Option<Box<BSTNode<T>>>"),
                field("right", "Option<Box<BSTNode<T>>>"),
            ],
        );
        let tree = make_struct(
            "BinarySearchTree",
            vec![field("root", "Option<Box<BSTNode<T>>>")],
        );
        let spec = make_spec_with_structs("BinarySearchTree", vec![node, tree], vec![]);
        let dot = generate_dot(&spec);
        assert!(dot.contains("digraph \"BinarySearchTree\""));
        assert!(dot.contains("BSTNode"));
        assert!(dot.contains("BinarySearchTree"));
        // BSTNode self-edges with labels
        assert!(dot.contains("BSTNode:left -> BSTNode"));
        assert!(dot.contains("BSTNode:right -> BSTNode"));
        assert!(dot.contains(r#"label="left""#));
        assert!(dot.contains(r#"label="right""#));
        // Tree -> Node edge
        assert!(dot.contains("BinarySearchTree:root -> BSTNode"));
        assert!(dot.contains(r#"label="root""#));
    }

    #[test]
    fn test_dot_spec_begin_end() {
        let spec = make_spec_with_structs("Test", vec![], vec![]);
        let dot = generate_dot(&spec);
        assert!(dot.starts_with("digraph \"Test\""));
        assert!(dot.trim_end().ends_with('}'));
    }

    #[test]
    fn test_mermaid_empty_structs_shows_methods() {
        let spec = make_spec_with_structs(
            "Mergesort",
            vec![],
            vec![MethodDef {
                name: "mergesort".into(),
                params: vec![ParamDef {
                    name: "arr".into(),
                    param_type: Type::Simple("&mut [T]".into()),
                }],
                returns: Some("()".into()),
                ..Default::default()
            }],
        );
        let m = generate_mermaid(&spec);
        assert!(m.contains("classDiagram"));
        assert!(m.contains("class Mergesort"));
        assert!(m.contains("mergesort"));
    }

    #[test]
    fn test_mermaid_struct_class() {
        let s = make_struct(
            "BSTNode",
            vec![
                field("value", "T"),
                field("left", "Option<Box<BSTNode<T>>>"),
            ],
        );
        let spec = make_spec_with_structs("BST", vec![s], vec![]);
        let m = generate_mermaid(&spec);
        assert!(m.contains("classDiagram"));
        assert!(m.contains("class BSTNode"));
        assert!(m.contains("+value: T"));
        // Mermaid uses ~ instead of <>
        assert!(m.contains("Option~Box~BSTNode~T~~"));
        // Mermaid now shows relationship labels
        assert!(m.contains("BSTNode --> BSTNode : left"));
    }

    #[test]
    fn test_mermaid_relationship_between_structs() {
        let node = make_struct("Node", vec![field("value", "T")]);
        let list = make_struct(
            "SinglyLinkedList",
            vec![field("head", "Option<Box<Node<T>>>")],
        );
        let spec = make_spec_with_structs("List", vec![node, list], vec![]);
        let m = generate_mermaid(&spec);
        assert!(m.contains("SinglyLinkedList --> Node : head"));
    }

    #[test]
    fn test_mermaid_escapes_angle_brackets() {
        let s = make_struct("Wrapper", vec![field("data", "Vec<u8>")]);
        let spec = make_spec_with_structs("Test", vec![s], vec![]);
        let m = generate_mermaid(&spec);
        assert!(m.contains("Vec~u8~"));
        assert!(!m.contains("Vec<u8>"));
    }

    #[test]
    fn test_generate_defaults_to_dot() {
        let spec = make_spec_with_structs("Test", vec![], vec![]);
        let result = generate(&spec, "unknown_format");
        assert!(result.contains("digraph"));
        assert!(!result.contains("classDiagram"));
    }

    #[test]
    fn test_generate_dot_format() {
        let spec = make_spec_with_structs("Test", vec![], vec![]);
        let result = generate(&spec, "dot");
        assert!(result.contains("digraph"));
    }

    #[test]
    fn test_generate_graphviz_format() {
        let spec = make_spec_with_structs("Test", vec![], vec![]);
        let result = generate(&spec, "graphviz");
        assert!(result.contains("digraph"));
    }

    #[test]
    fn test_generate_mermaid_format() {
        let spec = make_spec_with_structs("Test", vec![], vec![]);
        let result = generate(&spec, "mermaid");
        assert!(result.contains("classDiagram"));
    }

    #[test]
    fn test_dot_doubly_linked_list() {
        let node = make_struct(
            "Node",
            vec![
                field("value", "T"),
                field("prev", "Option<Box<Node<T>>>"),
                field("next", "Option<Box<Node<T>>>"),
            ],
        );
        let list = make_struct(
            "DoublyLinkedList",
            vec![
                field("head", "Option<Box<Node<T>>>"),
                field("tail", "Option<Box<Node<T>>>"),
            ],
        );
        let spec = make_spec_with_structs("DoublyLinkedList", vec![node, list], vec![]);
        let dot = generate_dot(&spec);
        assert!(dot.contains("Node:prev -> Node"));
        assert!(dot.contains("Node:next -> Node"));
        assert!(dot.contains("DoublyLinkedList:head -> Node"));
        assert!(dot.contains("DoublyLinkedList:tail -> Node"));
        assert!(dot.contains(r#"label="prev""#));
        assert!(dot.contains(r#"label="next""#));
        assert!(dot.contains(r#"label="head""#));
        assert!(dot.contains(r#"label="tail""#));
    }

    #[test]
    fn test_sequence_format_dispatch() {
        use crate::ast::TestCase;
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Stack".into(),
                category: "linear".into(),
                ..Default::default()
            },
            structs: vec![],
            methods: vec![],
            contracts: Contracts::default(),
            verification: Verification {
                test_cases: vec![TestCase {
                    name: "test".into(),
                    setup: None,
                    actions: vec!["do_thing()".into()],
                    assertions: vec![],
                }],
            },
        };
        let result = generate(&spec, "sequence");
        assert!(result.contains("sequenceDiagram"));
    }

    #[test]
    fn test_sequence_with_actions_and_assertions() {
        use crate::ast::TestCase;
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Stack".into(),
                category: "linear".into(),
                ..Default::default()
            },
            structs: vec![],
            methods: vec![],
            contracts: Contracts::default(),
            verification: Verification {
                test_cases: vec![TestCase {
                    name: "push_pop".into(),
                    setup: Some("let mut s = Stack::new();".into()),
                    actions: vec!["s.push(1)".into(), "s.push(2)".into()],
                    assertions: vec!["assert_eq!(s.pop(), 2)".into()],
                }],
            },
        };
        let seq = generate_sequence(&spec);
        assert!(seq.contains("sequenceDiagram"));
        assert!(seq.contains("participant Stack"));
        assert!(seq.contains("Client->>Stack: s.push(1)"));
        assert!(seq.contains("Client->>Stack: s.push(2)"));
        assert!(seq.contains("Note over Stack: setup: let mut s = Stack::new()"));
        assert!(seq.contains("Note over Stack: assert: assert_eq!(s.pop(), 2)"));
    }

    #[test]
    fn test_sequence_empty_test_cases() {
        let spec = make_spec_with_structs("Empty", vec![], vec![]);
        let seq = generate_sequence(&spec);
        assert!(seq.is_empty());
    }
}
