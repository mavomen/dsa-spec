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
