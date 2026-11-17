//! Compare the decoders (AVX-512 and scalar kernels, whole input, incremental, strict) against a
//! straightforward implementation of the decoding rules.
#![no_main]

use libfuzzer_sys::fuzz_target;
use ripmors::fuzzing::{decode_kernels, decode_reference};
use ripmors::{Alphabet, Decoder};

mod common;

const ALPHABETS: [Alphabet; 7] = [
    Alphabet::Standard,
    Alphabet::Greek,
    Alphabet::Russian,
    Alphabet::Japanese,
    Alphabet::Korean,
    Alphabet::Hebrew,
    Alphabet::Arabic,
];

/// Mostly Morse code: bytes below 0xf0 become Morse elements and separators, and the others
/// escape the next byte, so that any input can still be produced.
fn morse(data: &[u8]) -> Vec<u8> {
    const MORSE: &[u8; 16] = b"....----    /\n\t\r";
    let mut output = Vec::with_capacity(data.len());
    let mut bytes = data.iter();
    while let Some(&b) = bytes.next() {
        if b < 0xf0 {
            output.push(MORSE[usize::from(b % 16)]);
        } else if let Some(&raw) = bytes.next() {
            output.push(raw);
        }
    }
    output
}

fuzz_target!(|data: &[u8]| {
    let [selector, seed, data @ ..] = data else {
        return;
    };
    let alphabet = ALPHABETS[usize::from(selector % 7)];
    let input = if selector & 0x80 == 0 {
        morse(data)
    } else {
        data.to_vec()
    };
    let decoder = Decoder::new(alphabet);
    let expected = decode_reference(&input, alphabet);

    assert_eq!(decoder.decode(&input), expected);

    let [avx512, scalar] = decode_kernels(&input, alphabet);
    assert_eq!(avx512, scalar);

    let mut output = String::from("x");
    decoder.decode_into(&input, &mut output);
    assert_eq!(output[1..], expected);

    let mut output = String::new();
    let mut stream = decoder.stream();
    for piece in common::pieces(&input, *seed) {
        stream.feed(piece, &mut output);
    }
    stream.finish(&mut output);
    assert_eq!(output, expected);

    match decoder.decode_strict(&input) {
        Ok(output) => assert_eq!(output, expected),
        Err(e) => assert!(e.position() < input.len()),
    }
});
