use std::fs;

#[test]
fn test_generate_all_specs_succeeds() {
    let spec_dir = "../specs";
    let specs = fs::read_dir(spec_dir).expect("specs directory not found");

    for entry in specs {
        let path = entry.expect("failed to read entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let spec_content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read spec: {:?}", path));

            let spec: dsa_spec::ast::Spec = serde_yaml::from_str(&spec_content)
                .unwrap_or_else(|e| panic!("Failed to parse spec {}: {}", path.display(), e));

            use dsa_spec::backend::Backend;
            use dsa_spec::rust_backend::RustBackend;

            let backend = RustBackend::new("templates").expect("Failed to create RustBackend");
            let generated = backend
                .generate(&spec)
                .unwrap_or_else(|e| panic!("Generation failed for {}: {}", path.display(), e));

            assert!(
                !generated.is_empty(),
                "Generated code empty for {}",
                path.display()
            );
        }
    }
}
