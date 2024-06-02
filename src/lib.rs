mod mappings;

use std::io::{BufWriter, Read, Write};

pub use mappings::*;

pub fn ascii_encode_to_writer<W: Write>(
    writer: &mut W,
    s: &[u8],
    need_separator: &mut bool,
) -> Result<(), std::io::Error> {
    for c in s {
        if *c == b'\t' || *c == b'\n' || *c == b'\r' {
            writer.write_all(&[*c])?;
            *need_separator = false;
        } else {
            let morse = ascii_to_morse(*c as char);
            if !morse.is_empty() {
                if *need_separator {
                    writer.write_all(b" ")?;
                }
                writer.write_all(morse.as_bytes())?;
                *need_separator = true;
            }
        }
    }
    Ok(())
}

pub fn ascii_encode_to_string(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    ascii_encode_to_writer(&mut writer, s.as_bytes(), &mut false).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

pub fn standard_encode_to_writer<W: Write>(
    writer: &mut W,
    s: &str,
    need_separator: &mut bool,
) -> Result<(), std::io::Error> {
    for c in s.chars() {
        if c == '\t' || c == '\n' || c == '\r' {
            writer.write_all(&[c as u8])?;
            *need_separator = false;
        } else {
            let morse = standard_to_morse(c);
            if !morse.is_empty() {
                if *need_separator {
                    writer.write_all(b" ")?;
                    *need_separator = false;
                }
                writer.write_all(morse.as_bytes())?;
                *need_separator = true;
            }
        }
    }
    Ok(())
}

pub fn standard_encode_to_string(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    standard_encode_to_writer(&mut writer, s, &mut false).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

pub fn morse_decode_to_writer<W: Write, F: Fn(&[u8]) -> char>(
    writer: &mut W,
    s: &[u8],
    char_decode: &F,
) -> Result<usize, std::io::Error> {
    let mut vec = Vec::new();
    let mut chunk_start = 0;
    for (i, c) in s.iter().enumerate() {
        if *c == b'\t' || *c == b'\n' || *c == b'\r' || *c == b' ' {
            let decoded = char_decode(&s[chunk_start..i]);
            if decoded != '\0' {
                vec.push(decoded);
            }
            chunk_start = i + 1;
            if *c != b' ' {
                vec.push(*c as char);
            }
        }
    }
    let decoded: String = vec.into_iter().collect();
    writer.write_all(decoded.as_bytes()).map(|_| chunk_start)
}

pub fn morse_decode_to_writer_end<W: Write, F: Fn(&[u8]) -> char>(
    writer: &mut W,
    s: &[u8],
    char_decode: &F,
) -> Result<(), std::io::Error> {
    let chunk_start = morse_decode_to_writer(writer, s, char_decode)?;
    let decoded = char_decode(&s[chunk_start..]);
    if decoded != '\0' {
        writer.write_all(decoded.to_string().as_bytes())?;
    }
    Ok(())
}

pub fn morse_decode_to_string<F: Fn(&[u8]) -> char>(s: &[u8], char_decode: &F) -> String {
    let mut writer = BufWriter::new(Vec::new());
    morse_decode_to_writer_end(&mut writer, s, char_decode).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
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
        "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
    );
}

#[test]
fn test_standard_decode() {
    let f = |s| morse_decode_to_string(s, &morse_to_standard);
    assert_eq!(f(b".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        f(b".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."),
        "HELLO, WORLD!",
    );
}

#[test]
fn test_standard_encode_decode() {
    let f =
        |s| morse_decode_to_string(&standard_encode_to_string(s).as_bytes(), &morse_to_standard);
    assert_eq!(f("paris"), "PARIS");
    assert_eq!(f("Hello, World!"), "HELLO, WORLD!");
    assert_eq!(
        f("one line\nand  another\tline"),
        "ONE LINE\nAND  ANOTHER\tLINE"
    );
}

pub fn encode_stream<R: Read, W: Write>(i: &mut R, o: &mut W) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut need_separator = false;
    loop {
        let n = i.read(&mut input_buf).unwrap();
        if n == 0 {
            break;
        }
        ascii_encode_to_writer(o, &input_buf[..n], &mut need_separator).unwrap();
    }
}

pub fn decode_stream<R: Read, W: Write, F: Fn(&[u8]) -> char>(
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

        let bytes_used =
            morse_decode_to_writer(o, &input_buf[..bytes_available], char_decode).unwrap();

        input_buf.copy_within(bytes_used..bytes_available, 0);
        bytes_available -= bytes_used;
    }

    if bytes_available != 0 {
        morse_decode_to_writer_end(o, &input_buf[..bytes_available], char_decode).unwrap();
    }
}
