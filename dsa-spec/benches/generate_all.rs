use criterion::{Criterion, criterion_group, criterion_main};
use dsa_spec::backend::Backend;

fn collect_specs() -> Vec<(String, dsa_spec::ast::Spec)> {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs");
    let mut specs = Vec::new();
    let mut dirs = vec![spec_dir];
    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|ext| ext == "yaml") {
                    let yaml = match std::fs::read_to_string(&path) {
                        Ok(y) => y,
                        _ => continue,
                    };
                    let parsed = match dsa_spec::parser::parse(&yaml) {
                        Ok(p) => p,
                        _ => continue,
                    };
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    specs.push((name, parsed));
                }
            }
        }
    }
    specs.sort_by(|a, b| a.0.cmp(&b.0));
    specs
}

fn bench_rust(c: &mut Criterion) {
    let specs = collect_specs();
    let backend = dsa_spec::rust_backend::RustBackend::new("templates").unwrap();
    let mut group = c.benchmark_group("generate/rust");
    group.sample_size(20);
    for (name, spec) in &specs {
        group.bench_with_input(name, spec, |b, spec| {
            b.iter(|| backend.generate(spec));
        });
    }
    group.finish();
}

fn bench_python(c: &mut Criterion) {
    let specs = collect_specs();
    let backend = dsa_spec::python_backend::PythonBackend::new("templates").unwrap();
    let mut group = c.benchmark_group("generate/python");
    group.sample_size(20);
    for (name, spec) in &specs {
        group.bench_with_input(name, spec, |b, spec| {
            b.iter(|| backend.generate(spec));
        });
    }
    group.finish();
}

fn bench_csharp(c: &mut Criterion) {
    let specs = collect_specs();
    let backend = dsa_spec::csharp_backend::CSharpBackend::new("templates").unwrap();
    let mut group = c.benchmark_group("generate/csharp");
    group.sample_size(20);
    for (name, spec) in &specs {
        group.bench_with_input(name, spec, |b, spec| {
            b.iter(|| backend.generate(spec));
        });
    }
    group.finish();
}

fn bench_typescript(c: &mut Criterion) {
    let specs = collect_specs();
    let backend = dsa_spec::typescript_backend::TypeScriptBackend::new("templates").unwrap();
    let mut group = c.benchmark_group("generate/typescript");
    group.sample_size(20);
    for (name, spec) in &specs {
        group.bench_with_input(name, spec, |b, spec| {
            b.iter(|| backend.generate(spec));
        });
    }
    group.finish();
}

fn bench_go(c: &mut Criterion) {
    let specs = collect_specs();
    let backend = dsa_spec::go_backend::GoBackend::new("templates").unwrap();
    let mut group = c.benchmark_group("generate/go");
    group.sample_size(20);
    for (name, spec) in &specs {
        group.bench_with_input(name, spec, |b, spec| {
            b.iter(|| backend.generate(spec));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_rust,
    bench_python,
    bench_csharp,
    bench_typescript,
    bench_go,
);
criterion_main!(benches);
