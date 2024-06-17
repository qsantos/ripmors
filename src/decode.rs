use std::io::{BufWriter, Read, Write};

#[inline(always)] // prefer inline to avoid reloading constants in registers
unsafe fn morse_to_binary_fast(bytes: *const u8, len: usize) -> u8 {
    // Interpret next 8 bytes as u64
    let a = unsafe { (bytes as *const u64).read_unaligned() };
    // Only keep the LSB of each byte
    let b = 0x0101010101010101;
    let a = a & b;
    // Pack the bits together
    let a = a.wrapping_mul(0x102040810204080) >> 56;
    // Truncate to len lowest significant bits
    let a = a & !(0xff << len);
    // Add a leading one to distinguish e.g. - from .-, ..-, ...- and ....-
    // NOTE: we use "b" instead of "1" to avoid having to load another immediate in a register.
    // Since the lowest byte of b is 0x01 and we only keep the last byte, it works.
    let a = a | (b << len);
    // Done
    a as u8
}

#[test]
fn test_morse_to_binary_fast() {
    unsafe {
        // zero length
        assert_eq!(morse_to_binary_fast(b"________".as_ptr(), 0), 1);
        assert_eq!(morse_to_binary_fast(b"..._____".as_ptr(), 0), 1);
        assert_eq!(morse_to_binary_fast(b"---_____".as_ptr(), 0), 1);

        // non-zero length
        assert_eq!(morse_to_binary_fast(b"._______".as_ptr(), 1), 0b10);
        assert_eq!(morse_to_binary_fast(b"-_______".as_ptr(), 1), 0b11);
        assert_eq!(morse_to_binary_fast(b"..-.____".as_ptr(), 4), 0b10100);
    }
}

fn morse_to_binary_safe(bytes: &[u8], len: usize) -> u8 {
    let mut ret = 1;
    for byte in bytes[..len].iter().rev() {
        ret *= 2;
        ret |= byte & 1;
    }
    ret
}

#[test]
fn test_morse_to_binary_safe() {
    // zero length
    assert_eq!(morse_to_binary_safe(b"", 0), 1);
    assert_eq!(morse_to_binary_safe(b"...", 0), 1);
    assert_eq!(morse_to_binary_safe(b"---", 0), 1);

    // non-zero length
    assert_eq!(morse_to_binary_safe(b".", 1), 0b10);
    assert_eq!(morse_to_binary_safe(b"-", 1), 0b11);
    assert_eq!(morse_to_binary_safe(b"..-.", 4), 0b10100);
}

fn morse_to_binary(bytes: &[u8], len: usize) -> u8 {
    if len + 8 <= bytes.len() {
        unsafe { morse_to_binary_fast(bytes.as_ptr(), len) }
    } else {
        morse_to_binary_safe(bytes, len)
    }
}

#[test]
fn test_morse_to_binary() {
    // this is just the same tests as for test_morse_to_binary_fast and test_morse_to_binary_safe;
    // not very imaginative, but it's fast and we'll have covered everything

    // fast-friendly
    // zero length
    assert_eq!(morse_to_binary(b"________", 0), 1);
    assert_eq!(morse_to_binary(b"..._____", 0), 1);
    assert_eq!(morse_to_binary(b"---_____", 0), 1);

    // non-zero length
    assert_eq!(morse_to_binary(b"._______", 1), 0b10);
    assert_eq!(morse_to_binary(b"-_______", 1), 0b11);
    assert_eq!(morse_to_binary(b"..-.____", 4), 0b10100);

    // fast-unfriendly
    // zero length
    assert_eq!(morse_to_binary(b"", 0), 1);
    assert_eq!(morse_to_binary(b"...", 0), 1);
    assert_eq!(morse_to_binary(b"---", 0), 1);

    // non-zero length
    assert_eq!(morse_to_binary(b".", 1), 0b10);
    assert_eq!(morse_to_binary(b"-", 1), 0b11);
    assert_eq!(morse_to_binary(b"..-.", 4), 0b10100);
}

fn decode_buffer(
    input: &[u8],
    char_decode: fn(u8) -> char,
    output_buf: &mut Vec<char>,
) -> Result<usize, std::io::Error> {
    let mut chunk_start = 0;
    let last_seven_bytes = input.len().saturating_sub(7);
    for i in 0..last_seven_bytes {
        let c = input[i];
        if c <= b' ' {
            let binary =
                unsafe { morse_to_binary_fast(input.as_ptr().add(chunk_start), i - chunk_start) };
            let decoded = char_decode(binary);
            if decoded != '\0' {
                output_buf.push(decoded);
            }
            chunk_start = i + 1;
            if c != b' ' {
                output_buf.push(c as char);
            }
        } else if c == b'/' {
            output_buf.push(' ');
            chunk_start = i + 1;
        }
    }
    for i in last_seven_bytes..input.len() {
        let c = input[i];
        if c <= b' ' {
            let binary = morse_to_binary(&input[chunk_start..], i - chunk_start);
            let decoded = char_decode(binary);
            if decoded != '\0' {
                output_buf.push(decoded);
            }
            chunk_start = i + 1;
            if c != b' ' {
                output_buf.push(c as char);
            }
        } else if c == b'/' {
            output_buf.push(' ');
            chunk_start = i + 1;
        }
    }
    Ok(chunk_start)
}

