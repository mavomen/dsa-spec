use criterion::{Criterion, criterion_group, criterion_main};

fn collect_spec_yamls() -> Vec<(String, String)> {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs");
    let mut yamls = Vec::new();
    let mut dirs = vec![spec_dir];
    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|ext| ext == "yaml")
                    && let Ok(yaml) = std::fs::read_to_string(&path)
                {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    yamls.push((name, yaml));
                }
            }
        }
    }
    yamls.sort_by(|a, b| a.0.cmp(&b.0));
    yamls
}

fn bench_parse_all_specs(c: &mut Criterion) {
    let yamls = collect_spec_yamls();
    let mut group = c.benchmark_group("parse");
    group.sample_size(20);
    for (name, yaml) in &yamls {
        group.bench_with_input(name, yaml, |b, yaml| {
            b.iter(|| {
                let _ = dsa_spec::parser::parse(yaml);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse_all_specs);
criterion_main!(benches);
