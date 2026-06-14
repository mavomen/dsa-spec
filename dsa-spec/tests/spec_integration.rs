use std::fs;
use std::path::Path;

fn get_spec_files() -> Vec<std::path::PathBuf> {
    let spec_dir = "../specs";
    let mut files = Vec::new();
    walkdir_specs(Path::new(spec_dir), &mut files);
    files.sort();
    files
}

fn walkdir_specs(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                walkdir_specs(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                files.push(path);
            }
        }
    }
}

fn parse_spec(path: &std::path::Path) -> dsa_spec::ast::Spec {
    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read spec: {:?}", path));
    serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse spec {}: {}", path.display(), e))
}

#[test]
fn test_all_specs_parse_and_generate_rust() {
    use dsa_spec::backend::Backend;
    use dsa_spec::rust_backend::RustBackend;
    let backend = RustBackend::new("templates").expect("RustBackend");
    for path in get_spec_files() {
        let spec = parse_spec(&path);
        let generated = backend
            .generate(&spec)
            .unwrap_or_else(|e| panic!("Rust gen failed for {}: {}", path.display(), e));
        assert!(
            !generated.is_empty(),
            "Rust gen empty for {}",
            path.display()
        );
    }
}

#[test]
fn test_all_specs_parse_and_generate_python() {
    use dsa_spec::backend::Backend;
    use dsa_spec::python_backend::PythonBackend;
    let backend = PythonBackend::new("templates").expect("PythonBackend");
    for path in get_spec_files() {
        let spec = parse_spec(&path);
        let generated = backend
            .generate(&spec)
            .unwrap_or_else(|e| panic!("Python gen failed for {}: {}", path.display(), e));
        assert!(
            !generated.is_empty(),
            "Python gen empty for {}",
            path.display()
        );
    }
}

#[test]
fn test_all_specs_parse_and_generate_csharp() {
    use dsa_spec::backend::Backend;
    use dsa_spec::csharp_backend::CSharpBackend;
    let backend = CSharpBackend::new("templates").expect("CSharpBackend");
    for path in get_spec_files() {
        let spec = parse_spec(&path);
        let generated = backend
            .generate(&spec)
            .unwrap_or_else(|e| panic!("C# gen failed for {}: {}", path.display(), e));
        assert!(!generated.is_empty(), "C# gen empty for {}", path.display());
    }
}

#[test]
fn test_all_specs_parse_and_generate_typescript() {
    use dsa_spec::backend::Backend;
    use dsa_spec::typescript_backend::TypeScriptBackend;
    let backend = TypeScriptBackend::new("templates").expect("TypeScriptBackend");
    for path in get_spec_files() {
        let spec = parse_spec(&path);
        let generated = backend
            .generate(&spec)
            .unwrap_or_else(|e| panic!("TS gen failed for {}: {}", path.display(), e));
        assert!(!generated.is_empty(), "TS gen empty for {}", path.display());
    }
}

#[test]
fn test_all_specs_parse_and_generate_go() {
    use dsa_spec::backend::Backend;
    use dsa_spec::go_backend::GoBackend;
    let backend = GoBackend::new("templates").expect("GoBackend");
    for path in get_spec_files() {
        let spec = parse_spec(&path);
        let generated = backend
            .generate(&spec)
            .unwrap_or_else(|e| panic!("Go gen failed for {}: {}", path.display(), e));
        assert!(!generated.is_empty(), "Go gen empty for {}", path.display());
    }
}
