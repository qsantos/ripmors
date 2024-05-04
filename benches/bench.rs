use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{distributions::Alphanumeric, Rng};

use ripmors::ascii_encode;

fn criterion_benchmark(c: &mut Criterion) {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(1048576)
        .map(char::from)
        .collect();

    c.bench_function("encode ASCII", |b| b.iter(|| ascii_encode(black_box(&s))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

