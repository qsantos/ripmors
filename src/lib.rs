mod decode;
mod decode_mapping;
mod encode_ascii;
mod encode_ascii_mapping;
mod encode_standard;
mod encode_standard_mapping;

// Public API
pub use decode::{decode_stream, morse_decode_to_string};
pub use decode_mapping::{
    morse_to_arabic, morse_to_greek, morse_to_hebrew, morse_to_japanese, morse_to_korean,
    morse_to_russian, morse_to_standard,
};
pub use encode_ascii::{ascii_encode_to_string, encode_stream_ascii};
pub use encode_standard::{encode_stream_standard, standard_encode_to_string};

#[test]
fn test_standard_round_trip() {
    let f = |s| morse_decode_to_string(standard_encode_to_string(s).as_bytes(), morse_to_standard);
    assert_eq!(f("paris"), "PARIS");
    assert_eq!(f("Hello, World!"), "HELLO, WORLD!");
    assert_eq!(
        f("one line\nand  another\tline"),
        "ONE LINE\nAND  ANOTHER\tLINE"
    );
    assert_eq!(f("trailing SPACE "), "TRAILING SPACE ");
}
