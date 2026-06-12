use criterion::{Criterion, criterion_group, criterion_main};
use dsa_spec::backend::Backend;

fn bench_generate_all_backends(c: &mut Criterion) {
    let yaml = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/bst.yaml"),
    )
    .unwrap();
    let spec = dsa_spec::parser::parse(&yaml).unwrap();

    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(dsa_spec::rust_backend::RustBackend::new("templates").unwrap()),
        Box::new(dsa_spec::python_backend::PythonBackend::new("templates").unwrap()),
        Box::new(dsa_spec::csharp_backend::CSharpBackend::new("templates").unwrap()),
        Box::new(dsa_spec::typescript_backend::TypeScriptBackend::new("templates").unwrap()),
        Box::new(dsa_spec::go_backend::GoBackend::new("templates").unwrap()),
    ];

    c.bench_function("generate_all_backends", |b| {
        b.iter(|| {
            for backend in &backends {
                let _ = backend.generate(&spec);
            }
        });
    });
}

criterion_group!(benches, bench_generate_all_backends);
criterion_main!(benches);
