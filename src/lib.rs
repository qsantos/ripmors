mod mappings;

use std::io::{BufWriter, Write};

use mappings::{ascii_to_morse, morse_to_ascii, unicode_to_morse};

pub fn ascii_encode_to_writer<W: Write>(writer: &mut W, s: &[u8]) -> Result<(), std::io::Error> {
    writer.write_all(ascii_to_morse(s[0] as char).as_bytes())?;
    for c in &s[1..] {
        let morse = ascii_to_morse(*c as char);
        if !morse.is_empty() {
            writer.write_all(b" ")?;
            writer.write_all(morse.as_bytes())?;
        }
    }
    Ok(())
}

pub fn ascii_encode_to_string(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    ascii_encode_to_writer(&mut writer, s.as_bytes()).unwrap();
    String::from_utf8(writer.into_inner().unwrap()).unwrap()
}

pub fn unicode_encode_to_writer<W: Write>(writer: &mut W, s: &str) -> Result<(), std::io::Error> {
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        writer.write_all(unicode_to_morse(c).as_bytes())?;
    }
    for c in chars {
        let morse = unicode_to_morse(c);
        if !morse.is_empty() {
            writer.write_all(b" ")?;
            writer.write_all(morse.as_bytes())?;
        }
    }
    Ok(())
}

pub fn unicode_encode_to_string(s: &str) -> String {
    let parts: Vec<&str> = s
        .chars()
        .map(unicode_to_morse)
        .filter(|&x| !x.is_empty())
        .collect();
    parts.join(" ")
}

pub fn ascii_decode(s: &str) -> String {
    let parts: Vec<&str> = s
        .split(' ')
        .map(morse_to_ascii)
        .filter(|&x| !x.is_empty())
        .collect();
    parts.join("")
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
fn test_unicode_encode() {
    assert_eq!(
        unicode_encode_to_string("télégraphie"),
        "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. ."
    );
    assert_eq!(
        unicode_encode_to_string("でんしん"),
        ".-.-- .. .-.-. --.-. .-.-."
    );
    assert_eq!(unicode_encode_to_string("تلغراف"), "- .-.. --. .-. .- ..-.");
    assert_eq!(
        unicode_encode_to_string("телеграфия"),
        "- . .-.. . --. .-. .- ..-. .. .-.-"
    );
    assert_eq!(
        unicode_encode_to_string("τηλεγραφία"),
        "- .... .-.. . --. .-. .- ..-. .-"
    );
}

#[test]
fn test_ascii_decode() {
    assert_eq!(ascii_decode(".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        ascii_decode(".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."),
        "HELLO, WORLD!",
    );
}
