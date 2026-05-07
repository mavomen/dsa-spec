use crate::go_backend::GoBackend;
use crate::backend::Backend;
use crate::ast::Spec;

fn parse_spec(yaml: &str) -> Spec {
    serde_yaml::from_str(yaml).expect("Failed to parse test spec")
}

fn generate(spec: &Spec) -> String {
    let backend = GoBackend::new("templates")
        .expect("Failed to create GoBackend");
    backend.generate(spec).expect("Generation failed")
}

#[test]
fn test_basic_struct_generation() {
    let spec = parse_spec(r#"
spec_version: "1.0"
metadata:
  name: "Stack"
  category: "linear"
structs:
  - name: "Stack"
    generics:
      - name: "T"
    fields:
      - name: "items"
        type: "Vec<T>"
methods:
  - name: "Push"
    params:
      - name: "item"
        type: "T"
    returns: "void"
verification:
  test_cases: []
"#);
    let code = generate(&spec);
    assert!(code.contains("package stack"));
    assert!(code.contains("type Stack[T any] struct {"));
    assert!(code.contains("items []T"));
    assert!(code.contains("func (s *Stack[T]) Push(item T) {"));
    assert!(code.contains("panic(\"not implemented\")"));
}

#[test]
fn test_interface_generation() {
    let spec = parse_spec(r#"
spec_version: "1.0"
metadata:
  name: "MyStruct"
  category: "test"
structs:
  - name: "MyStruct"
    fields: []
methods:
  - name: "Process"
    returns: "Result<i32,string>"
verification:
  test_cases: []
"#);
    let code = generate(&spec);
    assert!(code.contains("type MyStructInterface interface {"));
    assert!(code.contains("Process() (int32, error)"));
}

#[test]
fn test_test_generation() {
    let spec = parse_spec(r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods: []
verification:
  test_cases:
    - name: "SimpleTest"
      setup: "x := 1"
      actions:
        - "y := x + 1"
      assertions:
        - "if y != 2 { t.Errorf(\"expected 2, got %d\", y) }"
"#);
    let code = generate(&spec);
    assert!(code.contains("import \"testing\""));
    assert!(code.contains("func TestSimpleTest(t *testing.T) {"));
    assert!(code.contains("x := 1"));
    assert!(code.contains("y := x + 1"));
    assert!(code.contains("t.Errorf"));

    #[test]
    fn test_package_name_sanitization() {
        let spec = Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Binary Search Tree".into(),
                category: "trees".into(),
                complexity: Complexity::default(),
                tags: vec![],
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![],
            verification: Verification::default(),
        };
        let backend = GoBackend::new("templates").unwrap();
        let code = backend.generate(&spec).unwrap();
        // Should hyphenate or remove spaces; our sanitizer filters to alphanumeric only
        assert!(code.contains("package binarysearchtree"));
    }

}
