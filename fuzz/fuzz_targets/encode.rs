//! Compare the encoders (AVX-512 and scalar kernels, whole input, incremental, strict) against a
//! straightforward implementation of the encoding rules, and check round trips.
#![no_main]

use libfuzzer_sys::fuzz_target;
use ripmors::fuzzing::{encode_kernels, encode_reference};
use ripmors::{Alphabet, Charset, Decoder, Encoder};

mod common;

fuzz_target!(|data: &[u8]| {
    let [selector, seed, input @ ..] = data else {
        return;
    };
    let charset = if selector & 1 == 0 {
        Charset::Utf8
    } else {
        Charset::Ascii
    };
    let encoder = Encoder::new(charset);
    let expected = encode_reference(input, charset);

    assert_eq!(encoder.encode(input), expected);

    // existing output before `floor` must never be modified
    for (prefix, floor) in [(&b""[..], 0), (b"x ", 2), (b"x ", 1)] {
        let [avx512, scalar] = encode_kernels(input, charset, prefix, floor);
        assert_eq!(avx512, scalar);
        assert_eq!(avx512.0[..floor], prefix[..floor]);
    }

    let mut output = String::from("x ");
    encoder.encode_into(input, &mut output);
    assert!(output.starts_with("x "));

    let mut output = String::new();
    let mut stream = encoder.stream();
    for piece in common::pieces(input, *seed) {
        stream.feed(piece, &mut output);
    }
    stream.finish(&mut output);
    assert_eq!(output, expected);

    match encoder.encode_strict(input) {
        Ok(output) => assert_eq!(output, expected),
        Err(e) => assert!(e.position() < input.len()),
    }

    // round trip of the characters that decode to themselves
    let text: Vec<u8> = input
        .iter()
        .copied()
        .filter(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == b' ')
        .collect();
    let morse = encoder.encode(&text);
    let decoded = Decoder::new(Alphabet::Standard).decode(morse.as_bytes());
    assert_eq!(decoded.as_bytes(), text);
});
