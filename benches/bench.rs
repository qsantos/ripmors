use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::{distributions::Alphanumeric, Rng};

use ripmors::{ascii_encode, unicode_encode};

fn ascii_benchmark(c: &mut Criterion) {
    let ascii: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(1048576)
        .map(char::from)
        .collect();
    let mut group = c.benchmark_group("ASCII");
    group.throughput(Throughput::Bytes(ascii.len() as u64));
    group.bench_function("encode", |b| b.iter(|| ascii_encode(black_box(&ascii))));
    group.finish();
}

fn unicode_benchmark(c: &mut Criterion) {
    let unicode = std::fs::read_to_string("benches/unicode_test_page.txt").unwrap();
    let mut group = c.benchmark_group("Unicode");
    group.throughput(Throughput::Bytes(unicode.len() as u64));
    group.bench_function("encode", |b| b.iter(|| unicode_encode(black_box(&unicode))));
    group.finish();
}

criterion_group!(benches, ascii_benchmark, unicode_benchmark);
criterion_main!(benches);
