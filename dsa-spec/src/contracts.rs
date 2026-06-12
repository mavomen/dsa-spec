use crate::ast::Spec;

/// Injects contract assertions into each method's `injected_assertions` field.
///
/// For each method, collects precondition, postcondition, and spec-level
/// invariant conditions as raw strings (language-agnostic). Backends
/// render these into language-specific assertion syntax when
/// `contracts_enabled` is set.
pub fn inject_assertions(spec: &Spec) -> Spec {
    let mut result = spec.clone();

    for method in &mut result.methods {
        let mut assertions = Vec::new();

        for pre in &method.preconditions {
            assertions.push(format!("precondition: {pre}"));
        }
        for post in &method.postconditions {
            assertions.push(format!("postcondition: {post}"));
        }
        for inv in &result.contracts.invariants {
            assertions.push(format!("invariant: {inv}"));
        }

        method.injected_assertions = assertions;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Contracts, Metadata, MethodDef, Spec, Verification};

    #[test]
    fn test_inject_assertions_no_contracts() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![MethodDef {
                name: "foo".into(),
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let result = inject_assertions(&spec);
        assert!(result.methods[0].injected_assertions.is_empty());
    }

    #[test]
    fn test_inject_assertions_preconditions() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![MethodDef {
                name: "foo".into(),
                preconditions: vec!["x > 0".into(), "x < 100".into()],
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let result = inject_assertions(&spec);
        let assertions = &result.methods[0].injected_assertions;
        assert_eq!(assertions.len(), 2);
        assert!(assertions[0].contains("x > 0"));
        assert!(assertions[1].contains("x < 100"));
    }

    #[test]
    fn test_inject_assertions_postconditions() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![MethodDef {
                name: "foo".into(),
                postconditions: vec!["result > 0".into()],
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let result = inject_assertions(&spec);
        let assertions = &result.methods[0].injected_assertions;
        assert_eq!(assertions.len(), 1);
        assert!(assertions[0].contains("result > 0"));
    }

    #[test]
    fn test_inject_assertions_invariants() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts {
                invariants: vec!["size <= capacity".into()],
            },
            structs: vec![],
            methods: vec![MethodDef {
                name: "foo".into(),
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let result = inject_assertions(&spec);
        let assertions = &result.methods[0].injected_assertions;
        assert_eq!(assertions.len(), 1);
        assert!(assertions[0].contains("size <= capacity"));
    }

    #[test]
    fn test_inject_assertions_all_three() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts {
                invariants: vec!["inv1".into()],
            },
            structs: vec![],
            methods: vec![MethodDef {
                name: "foo".into(),
                preconditions: vec!["pre1".into()],
                postconditions: vec!["post1".into()],
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let result = inject_assertions(&spec);
        let assertions = &result.methods[0].injected_assertions;
        assert_eq!(assertions.len(), 3);
        assert!(assertions[0].contains("pre1"));
        assert!(assertions[1].contains("post1"));
        assert!(assertions[2].contains("inv1"));
    }

    #[test]
    fn test_original_spec_not_mutated() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![MethodDef {
                name: "foo".into(),
                preconditions: vec!["x > 0".into()],
                ..Default::default()
            }],
            verification: Verification::default(),
        };
        let _ = inject_assertions(&spec);
        // Original should be unmodified
        assert!(spec.methods[0].injected_assertions.is_empty());
    }

    #[test]
    fn test_multiple_methods_each_get_assertions() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts {
                invariants: vec!["inv".into()],
            },
            structs: vec![],
            methods: vec![
                MethodDef {
                    name: "a".into(),
                    preconditions: vec!["pre_a".into()],
                    ..Default::default()
                },
                MethodDef {
                    name: "b".into(),
                    postconditions: vec!["post_b".into()],
                    ..Default::default()
                },
            ],
            verification: Verification::default(),
        };
        let result = inject_assertions(&spec);
        assert_eq!(result.methods[0].injected_assertions.len(), 2); // pre + inv
        assert_eq!(result.methods[1].injected_assertions.len(), 2); // post + inv
        assert!(result.methods[0].injected_assertions[0].contains("pre_a"));
        assert!(result.methods[1].injected_assertions[0].contains("post_b"));
    }
}
