#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
//! # Feature Flags
//! There is a single feature flag, `clap`. It is only used for the binary. It only exists to
//! reduce the dependencies of the libraries without splitting the crate.

// Do not consider the body of unsafe functions to be an unsafe block by default. This forces one
// to explicitly mark what parts of an unsafe functions are actually unsafe.
#![warn(unsafe_op_in_unsafe_fn)]
// Enforce "SAFETY:" comments before unsafe blocks
#![warn(clippy::undocumented_unsafe_blocks)]
// Enforce "# Safety" section in documentation of unsafe functions
#![warn(clippy::missing_safety_doc)]

mod decode;
mod decode_mapping;
mod encode_ascii;
mod encode_ascii_mapping;
mod encode_unicode;
mod encode_unicode_mapping;

// Public API
pub use decode::{decode_stream, decode_string};
pub use decode_mapping::{
    to_arabic, to_greek, to_hebrew, to_japanese, to_korean, to_russian, to_standard,
};
pub use encode_ascii::{encode_stream_ascii, encode_string_ascii};
pub use encode_unicode::{encode_stream, encode_string};

#[test]
fn test_unicode_round_trip() {
    let f = |s| decode_string(encode_string(s).as_bytes(), to_standard);
    assert_eq!(f("paris"), "PARIS");
    assert_eq!(f("Hello, World!"), "HELLO, WORLD!");
    assert_eq!(
        f("one line\nand  another\tline"),
        "ONE LINE\nAND  ANOTHER\tLINE"
    );
    assert_eq!(f("trailing SPACE "), "TRAILING SPACE ");
}
