mod mappings;

use std::io::{BufWriter, Write};

pub use mappings::*;

pub fn ascii_encode_to_writer<W: Write>(writer: &mut W, s: &[u8]) -> Result<(), std::io::Error> {
    for c in s {
        let morse = ascii_to_morse(*c as char);
        if !morse.is_empty() {
            writer.write_all(morse.as_bytes())?;
        }
    }
    Ok(())
}

pub fn ascii_encode_to_string(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    ascii_encode_to_writer(&mut writer, s.as_bytes()).unwrap();
    let mut vec = writer.into_inner().unwrap();
    if vec.last() == Some(&b' ') {
        vec.pop();
    }
    String::from_utf8(vec).unwrap()
}

pub fn standard_encode_to_writer<W: Write>(writer: &mut W, s: &str) -> Result<(), std::io::Error> {
    for c in s.chars() {
        let morse = standard_to_morse(c);
        if !morse.is_empty() {
            writer.write_all(morse.as_bytes())?;
        }
    }
    Ok(())
}

pub fn standard_encode_to_string(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    standard_encode_to_writer(&mut writer, s).unwrap();
    let mut vec = writer.into_inner().unwrap();
    if vec.last() == Some(&b' ') {
        vec.pop();
    }
    String::from_utf8(vec).unwrap()
}

macro_rules! make_decode {
    ($name:ident, $char_decode:ident) => {
        pub fn $name(s: &str) -> String {
            s.split(' ')
                .map($char_decode)
                .filter(|&x| x != '\0')
                .collect()
        }
    };
}

make_decode!(standard_decode, morse_to_standard);
make_decode!(greek_decode, morse_to_greek);
make_decode!(russian_decode, morse_to_russian);
make_decode!(japanese_decode, morse_to_japanese);
make_decode!(korean_decode, morse_to_korean);
make_decode!(hebrew_decode, morse_to_hebrew);
make_decode!(arabic_decode, morse_to_arabic);

#[test]
fn test_ascii_encode() {
    assert_eq!(ascii_encode_to_string("PARIS"), ".--. .- .-. .. ...");
    assert_eq!(
        ascii_encode_to_string("Hello, World!"),
        ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."
    );
}

#[test]
fn test_standard_encode() {
    assert_eq!(
        standard_encode_to_string("télégraphie"),
        "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. ."
    );
    assert_eq!(
        standard_encode_to_string("でんしん"),
        ".-.-- .. .-.-. --.-. .-.-."
    );
    assert_eq!(
        standard_encode_to_string("تلغراف"),
        "- .-.. --. .-. .- ..-."
    );
    assert_eq!(
        standard_encode_to_string("телеграфия"),
        "- . .-.. . --. .-. .- ..-. .. .-.-"
    );
    assert_eq!(
        standard_encode_to_string("τηλεγραφία"),
        "- .... .-.. . --. .-. .- ..-. .. .-"
    );
    assert_eq!(
        standard_encode_to_string("one line\nand  another\tline"),
        "--- -. . / .-.. .. -. . \n.- -. -.. / / .- -. --- - .... . .-. \t.-.. .. -. ."
    );
}

#[test]
fn test_standard_decode() {
    assert_eq!(standard_decode(".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        standard_decode(".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."),
        "HELLO, WORLD!",
    );
}
