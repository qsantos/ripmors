use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{distributions::Alphanumeric, Rng};

use ripmors::{ascii_encode, unicode_encode};

fn criterion_benchmark(c: &mut Criterion) {
    let ascii: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(1048576)
        .map(char::from)
        .collect();

    let unicode = std::fs::read_to_string("benches/unicode_test_page.txt").unwrap();

    c.bench_function("encode ASCII", |b| b.iter(|| ascii_encode(black_box(&ascii))));
    c.bench_function("encode Unicode", |b| b.iter(|| unicode_encode(black_box(&unicode))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
