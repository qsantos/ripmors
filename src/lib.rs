mod mappings;

use std::io::{BufWriter, Read, Write};

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

// TODO: unify with morse_decode_buffer
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

pub fn morse_decode_buffer<F: Fn(&str) -> char>(s: &str, char_decode: &F) -> (String, usize) {
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
    (vec.into_iter().collect(), chunk_start)
}

pub fn morse_decode_to_writer<W: Write, F: Fn(&str) -> char>(
    writer: &mut W,
    s: &str,
    char_decode: &F,
) -> Result<usize, std::io::Error> {
    let (decoded, bytes_used) = morse_decode_buffer(s, char_decode);
    writer.write_all(decoded.as_bytes()).map(|_| bytes_used)
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

pub fn encode_stream<R: Read, W: Write>(i: &mut R, o: &mut W) {
    let mut input_buf = vec![0u8; 1 << 15];
    loop {
        let n = i.read(&mut input_buf).unwrap();
        if n == 0 {
            break;
        }
        ascii_encode_to_writer(o, &input_buf[..n]).unwrap();
    }
}

pub fn decode_stream<R: Read, W: Write, F: Fn(&str) -> char>(
    i: &mut R,
    o: &mut W,
    char_decode: &F,
) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    loop {
        let bytes_read = i.read(&mut input_buf[bytes_available..]).unwrap();
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;
        let s = match std::str::from_utf8(&input_buf[..bytes_available]) {
            Ok(s) => s,
            Err(e) => {
                let bytes_decoded = e.valid_up_to();
                unsafe { std::str::from_utf8_unchecked(&input_buf[..bytes_decoded]) }
            }
        };

        // TODO: decode last character at end of input
        let bytes_used = morse_decode_to_writer(o, s, char_decode).unwrap();

        input_buf.copy_within(bytes_used..bytes_available, 0);
        bytes_available -= bytes_used;
    }
}
