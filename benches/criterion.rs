use std::io::Seek;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};

use ripmors::*;

fn ascii_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("1-original.txt").unwrap();
    let data = data.as_bytes();
    let mut f = std::fs::File::open("1-original.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Encode ASCII");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("string", |b| {
        b.iter(|| Encoder::new(Charset::Ascii).encode(black_box(data)))
    });

    group.bench_function("stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            Encoder::new(Charset::Ascii)
                .encode_stream(&mut f, &mut devnull)
                .unwrap();
        })
    });

    group.finish();
}

fn unicode_benchmark(c: &mut Criterion) {
    let data = std::fs::read_to_string("4-unicode.txt").unwrap();
    let mut f = std::fs::File::open("4-unicode.txt").unwrap();
    let mut devnull = std::fs::File::create("/dev/null").unwrap();
    let mut group = c.benchmark_group("Encode Unicode");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("string", |b| {
        b.iter(|| Encoder::new(Charset::Utf8).encode(black_box(data.as_bytes())))
    });

    group.bench_function("stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            Encoder::new(Charset::Utf8)
                .encode_stream(&mut f, &mut devnull)
                .unwrap();
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
        b.iter(|| Decoder::new(Alphabet::Standard).decode(black_box(data.as_bytes())))
    });

    group.bench_function("stream", |b| {
        b.iter(|| {
            f.rewind().unwrap();
            Decoder::new(Alphabet::Standard)
                .decode_stream(&mut f, &mut devnull)
                .unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    ascii_benchmark,
    unicode_benchmark,
    decode_benchmark
);
criterion_main!(benches);
