//! Compare the AVX-512 kernel (when available) and the portable code with a straightforward
//! reference, for all the alphabets and custom tables, on random and structured inputs, with
//! sequences straddling the boundaries of blocks and of input pieces.

use super::*;
use rand::{RngExt, SeedableRng, rngs::StdRng};

const ALPHABETS: [Alphabet; 7] = [
    Alphabet::Standard,
    Alphabet::Greek,
    Alphabet::Russian,
    Alphabet::Japanese,
    Alphabet::Korean,
    Alphabet::Hebrew,
    Alphabet::Arabic,
];

/// A custom table mixing UTF-8 encodings of 1 to 4 bytes.
fn to_mixed(binary: u8) -> char {
    match binary % 5 {
        0 => '\0',
        1 => (b'A' + binary % 26) as char,
        2 => char::from_u32(0x400 + binary as u32).unwrap(),
        3 => char::from_u32(0x30a0 + binary as u32 % 96).unwrap(),
        _ => char::from_u32(0x1f600 + binary as u32 % 64).unwrap(),
    }
}

/// A custom table with UTF-8 encodings of 1 to 3 bytes, including control characters, and the
/// extreme values of each length.
fn to_controls(binary: u8) -> char {
    match binary % 4 {
        0 => (binary % 32) as char,
        1 => '\u{80}',
        2 => '\u{ffff}',
        _ => '\u{7ff}',
    }
}

/// A custom table where the length of the UTF-8 encoding depends on the first two elements:
/// sequences starting with `.` are ASCII, with `-.` two bytes, and with `--` three bytes.
fn to_split(binary: u8) -> char {
    if binary < 2 {
        '\0'
    } else if binary & 1 == 0 {
        (b'a' + binary % 26) as char
    } else if binary < 4 || binary & 2 == 0 {
        char::from_u32(0x400 + binary as u32).unwrap()
    } else {
        char::from_u32(0x3040 + binary as u32).unwrap()
    }
}

/// All the tables to test.
fn tables() -> Vec<(String, DecodeTable)> {
    let mut tables: Vec<(String, DecodeTable)> = ALPHABETS
        .iter()
        .map(|&alphabet| {
            (
                format!("{alphabet:?}"),
                Decoder::new(alphabet).table.into_owned(),
            )
        })
        .collect();
    tables.push(("mixed".into(), DecodeTable::from_fn(to_mixed)));
    tables.push(("controls".into(), DecodeTable::from_fn(to_controls)));
    tables.push(("split".into(), DecodeTable::from_fn(to_split)));
    tables
}

/// Straightforward implementation of the decoding rules.
fn reference(input: &[u8], table: &DecodeTable) -> Vec<u8> {
    let mut output = Vec::new();
    let mut start = 0;
    for i in 0..=input.len() {
        let c = input.get(i).copied();
        if c.is_some_and(|c| !matches!(c, b' ' | b'/' | b'\t' | b'\n' | b'\r')) {
            continue;
        }
        let sequence = &input[start..i];
        if sequence.len() <= 7 && sequence.iter().all(|&c| c == b'.' || c == b'-') {
            let mut binary = 1;
            for &c in sequence.iter().rev() {
                binary = binary * 2 + (c == b'-') as usize;
            }
            output.extend_from_slice(&table.utf8[binary][..table.len[binary] as usize]);
        }
        match c {
            Some(b'/') => output.push(b' '),
            Some(b' ') | None => (),
            Some(c) => output.push(c),
        }
        start = i + 1;
    }
    output
}

/// Decode `input` at once, with or without the AVX-512 kernel.
fn decode(input: &[u8], table: &DecodeTable, allow_avx512: bool) -> Vec<u8> {
    let mut output = Vec::new();
    let chunk_start = decode_buffer_with(input, table, &mut output, allow_avx512);
    table.push(sequence_index(&input[chunk_start..]), &mut output);
    output
}

fn check(input: &[u8], name: &str, table: &DecodeTable) {
    let expected = reference(input, table);
    for allow_avx512 in [false, true] {
        let output = decode(input, table, allow_avx512);
        if output != expected {
            let k = output.iter().zip(&expected).position(|(a, b)| a != b);
            panic!(
                "{name} allow_avx512={allow_avx512} len={} output={} expected={} first \
                 difference at {k:?}\ninput={:?}",
                input.len(),
                output.len(),
                expected.len(),
                String::from_utf8_lossy(input),
            );
        }
    }
}

fn element(rng: &mut StdRng) -> u8 {
    if rng.random_bool(0.5) { b'.' } else { b'-' }
}

