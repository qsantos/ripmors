use std::io::Seek;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use ripmors::*;

fn ascii_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("1-original.txt").unwrap();
    let mut f = std::fs::File::open("1-original.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("ASCII");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("encode_string", |b| {
        b.iter(|| ascii_encode_to_string(black_box(&data)))
    });

    group.bench_function("encode_stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            encode_stream_ascii(&mut f, &mut devnull);
        })
    });

    group.finish();
}

fn standard_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("4-unicode.txt").unwrap();
    let mut f = std::fs::File::open("4-unicode.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Unicode");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("encode_string", |b| {
        b.iter(|| standard_encode_to_string(black_box(&data)))
    });

    group.bench_function("encode_stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            encode_stream_standard(&mut f, &mut devnull);
        })
    });

    group.finish();
}

criterion_group!(benches, ascii_benchmark, standard_benchmark);
criterion_main!(benches);
