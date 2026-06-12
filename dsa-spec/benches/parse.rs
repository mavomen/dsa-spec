use criterion::{Criterion, criterion_group, criterion_main};

fn bench_parse_all_specs(c: &mut Criterion) {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs");
    let yamls: Vec<String> = std::fs::read_dir(&spec_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
        .map(|e| std::fs::read_to_string(e.path()).unwrap())
        .collect();

    c.bench_function("parse_all_specs", |b| {
        b.iter(|| {
            for yaml in &yamls {
                let _ = dsa_spec::parser::parse(yaml);
            }
        });
    });
}

criterion_group!(benches, bench_parse_all_specs);
criterion_main!(benches);
