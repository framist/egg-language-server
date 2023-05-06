use criterion::{black_box, criterion_group, criterion_main, Criterion};
use egg_language_server::*;

// TODO

fn test_py(code: &str) {
    py_parser(code);
}

const CODE: &str = r#"
def add1(x):
    return x + 1
y = 1
add1(y)
"#;


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("my", |b| b.iter(|| test_py(black_box(CODE))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);