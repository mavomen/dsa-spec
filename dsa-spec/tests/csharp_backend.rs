use dsa_spec::ast::Spec;
use dsa_spec::backend::Backend;
use dsa_spec::csharp_backend::CSharpBackend;

fn parse_spec(yaml: &str) -> Spec {
    serde_yaml::from_str(yaml).expect("Failed to parse test spec")
}

fn generate(spec: &Spec) -> String {
    let backend = CSharpBackend::new("templates").expect("Failed to create CSharpBackend");
    backend.generate(spec).expect("Generation failed")
}

#[test]
fn test_basic_class_generation() {
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
      - name: "_items"
        type: "Vec<T>"
methods:
  - name: "Push"
    params:
      - name: "item"
        type: "T"
    returns: "void"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("public class Stack<T>"));
    assert!(code.contains("public List<T> _items { get; set; }"));
    assert!(code.contains("public void Push(T item)"));
    assert!(code.contains("throw new NotImplementedException();"));
}

#[test]
fn test_nullable_reference_types() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods:
  - name: "GetValue"
    returns: "Option<string>"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    // Static class generates: public static string? GetValue()
    assert!(code.contains("string? GetValue()"));
}

#[test]
fn test_xunit_test_generation() {
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
    - name: "SimpleTest"
      setup: "var x = 1;"
      actions:
        - "x += 1"
      assertions:
        - "Assert.Equal(2, x)"
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("[Fact]"));
    assert!(code.contains("public void SimpleTest_Test()"));
    assert!(code.contains("var x = 1;"));
    assert!(code.contains("x += 1;"));
    assert!(code.contains("Assert.Equal(2, x);"));
}

#[test]
fn test_throws_exception_for_result_type() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods:
  - name: "TryParse"
    returns: "Result<i32,string>"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    // Static class generates: public static int TryParse()
    assert!(code.contains("int TryParse()"));
}

#[test]
fn test_nullable_int_is_properly_handled() {
    let spec = parse_spec(
        r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods:
  - name: "Get"
    returns: "Option<i32>"
verification:
  test_cases: []
"#,
    );
    let code = generate(&spec);
    assert!(code.contains("int? Get()"));
}

#[test]
fn test_formatting_fallback_when_dotnet_format_missing() {
    use dsa_spec::ast::{Metadata, Spec, Verification};
    let spec = Spec {
        spec_version: "1.0".into(),
        metadata: Metadata {
            name: "Test".into(),
            category: "test".into(),
            ..Default::default()
        },
        structs: vec![],
        methods: vec![],
        verification: Verification::default(),
        ..Default::default()
    };
    let backend = CSharpBackend::new("templates").unwrap();
    let result = backend.generate(&spec);
    assert!(result.is_ok(), "C# backend should fallback to raw code");
    assert!(!result.unwrap().is_empty());
}
