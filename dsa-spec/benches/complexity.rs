use criterion::{Criterion, criterion_group, criterion_main};

fn bench_complexity_report(c: &mut Criterion) {
    let specs = dsa_spec::complexity::load_specs_from_dir(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../specs")
            .to_str()
            .unwrap(),
    )
    .unwrap();

    c.bench_function("complexity_report", |b| {
        b.iter(|| {
            let _ = dsa_spec::complexity::generate_report(&specs);
        });
    });

    c.bench_function("complexity_json", |b| {
        b.iter(|| {
            let _ = dsa_spec::complexity::generate_json_report(&specs);
        });
    });
}

criterion_group!(benches, bench_complexity_report);
criterion_main!(benches);
