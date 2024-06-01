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

pub fn morse_decode_to_string<F: Fn(&str) -> char>(s: &str, char_decode: &F) -> String {
    let mut vec = Vec::new();
    let mut chunk_start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '\t' | '\n' | '\r' => {
                let decoded = char_decode(&s[chunk_start..i]);
                if decoded != '\0' {
                    vec.push(decoded);
                }
                chunk_start = i + 1;
                vec.push(c);
            }
            ' ' => {
                let decoded = char_decode(&s[chunk_start..i]);
                if decoded != '\0' {
                    vec.push(decoded);
                }
                chunk_start = i + 1;
            }
            _ => (),
        }
    }
    vec.push(char_decode(&s[chunk_start..]));
    vec.into_iter().collect()
}

pub fn morse_decode_to_writer<W: Write, F: Fn(&str) -> char>(
    writer: &mut W,
    s: &str,
    char_decode: &F,
) -> Result<(), std::io::Error> {
    let decoded = morse_decode_to_string(s, &char_decode);
    writer.write_all(decoded.as_bytes())
}

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
    let f = |s| morse_decode_to_string(s, &morse_to_standard);
    assert_eq!(f(".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        f(".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."),
        "HELLO, WORLD!",
    );
}

#[test]
fn test_standard_encode_decode() {
    let f = |s| morse_decode_to_string(&standard_encode_to_string(s), &morse_to_standard);
    assert_eq!(f("paris"), "PARIS");
    assert_eq!(f("Hello, World!"), "HELLO, WORLD!");
    assert_eq!(
        f("one line\nand  another\tline"),
        "ONE LINE\nAND  ANOTHER\tLINE"
    );
}
