use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn bench_simple_eval(c: &mut Criterion) {
    let query = "data.test.allow";
    let mut module_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    module_path.push("benches/simple.rego");
    let module = std::fs::read_to_string(&module_path).unwrap();
    let wasm = opa_go::wasm::compile("data.test.allow", &module_path).unwrap();

    let go = opa_go::Rego::new(query, "test", module.as_str()).unwrap();
    let mut wasm = opa_wasm::Policy::from_wasm(&wasm).unwrap();
    let mut rego = opa_rego::Policy::from_query(query, &[module.as_str()]).unwrap();

    let mut group = c.benchmark_group("simple eval");

    group.bench_function(BenchmarkId::new("go", "default true"), |b| {
        b.iter(|| {
            let result = go.eval_bool(black_box(&())).unwrap();
            assert_eq!(true, result);
        })
    });

    group.bench_function(BenchmarkId::new("wasm", "default true"), |b| {
        b.iter(|| {
            let result = wasm.evaluate(black_box(&()));
            assert!(result.is_ok());
        })
    });

    group.bench_function(BenchmarkId::new("rust-rego", "default true"), |b| {
        b.iter(|| {
            let result: bool = rego.evaluate(black_box(())).unwrap();
            assert_eq!(true, result);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_simple_eval);
criterion_main!(benches);
