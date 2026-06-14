use std::fs;
use std::path::{Path, PathBuf};

use dsa_spec::backend::Backend;

/// Backends to test — (label, dir name, generator function)
struct LangBackend {
    label: &'static str,
    ext: &'static str,
    generate: fn(&dsa_spec::ast::Spec) -> String,
}

impl LangBackend {
    fn golden_path(&self, spec_stem: &str, golden_dir: &Path) -> PathBuf {
        golden_dir.join(format!("{}_{}.golden", spec_stem, self.ext))
    }
}

fn make_backends() -> Vec<LangBackend> {
    vec![
        LangBackend {
            label: "Rust",
            ext: "rs",
            generate: |spec| {
                let b = dsa_spec::rust_backend::RustBackend::new("templates").unwrap();
                b.generate(spec).unwrap()
            },
        },
        LangBackend {
            label: "Python",
            ext: "py",
            generate: |spec| {
                let b = dsa_spec::python_backend::PythonBackend::new("templates").unwrap();
                b.generate(spec).unwrap()
            },
        },
        LangBackend {
            label: "C#",
            ext: "cs",
            generate: |spec| {
                let b = dsa_spec::csharp_backend::CSharpBackend::new("templates").unwrap();
                b.generate(spec).unwrap()
            },
        },
        LangBackend {
            label: "TypeScript",
            ext: "ts",
            generate: |spec| {
                let b = dsa_spec::typescript_backend::TypeScriptBackend::new("templates").unwrap();
                b.generate(spec).unwrap()
            },
        },
        LangBackend {
            label: "Go",
            ext: "go",
            generate: |spec| {
                let b = dsa_spec::go_backend::GoBackend::new("templates").unwrap();
                b.generate(spec).unwrap()
            },
        },
    ]
}

fn spec_files() -> Vec<PathBuf> {
    let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs");
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walkdir_specs(&spec_dir) {
        files.push(entry);
    }
    files.sort();
    files
}

fn walkdir_specs(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir_specs(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                files.push(path);
            }
        }
    }
    files
}

fn parse_spec(path: &Path) -> dsa_spec::ast::Spec {
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn check_or_update_golden(
    generated: &str,
    golden_path: &Path,
    spec_name: &str,
    backend_label: &str,
) {
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::create_dir_all(golden_path.parent().unwrap()).unwrap();
        fs::write(golden_path, generated).unwrap();
        return;
    }

    let expected = fs::read_to_string(golden_path).unwrap_or_else(|e| {
        panic!(
            "Missing golden file {} for {}/{}.\nSet UPDATE_GOLDEN=1 to create it.\nError: {}",
            golden_path.display(),
            spec_name,
            backend_label,
            e
        )
    });

    if generated != expected {
        let diff_path = golden_path.with_extension("diff");
        fs::write(&diff_path, generated).unwrap();
        panic!(
            "Golden mismatch for {}/{}.\nExpected: {}\nGot:      {}\nDiff written to: {}",
            spec_name,
            backend_label,
            golden_path.display(),
            diff_path.display(),
            diff_path.display()
        );
    }
}

#[test]
fn test_golden_files_rust() {
    let backends = make_backends();
    let gdir = golden_dir();
    for path in &spec_files() {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let spec = parse_spec(path);
        for bk in &backends {
            let code = (bk.generate)(&spec);
            let gp = bk.golden_path(stem, &gdir);
            check_or_update_golden(&code, &gp, stem, bk.label);
        }
    }
}
