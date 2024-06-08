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
pub use encode_ascii::{ascii_encode_to_string, encode_stream_ascii};
pub use encode_unicode::{encode_stream_unicode, unicode_encode_to_string};

#[test]
fn test_unicode_round_trip() {
    let f = |s| decode_string(unicode_encode_to_string(s).as_bytes(), to_standard);
    assert_eq!(f("paris"), "PARIS");
    assert_eq!(f("Hello, World!"), "HELLO, WORLD!");
    assert_eq!(
        f("one line\nand  another\tline"),
        "ONE LINE\nAND  ANOTHER\tLINE"
    );
    assert_eq!(f("trailing SPACE "), "TRAILING SPACE ");
}
