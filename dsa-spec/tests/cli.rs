use std::process::Command;

#[test]
fn test_generate_command_exists() {
    let output = Command::new("cargo")
        .args(["run", "--", "generate"])
        .output()
        .expect("failed to run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate command"));
}

#[test]
fn test_validate_command_exists() {
    let output = Command::new("cargo")
        .args(["run", "--", "validate"])
        .output()
        .expect("failed to run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validate command"));
}
