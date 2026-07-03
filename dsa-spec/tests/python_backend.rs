use dsa_spec::ast::Spec;
use dsa_spec::backend::Backend;
use dsa_spec::python_backend::PythonBackend;

fn parse_spec(yaml: &str) -> Spec {
    serde_yml::from_str(yaml).expect("Failed to parse test spec")
}

fn generate(spec: &Spec) -> String {
    let backend = PythonBackend::new("templates").expect("Failed to create PythonBackend");
    let files = backend.generate(spec).expect("Generation failed");
    files
        .into_iter()
        .map(|(_, code)| code)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_basic_dataclass_generation() {
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
    assert!(code.contains("@dataclass"));
    assert!(code.contains("class Stack(Generic[T]):"));
    assert!(code.contains("items: List[T]"));
    assert!(code.contains("def push(self, item: T) -> None:"));
    assert!(code.contains("raise NotImplementedError"));
}

#[test]
fn test_option_translated_to_optional() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods:
  - name: "get"
    returns: "Option<i32>"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("-> Optional[int]"));
    assert!(!code.contains("Option["));
}

#[test]
fn test_result_becomes_type_with_exception_doc() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods:
  - name: "try_parse"
    returns: "Result<i32,String>"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("-> int:"));
    assert!(code.contains("Raises:"));
    assert!(!code.contains("Result["));
}

#[test]
fn test_generic_struct_with_constraints() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "MyStruct"
  category: "test"
structs:
  - name: "MyStruct"
    generics:
      - name: "T"
        constraints: ["Clone","Ord"]
    fields:
      - name: "value"
        type: "T"
methods: []
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(
        code.contains("T = TypeVar('T')") || code.contains("T = TypeVar(\"T\")"),
        "Expected T = TypeVar line, got:\n{}",
        code
    );
    assert!(code.contains("class MyStruct(Generic[T]):"));
    assert!(code.contains("value: T"));
}

#[test]
fn test_pytest_test_cases_generated() {
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
    - name: "simple_test"
      setup: "x = 1"
      actions: ["y = x + 1"]
      assertions: ["assert y == 2"]
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("def test_simple_test():"));
    assert!(code.contains("x = 1"));
    assert!(code.contains("y = x + 1"));
    assert!(code.contains("assert y == 2"));
    assert!(code.contains("pytest.main"));
}

#[test]
fn test_formatting_fallback_when_black_missing() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods: []
verification:
  test_cases: []
"#,
    );
    let backend = PythonBackend::new("templates").unwrap();
    let result = backend.generate(&spec);
    assert!(result.is_ok());
}

#[test]
fn test_edge_case_empty_spec_does_not_crash() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Empty"
  category: "test"
structs: []
methods: []
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(!code.is_empty()); // at least docstring and imports
}
