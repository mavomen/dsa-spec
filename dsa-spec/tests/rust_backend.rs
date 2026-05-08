use std::fs;
use std::process::Command;

fn create_temp_spec() -> tempfile::NamedTempFile {
    let spec_yaml = r#"
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
"#;
    let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    fs::write(file.path(), spec_yaml).expect("Failed to write temp spec");
    file
}

#[test]
fn test_rust_backend_generates_valid_syntax() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "rust",
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "generate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The generated Rust code should contain the struct and method stub
    assert!(stdout.contains("pub struct Stack<T>"));
    assert!(stdout.contains("fn push"));
    assert!(stdout.contains("item: T"));
    assert!(stdout.contains("todo!()"));
}

#[test]
fn test_rust_backend_handles_empty_structs() {
    let spec_yaml = r#"
spec_version: "1.0"
metadata:
  name: "EmptyStruct"
  category: "test"
structs:
  - name: "Empty"
    fields: []
methods: []
verification:
  test_cases: []
"#;
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), spec_yaml).unwrap();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            file.path().to_str().unwrap(),
            "--lang",
            "rust",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pub struct Empty"));
}