fn decode_buffer_end(
    output: &mut impl Write,
    input: &[u8],
    char_decode: fn(u8) -> char,
) -> Result<(), std::io::Error> {
    let mut output_buf = Vec::with_capacity(input.len());
    let chunk_start = decode_buffer(input, char_decode, &mut output_buf)?;
    let binary = morse_to_binary(&input[chunk_start..], input.len() - chunk_start);
    let decoded = char_decode(binary);
    if decoded != '\0' {
        output_buf.push(decoded);
    }
    if !output_buf.is_empty() {
        let decoded: String = output_buf.iter().collect();
        output.write_all(decoded.as_bytes())?;
    }
    Ok(())
}

/// Decode Morse code from a [byte slice][slice] into into a [String].
///
/// Bytes from `input` are interpreted as ASCII characters.
///
/// - Full stop (.) is interpreted as Morse dot;
/// - Hyphen (-) is interpreted as Morse dash;
/// - Space ( ) is interpreted as letter space;
/// - Slash (/) is interpreted as word space;
/// - Tab (\t), line feed (\n) and carriage return (\r) are kept as-is.
///
/// Other ASCII characters, and non-ASCII bytes, such as UTF-8 encodings, are ignored.
///
/// The second argument selects a local variant of Morse code. It should be one of:
///
/// - [to_standard][crate::to_standard] for [International Morse code and Latin extensions](https://en.wikipedia.org/wiki/Morse_code#Letters,_numbers,_punctuation,_prosigns_for_Morse_code_and_non-Latin_variants);
/// - [to_arabic][crate::to_arabic] for Arabic;
/// - [to_greek][crate::to_greek] for Greek;
/// - [to_hebrew][crate::to_hebrew] for Hebrew;
/// - [to_japanese][crate::to_japanese] for [Japanese](https://en.wikipedia.org/wiki/Wabun_code) (Katakana);
/// - [to_korean][crate::to_korean] for [Korean](https://en.wikipedia.org/wiki/SKATS) (Hangul);
/// - [to_russian][crate::to_russian] for [Russian](https://en.wikipedia.org/wiki/Russian_Morse_code) (Cyrillic).
///
/// ```
/// let morse = "-- --- .-. ... . / -.-. --- -.. .";
/// let string = ripmors::decode_string(morse.as_bytes(), ripmors::to_standard);
/// assert_eq!(string, "MORSE CODE");
/// ```
pub fn decode_string(input: &[u8], char_decode: fn(u8) -> char) -> String {
    let mut writer = BufWriter::new(Vec::new());
    decode_buffer_end(&mut writer, input, char_decode).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

/// Decode Morse code from a [Read][std::io::Read] object into a [Write][std::io::Write] object.
///
/// Bytes from `input` are interpreted as ASCII characters.
///
/// - Full stop (.) is interpreted as Morse dot;
/// - Hyphen (-) is interpreted as Morse dash;
/// - Space ( ) is interpreted as letter space;
/// - Slash (/) is interpreted as word space;
/// - Tab (\t), line feed (\n) and carriage return (\r) are kept as-is.
///
/// Other ASCII characters, and non-ASCII bytes, such as UTF-8 encodings, are ignored.
///
/// **Note:** This will read data from `input` until exhaustion.
///
/// The third argument selects a local variant of Morse code. It should be one of:
///
/// - [to_standard][crate::to_standard] for [International Morse code and Latin extensions](https://en.wikipedia.org/wiki/Morse_code#Letters,_numbers,_punctuation,_prosigns_for_Morse_code_and_non-Latin_variants);
/// - [to_arabic][crate::to_arabic] for Arabic;
/// - [to_greek][crate::to_greek] for Greek;
/// - [to_hebrew][crate::to_hebrew] for Hebrew;
/// - [to_japanese][crate::to_japanese] for [Japanese](https://en.wikipedia.org/wiki/Wabun_code) (Katakana);
/// - [to_korean][crate::to_korean] for [Korean](https://en.wikipedia.org/wiki/SKATS) (Hangul);
/// - [to_russian][crate::to_russian] for [Russian](https://en.wikipedia.org/wiki/Russian_Morse_code) (Cyrillic).
///
/// For example, the program below decodes international Morse code.
/// ```no_run
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     let mut stdin = std::io::stdin();
///     let mut stdout = std::io::stdout();
///     ripmors::decode_stream(&mut stdin, &mut stdout, ripmors::to_standard);
/// }
/// ```
pub fn decode_stream(input: &mut impl Read, output: &mut impl Write, char_decode: fn(u8) -> char) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut output_buf = Vec::with_capacity(1 << 15);
    loop {
        let bytes_read = input.read(&mut input_buf[bytes_available..]).unwrap();
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;

        let bytes_used =
            decode_buffer(&input_buf[..bytes_available], char_decode, &mut output_buf).unwrap();

        // flush buffer
        if !output_buf.is_empty() {
            let decoded: String = output_buf.iter().collect();
            output.write_all(decoded.as_bytes()).unwrap();
            output_buf.clear();
        }

        input_buf.copy_within(bytes_used..bytes_available, 0);
        bytes_available -= bytes_used;
    }

    if bytes_available != 0 {
        decode_buffer_end(output, &input_buf[..bytes_available], char_decode).unwrap();
    }
}

#[test]
fn test_standard_decode() {
    use crate::decode_mapping::to_standard;
    let f = |s| decode_string(s, to_standard);
    assert_eq!(f(b".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        f(b".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--"),
        "HELLO, WORLD!",
    );
}
