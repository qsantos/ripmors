use std::io::{Read, Write};

#[inline(always)] // prefer inline to avoid reloading constants in registers
fn morse_to_binary_fast(bytes: &[u8; 8], len: usize) -> u8 {
    // Interpret next 8 bytes as u64
    // SAFETY: `bytes` is a reference to a slice of 8 initialized bytes
    let a = unsafe { (bytes.as_ptr() as *const u64).read_unaligned() };
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
    // zero length
    assert_eq!(morse_to_binary_fast(b"________", 0), 1);
    assert_eq!(morse_to_binary_fast(b"..._____", 0), 1);
    assert_eq!(morse_to_binary_fast(b"---_____", 0), 1);

    // non-zero length
    assert_eq!(morse_to_binary_fast(b"._______", 1), 0b10);
    assert_eq!(morse_to_binary_fast(b"-_______", 1), 0b11);
    assert_eq!(morse_to_binary_fast(b"..-.____", 4), 0b10100);
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
        // SAFETY: the above condition ensures that the pointer is valid and points to 8
        // initialized bytes
        let eight_bytes: &[u8; 8] = unsafe { &*bytes.as_ptr().cast() };
        morse_to_binary_fast(eight_bytes, len)
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

fn decode_buffer(input: &[u8], char_decode: fn(u8) -> char, output_buf: &mut Vec<char>) -> usize {
    let mut chunk_start = 0;
    let last_seven_bytes = input.len().saturating_sub(7);
    for i in 0..last_seven_bytes {
        let c = input[i];
        if c <= b' ' {
            // SAFETY: `chunk_start < i < input.len() - 7` so the first argument is always a valid
            // pointer to eight initialized bytes
            let eight_bytes: &[u8; 8] = unsafe { &*input.as_ptr().add(chunk_start).cast() };
            let binary = morse_to_binary_fast(eight_bytes, i - chunk_start);
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
    chunk_start
}

fn decode_buffer_end(input: &[u8], char_decode: fn(u8) -> char, output_buf: &mut Vec<char>) {
    let chunk_start = decode_buffer(input, char_decode, output_buf);
    let binary = morse_to_binary(&input[chunk_start..], input.len() - chunk_start);
    let decoded = char_decode(binary);
    if decoded != '\0' {
        output_buf.push(decoded);
    }
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
    let mut output_buf = Vec::with_capacity(input.len());
    decode_buffer_end(input, char_decode, &mut output_buf);
    output_buf.iter().collect()
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
pub fn decode_stream(
    input: &mut impl Read,
    output: &mut impl Write,
    char_decode: fn(u8) -> char,
) -> Result<(), std::io::Error> {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut output_buf = Vec::with_capacity(1 << 15);
    loop {
        let bytes_read = input.read(&mut input_buf[bytes_available..])?;
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;

        let bytes_used = decode_buffer(&input_buf[..bytes_available], char_decode, &mut output_buf);

        // flush buffer
        if !output_buf.is_empty() {
            let decoded: String = output_buf.iter().collect();
            output.write_all(decoded.as_bytes())?;
            output_buf.clear();
        }

        input_buf.copy_within(bytes_used..bytes_available, 0);
        bytes_available -= bytes_used;
    }

    if bytes_available != 0 {
        decode_buffer_end(&input_buf[..bytes_available], char_decode, &mut output_buf);
        if !output_buf.is_empty() {
            let decoded: String = output_buf.iter().collect();
            output.write_all(decoded.as_bytes())?;
        }
    }

    Ok(())
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
