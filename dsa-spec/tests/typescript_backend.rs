use dsa_spec::ast::Spec;
use dsa_spec::backend::Backend;
use dsa_spec::typescript_backend::TypeScriptBackend;

fn parse_spec(yaml: &str) -> Spec {
    serde_yaml::from_str(yaml).expect("Failed to parse test spec")
}

fn generate(spec: &Spec) -> String {
    let backend = TypeScriptBackend::new("templates").expect("Failed to create TypeScriptBackend");
    backend.generate(spec).expect("Generation failed")
}

#[test]
fn test_interface_and_class_generation() {
    let spec = parse_spec(
        r#"
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
  - name: "push"
    params:
      - name: "item"
        type: "T"
    returns: "void"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("export interface Stack<T>"));
    assert!(code.contains("export class StackImpl<T>"));
    assert!(code.contains("items: T[]"));
    assert!(code.contains("push(item: T): void"));
    assert!(code.contains("throw new Error('Not implemented');"));
}

#[test]
fn test_jest_test_generation() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods: []
verification:
  test_cases:
    - name: "example"
      setup: "const x = 1;"
      actions:
        - "const y = x + 1"
      assertions:
        - "expect(y).toBe(2)"
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("describe('example', () => {"));
    assert!(code.contains("it('should work', () => {"));
    assert!(code.contains("const x = 1;"));
    assert!(code.contains("const y = x + 1;"));
    assert!(code.contains("expect(y).toBe(2);"));
}

#[test]
fn test_option_to_union_in_output() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods:
  - name: "find"
    returns: "Option<string>"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("string | null"));
}