/// Generate Morse code: sequences of up to 7 elements (starting with `prefix` when given), with
/// occasional longer sequences, separators of all kinds, and invalid bytes, with weights
/// depending on `profile`.
fn generate(rng: &mut StdRng, len: usize, profile: u32, prefix: Option<&[u8]>) -> Vec<u8> {
    let mut input = Vec::with_capacity(len + 32);
    while input.len() < len {
        let elements = match profile {
            // mostly regular text
            0 => rng.random_range(1..=5),
            // short sequences, many separators
            1 => rng.random_range(0..=2),
            // sequences near the limit of 7 elements
            _ => rng.random_range(5..=9),
        };
        let start = input.len();
        for _ in 0..elements {
            input.push(element(rng));
        }
        if let Some(prefix) = prefix {
            let n = prefix.len().min(elements);
            input[start..start + n].copy_from_slice(&prefix[..n]);
        }
        if rng.random_ratio(1, 200) {
            // long sequence
            for _ in 0..rng.random_range(8..20) {
                input.push(element(rng));
            }
        }
        if rng.random_ratio(1, 200) {
            // invalid byte inside a sequence
            // NOTE: some of them are equal to elements or separators modulo 64 or 128
            const INVALID: &[u8] = b"x\0\x01\x0b\x7f\x80\xff\xae\xad\xa0\x6e\x6d\x60\x6f\x49";
            input.push(INVALID[rng.random_range(0..INVALID.len())]);
        }
        let separator = match rng.random_range(0..100) {
            0..70 => b' ',
            70..85 => b'/',
            85..92 => b'\n',
            92..95 => b'\t',
            95..98 => b'\r',
            98 => b'\0',
            _ => rng.random_range(1..b' '),
        };
        input.push(separator);
        if profile == 1 && rng.random_bool(0.3) {
            input.push(b' ');
        }
    }
    input.truncate(len);
    input
}

#[test]
#[cfg_attr(miri, ignore)]
fn structured() {
    let mut rng = StdRng::seed_from_u64(0);
    for (name, table) in tables() {
        for profile in 0..3 {
            // all the lengths around the first few batches
            for len in 0..1100 {
                check(&generate(&mut rng, len, profile, None), &name, &table);
            }
            for _ in 0..10 {
                let len = rng.random_range(1100..40_000);
                check(&generate(&mut rng, len, profile, None), &name, &table);
            }
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn structured_offsets() {
    // the same input at every offset, so that each sequence straddles the block boundaries
    let mut rng = StdRng::seed_from_u64(1);
    for (name, table) in tables() {
        for profile in 0..3 {
            let input = generate(&mut rng, 1000, profile, None);
            for offset in 0..130 {
                check(&input[offset..], &name, &table);
            }
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn changing_density() {
    // segments with only ASCII characters, or many characters of two or three bytes, so that the
    // kernel switches between its output paths
    let mut rng = StdRng::seed_from_u64(2);
    let table = DecodeTable::from_fn(to_split);
    let prefixes: [Option<&[u8]>; 4] = [Some(b"."), Some(b"-."), Some(b"--"), None];
    for _ in 0..20 {
        let mut input = Vec::new();
        for _ in 0..rng.random_range(1..12) {
            let len = rng.random_range(0..60_000);
            let prefix = prefixes[rng.random_range(0..prefixes.len())];
            let profile = rng.random_range(0..3);
            input.extend(generate(&mut rng, len, profile, prefix));
        }
        check(&input, "split", &table);
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn random_bytes() {
    let mut rng = StdRng::seed_from_u64(3);
    let all_bytes: Vec<u8> = (0..=255).collect();
    let alphabets: [&[u8]; 5] = [
        b".- /",
        b".-- /\n\r",
        b"..........------- /",
        b"..--/ \t\r\x01\x1f\x20\x21\xff\x0b\xae\xad\xa0\x6e\x6f\x60",
        &all_bytes,
    ];
    for (name, table) in tables() {
        for alphabet in alphabets {
            for _ in 0..30 {
                let len = rng.random_range(0..5_000);
                let input: Vec<u8> = (0..len)
                    .map(|_| alphabet[rng.random_range(0..alphabet.len())])
                    .collect();
                check(&input, &name, &table);
            }
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn stream() {
    // incremental decoding, with pieces of random sizes
    let mut rng = StdRng::seed_from_u64(4);
    let decoders = ALPHABETS
        .iter()
        .map(|&alphabet| Decoder::new(alphabet))
        .chain([to_mixed, to_controls, to_split].map(Decoder::from_fn));
    for decoder in decoders {
        for max_piece in [1, 7, 100, 1000, 100_000] {
            let len = rng.random_range(0..200_000);
            let profile = rng.random_range(0..3);
            let input = generate(&mut rng, len, profile, None);
            let expected = reference(&input, &decoder.table);
            let mut stream = decoder.stream();
            let mut output = String::new();
            let mut rest = &input[..];
            while !rest.is_empty() {
                let n = rng.random_range(0..=max_piece.min(rest.len()));
                stream.feed(&rest[..n], &mut output);
                rest = &rest[n..];
            }
            stream.finish(&mut output);
            assert!(output.as_bytes() == expected, "max_piece={max_piece}");
            assert!(decoder.decode(&input).as_bytes() == expected);
        }
    }
}
