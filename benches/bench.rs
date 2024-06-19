use std::io::Seek;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use ripmors::*;

fn ascii_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("1-original.txt").unwrap();
    let data = data.as_bytes();
    let mut f = std::fs::File::open("1-original.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Encode ASCII");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("string", |b| {
        b.iter(|| encode_string_ascii(black_box(&data)))
    });

    group.bench_function("stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            encode_stream_ascii(&mut f, &mut devnull).unwrap();
        })
    });

    group.finish();
}

fn standard_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("4-unicode.txt").unwrap();
    let mut f = std::fs::File::open("4-unicode.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Encode Unicode");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("string", |b| b.iter(|| encode_string(black_box(&data))));

    group.bench_function("stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            encode_stream(&mut f, &mut devnull);
        })
    });

    group.finish();
}

fn decode_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("2-encoded.txt").unwrap();
    let mut f = std::fs::File::open("2-encoded.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Decode");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("string", |b| {
        b.iter(|| decode_string(black_box(&data.as_bytes()), to_standard))
    });

    group.bench_function("stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            decode_stream(&mut f, &mut devnull, to_standard).unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    ascii_benchmark,
    standard_benchmark,
    decode_benchmark
);
criterion_main!(benches);
