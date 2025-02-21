use std::io::Seek;

use std::hint::black_box;
use tango_bench::{IntoBenchmarks, benchmark_fn, tango_benchmarks, tango_main};

use ripmors::*;

fn ascii_benchmarks() -> impl IntoBenchmarks {
    [
        benchmark_fn("encode_string_ascii", |b| {
            let data = std::fs::read_to_string("1-original.txt").unwrap();
            b.iter(move || encode_string_ascii(black_box(data.as_bytes())))
        }),
        benchmark_fn("encode_stream_ascii", |b| {
            let mut f = std::fs::File::open("1-original.txt").unwrap();
            let mut devnull = std::fs::File::create("/dev/null").unwrap();
            b.iter(move || {
                f.rewind().unwrap();
                encode_stream_ascii(&mut f, &mut devnull).unwrap();
            })
        }),
    ]
}

fn unicode_benchmarks() -> impl IntoBenchmarks {
    [
        benchmark_fn("encode_string_unicode", |b| {
            let data = std::fs::read_to_string("4-unicode.txt").unwrap();
            b.iter(move || encode_string(black_box(&data)))
        }),
        benchmark_fn("encode_stream_unicode", |b| {
            let mut f = std::fs::File::open("4-unicode.txt").unwrap();
            let mut devnull = std::fs::File::create("/dev/null").unwrap();
            b.iter(move || {
                f.rewind().unwrap();
                encode_stream(&mut f, &mut devnull).unwrap();
            })
        }),
    ]
}

fn decode_benchmarks() -> impl IntoBenchmarks {
    [
        benchmark_fn("decode_string", |b| {
            let data = std::fs::read_to_string("2-encoded.txt").unwrap();
            b.iter(move || decode_string(black_box(&data.as_bytes()), to_standard))
        }),
        benchmark_fn("decode_stream", |b| {
            let mut f = std::fs::File::open("2-encoded.txt").unwrap();
            let mut devnull = std::fs::File::create("/dev/null").unwrap();
            b.iter(move || {
                f.rewind().unwrap();
                encode_stream(&mut f, &mut devnull).unwrap();
            })
        }),
    ]
}

tango_benchmarks!(
    ascii_benchmarks(),
    unicode_benchmarks(),
    decode_benchmarks()
);
tango_main!();
