use std::fs;
use std::process::Command;

fn create_temp_spec() -> tempfile::NamedTempFile {
    let spec_yaml = r#"
spec_version: "1.0"
metadata:
  name: "Test"
  category: "test"
structs: []
methods: []
verification:
  test_cases: []
"#;
    let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    fs::write(file.path(), spec_yaml).expect("Failed to write temp spec");
    file
}

#[test]
fn test_generate_command_exists() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args(["run", "--", "generate", spec.path().to_str().unwrap()])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "generate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "generate should produce output");
}

#[test]
fn test_validate_command_exists() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args(["run", "--", "validate", spec.path().to_str().unwrap()])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "validate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Spec is valid"));
}

#[test]
fn test_generate_lang_python() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "python",
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "python gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn test_generate_lang_csharp() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "csharp",
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "csharp gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_generate_lang_typescript() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "typescript",
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "ts gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_generate_lang_go() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "go",
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "go gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_generate_lang_all() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "all",
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "all gen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_generate_invalid_lang_exits_with_error() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--lang",
            "java",
        ])
        .output()
        .expect("failed to run cli");
    assert!(!output.status.success(), "invalid lang should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported language"));
}

#[test]
fn test_generate_missing_spec_file_exits_with_error() {
    let output = Command::new("cargo")
        .args(["run", "--", "generate", "/nonexistent/path/spec.yaml"])
        .output()
        .expect("failed to run cli");
    assert!(!output.status.success(), "missing spec should fail");
}

#[test]
fn test_generate_invalid_spec_exits_with_error() {
    let file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    std::fs::write(file.path(), "invalid: [unclosed").expect("write");
    let output = Command::new("cargo")
        .args(["run", "--", "generate", file.path().to_str().unwrap()])
        .output()
        .expect("failed to run cli");
    assert!(!output.status.success(), "invalid spec should fail");
}

#[test]
fn test_help_succeeds() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("failed to run cli");
    assert!(output.status.success(), "--help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("generate") || stdout.contains("validate"));
}

#[test]
fn test_validate_invalid_spec_exits_with_error() {
    let spec = create_temp_spec();
    // Overwrite with invalid content
    std::fs::write(spec.path(), "metadata: {name: '', category: ''}\nspec_version: ''\nstructs: []\nmethods: []\nverification:\n  test_cases: []").unwrap();
    let output = Command::new("cargo")
        .args(["run", "--", "validate", spec.path().to_str().unwrap()])
        .output()
        .expect("failed to run cli");
    assert!(
        !output.status.success(),
        "invalid spec validation should fail"
    );
}

#[test]
fn test_generate_with_output_file() {
    let spec = create_temp_spec();
    let out_dir = tempfile::TempDir::new().expect("temp dir");
    let out_file = out_dir.path().join("output.rs");
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            spec.path().to_str().unwrap(),
            "--output",
            out_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run cli");
    assert!(
        output.status.success(),
        "generate with output should succeed"
    );
    assert!(out_file.exists(), "output file should exist");
    let content = std::fs::read_to_string(&out_file).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn test_generate_with_contracts_flag() {
    let spec_yaml = r#"
spec_version: "1.0"
metadata:
  name: "TestContracts"
  category: "test"
structs:
  - name: "Foo"
    fields:
      - name: "val"
        type: "i32"
methods:
  - name: "check"
    preconditions:
      - "val > 0"
    postconditions:
      - "result is valid"
verification:
  test_cases: []
"#;
    let file = tempfile::NamedTempFile::new().expect("temp file");
    std::fs::write(file.path(), spec_yaml).unwrap();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            file.path().to_str().unwrap(),
            "--lang",
            "rust",
            "--contracts",
        ])
        .output()
        .expect("failed to run");
    assert!(
        output.status.success(),
        "generate --contracts failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("assert!(false, \"precondition: val > 0\");"),
        "should contain contract assertion, got: {}",
        stdout
    );
    assert!(
        stdout.contains("assert!(false, \"postcondition: result is valid\");"),
        "should contain postcondition assertion"
    );
}

#[test]
fn test_verify_command_exists() {
    let spec_yaml = r#"
spec_version: "1.0"
metadata:
  name: "VerifyTest"
  category: "test"
structs:
  - name: "Foo"
    fields:
      - name: "val"
        type: "i32"
methods:
  - name: "check"
    preconditions:
      - "val > 0"
    postconditions: []
verification:
  test_cases: []
"#;
    let file = tempfile::NamedTempFile::new().expect("temp file");
    std::fs::write(file.path(), spec_yaml).unwrap();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "verify",
            file.path().to_str().unwrap(),
            "--lang",
            "rust",
        ])
        .output()
        .expect("failed to run");
    assert!(
        output.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--- rust ---"));
    assert!(stdout.contains("todo!()"));
}

#[test]
fn test_verify_unsupported_backend_fails() {
    let spec = create_temp_spec();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "verify",
            spec.path().to_str().unwrap(),
            "--backend",
            "z3",
        ])
        .output()
        .expect("failed to run");
    assert!(!output.status.success(), "unsupported backend should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported verification backend"));
}
