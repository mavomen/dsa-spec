use crate::ast::Spec;
use crate::error::SpecError;
use std::fs;

/// Migrate a spec file to the target version.
/// Creates a `.bak` backup before modifying the file.
pub fn migrate_spec_file(path: &str, target_version: &str) -> Result<(), SpecError> {
    let yaml = fs::read_to_string(path).map_err(|e| SpecError::IoError {
        message: format!("failed to read {path}: {e}"),
    })?;

    let mut spec: Spec = serde_yaml::from_str(&yaml).map_err(|e| SpecError::ParseError {
        message: e.to_string(),
        line: e.location().map(|l| l.line()),
        column: e.location().map(|l| l.column()),
    })?;

    if spec.spec_version == target_version {
        return Ok(());
    }

    match spec.spec_version.as_str() {
        "1.0" if target_version == "2.0" => {
            spec.spec_version = "2.0".into();
        }
        _ => {
            return Err(SpecError::VersionMismatch {
                expected: "1.0".into(),
                found: spec.spec_version.clone(),
            });
        }
    }

    let bak_path = format!("{path}.bak");
    fs::copy(path, &bak_path).map_err(|e| SpecError::IoError {
        message: format!("failed to create backup {bak_path}: {e}"),
    })?;

    let out = serde_yaml::to_string(&spec).map_err(|e| SpecError::IoError {
        message: format!("failed to serialize spec: {e}"),
    })?;
    fs::write(path, out).map_err(|e| SpecError::IoError {
        message: format!("failed to write {path}: {e}"),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Metadata, Spec};

    fn make_v1_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Test".into(),
                category: "test".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_migrate_v1_to_v2_updates_version() {
        let spec = make_v1_spec();
        let yaml = serde_yaml::to_string(&spec).unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("spec.yaml");
        fs::write(&path, &yaml).unwrap();

        migrate_spec_file(path.to_str().unwrap(), "2.0").unwrap();

        let result = fs::read_to_string(&path).unwrap();
        let migrated: Spec = serde_yaml::from_str(&result).unwrap();
        assert_eq!(migrated.spec_version, "2.0");
    }

    #[test]
    fn test_migrate_same_version_is_noop() {
        let spec = make_v1_spec();
        let yaml = serde_yaml::to_string(&spec).unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("spec.yaml");
        fs::write(&path, &yaml).unwrap();

        migrate_spec_file(path.to_str().unwrap(), "1.0").unwrap();
        let result = fs::read_to_string(&path).unwrap();
        assert_eq!(result, yaml);
    }

    #[test]
    fn test_migrate_unknown_path_fails() {
        let result = migrate_spec_file("/nonexistent/path.yaml", "2.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_migrate_creates_backup() {
        let spec = make_v1_spec();
        let yaml = serde_yaml::to_string(&spec).unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("spec.yaml");
        fs::write(&path, &yaml).unwrap();

        migrate_spec_file(path.to_str().unwrap(), "2.0").unwrap();

        let bak_path = dir.path().join("spec.yaml.bak");
        assert!(bak_path.exists());
        let bak_content = fs::read_to_string(bak_path).unwrap();
        assert_eq!(bak_content, yaml);
    }

    #[test]
    fn test_migrate_unsupported_version_fails() {
        let mut spec = make_v1_spec();
        spec.spec_version = "0.5".into();
        let yaml = serde_yaml::to_string(&spec).unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("spec.yaml");
        fs::write(&path, &yaml).unwrap();

        let result = migrate_spec_file(path.to_str().unwrap(), "2.0");
        assert!(result.is_err());
    }
}
