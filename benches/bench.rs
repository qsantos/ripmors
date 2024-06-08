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

fn decode_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("2-encoded.txt").unwrap();
    let mut f = std::fs::File::open("2-encoded.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Unicode");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("decode_string", |b| {
        b.iter(|| morse_decode_to_string(black_box(&data.as_bytes()), morse_to_standard))
    });

    group.bench_function("decode_stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            decode_stream(&mut f, &mut devnull, morse_to_standard);
        })
    });

    group.finish();
}

criterion_group!(benches, ascii_benchmark, standard_benchmark, decode_benchmark);
criterion_main!(benches);
