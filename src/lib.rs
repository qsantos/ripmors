#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
//! # Morse code format
//!
//! Morse code is represented with ASCII characters:
//!
//! - `.` and `-` are the Morse elements (dot and dash);
//! - ` ` (space) is the letter space, which ends each character;
//! - `/` is the word space, which encodes a space;
//! - `\t`, `\n` and `\r` are kept as-is; when encoding, they replace the preceding letter space.
//!
//! When decoding, a Morse sequence is dropped (produces nothing) when it contains any other byte,
//! when it has more than 7 elements, or when it does not correspond to a character of the
//! alphabet. When encoding, characters that cannot be encoded are dropped. Use
//! [`Decoder::decode_strict`] and [`Encoder::encode_strict`] to detect these cases instead.
//!
//! # API overview
//!
//! - [`Encoder`] and [`Decoder`] encode and decode byte slices ([`Encoder::encode`],
//!   [`Decoder::decode`]), possibly into an existing buffer ([`Encoder::encode_into`],
//!   [`Decoder::decode_into`]), or from a [`Read`][std::io::Read] object to a
//!   [`Write`][std::io::Write] object ([`Encoder::encode_stream`], [`Decoder::decode_stream`]);
//! - [`Encoder::encode_parallel`], [`Encoder::encode_parallel_write`],
//!   [`Encoder::encode_stream_parallel`] and their [`Decoder`] counterparts do the same with
//!   several threads;
//! - [`EncodeStream`] and [`DecodeStream`] encode and decode input given in pieces, which lets
//!   you bring your own I/O.
//!
//! # Feature Flags
//! There are two feature flags, `clap` and `fast-io`, enabled by default. They are only used for
//! the binary (`fast-io` memory-maps the input when it is a regular file, and enlarges pipes, with
//! `libc`). They only exist to reduce the dependencies of the libraries without splitting the
//! crate.

// Do not consider the body of unsafe functions to be an unsafe block by default. This forces one
// to explicitly mark what parts of an unsafe functions are actually unsafe.
#![warn(unsafe_op_in_unsafe_fn)]
// Enforce "SAFETY:" comments before unsafe blocks
#![warn(clippy::undocumented_unsafe_blocks)]
// Enforce "# Safety" section in documentation of unsafe functions
#![warn(clippy::missing_safety_doc)]

mod decode;
mod decode_mapping;
mod encode;
mod encode_ascii;
mod encode_ascii_mapping;
mod encode_unicode;
mod encode_unicode_mapping;
mod parallel;

// Public API
pub use decode::{Alphabet, DecodeError, DecodeErrorKind, DecodeStream, Decoder};
pub use decode_mapping::{
    to_arabic, to_greek, to_hebrew, to_japanese, to_korean, to_russian, to_standard,
};
pub use encode::{Charset, EncodeError, EncodeErrorKind, EncodeStream, Encoder};

/// Encode characters from a [string slice][&str] into a [String].
#[deprecated(
    since = "0.3.0",
    note = "use `Encoder::default().encode(input.as_bytes())`"
)]
pub fn encode_string(input: &str) -> String {
    Encoder::new(Charset::Utf8).encode(input.as_bytes())
}

/// Encode ASCII characters from a [byte slice][slice] into a [String].
#[deprecated(
    since = "0.3.0",
    note = "use `Encoder::new(Charset::Ascii).encode(input)`"
)]
pub fn encode_string_ascii(input: &[u8]) -> String {
    Encoder::new(Charset::Ascii).encode(input)
}

/// Encode Unicode characters from a [Read][std::io::Read] object into a
/// [Write][std::io::Write] object.
#[deprecated(
    since = "0.3.0",
    note = "use `Encoder::default().encode_stream(input, output)`"
)]
pub fn encode_stream(
    input: &mut impl std::io::Read,
    output: &mut impl std::io::Write,
) -> std::io::Result<()> {
    Encoder::new(Charset::Utf8).encode_stream(input, output)
}

/// Encode ASCII characters from a [Read][std::io::Read] object into a [Write][std::io::Write]
/// object.
#[deprecated(
    since = "0.3.0",
    note = "use `Encoder::new(Charset::Ascii).encode_stream(input, output)`"
)]
pub fn encode_stream_ascii(
    input: &mut impl std::io::Read,
    output: &mut impl std::io::Write,
) -> std::io::Result<()> {
    Encoder::new(Charset::Ascii).encode_stream(input, output)
}

/// Decode Morse code from a [byte slice][slice] into into a [String].
#[deprecated(
    since = "0.3.0",
    note = "use `Decoder::new(alphabet).decode(input)`, or `Decoder::from_fn(char_decode)` for a custom alphabet"
)]
pub fn decode_string(input: &[u8], char_decode: fn(u8) -> char) -> String {
    Decoder::from_fn(char_decode).decode(input)
}

/// Decode Morse code from a [Read][std::io::Read] object into a [Write][std::io::Write] object.
#[deprecated(
    since = "0.3.0",
    note = "use `Decoder::new(alphabet).decode_stream(input, output)`, or `Decoder::from_fn(char_decode)` for a custom alphabet"
)]
pub fn decode_stream(
    input: &mut impl std::io::Read,
    output: &mut impl std::io::Write,
    char_decode: fn(u8) -> char,
) -> std::io::Result<()> {
    Decoder::from_fn(char_decode).decode_stream(input, output)
}

#[test]
fn test_unicode_round_trip() {
    let f = |s: &str| Decoder::default().decode(Encoder::default().encode(s.as_bytes()).as_bytes());
    assert_eq!(f("paris"), "PARIS");
    assert_eq!(f("Hello, World!"), "HELLO, WORLD!");
    assert_eq!(
        f("one line\nand  another\tline"),
        "ONE LINE\nAND  ANOTHER\tLINE"
    );
    assert_eq!(f("trailing SPACE "), "TRAILING SPACE ");
}
