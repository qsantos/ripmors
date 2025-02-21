use std::io::{Read, Write};

use crate::encode_ascii_mapping::ASCII_TO_QWORD;

fn encode_buffer_ascii(input: &[u8], output_buf: &mut Vec<u8>) {
    let mut cur = output_buf.len();
    output_buf.reserve(input.len() * 18 + cur);
    for c in input {
        let (bytes, len) = ASCII_TO_QWORD[*c as usize];
        if len == 0 {
        } else if len <= 8 {
            if (*c == b'\t' || *c == b'\n' || *c == b'\r')
                && cur > 0
                // SAFETY: accessing `output_buf[cur - 1]` is safe because we only increase cur
                // after writing to `output_buf` and `cur > 0` is checked on the previous line
                && unsafe { *(output_buf.as_ptr().add(cur - 1)) } == b' '
            {
                cur -= 1;
            }
            // SAFETY: we reserved 18 bytes in output_buf for each byte in input
            unsafe {
                let dst = output_buf.as_mut_ptr().add(cur) as *mut u64;
                dst.write_unaligned(bytes);
            }
        } else {
            // handle only ASCII character encoded as more than 7 elements + space
            assert_eq!(*c, b'%');
            // SAFETY: we reserved 18 bytes in output_buf for each byte in input
            unsafe { std::slice::from_raw_parts_mut(output_buf.as_mut_ptr().add(cur), 18) }
                .copy_from_slice(b"----- -..-. ----- ");
        }
        cur += len;
    }
    // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase cur
    // after writing to `output_buf`
    unsafe { output_buf.set_len(cur) };
}

/// Encode ASCII characters from a [byte slice][slice] into a [String].
///
/// Bytes from `input` are interpreted as ASCII characters. The following ASCII characters are used to
/// represent Morse code:
///
/// - Full stop (.) represents the Morse dot;
/// - Hyphen (-) represents the Morse dash;
/// - Space ( ) represents the letter space;
/// - Slash (/) represents the word space;
/// - Tab (\t), line feed (\n) and carriage return (\r) are kept as-is.
///
/// ASCII characters that cannot be converted to Morse and non-ASCII bytes, such as UTF-8 encodings, are ignored.
///
/// For example, the program below encodes ASCII input.
/// ```
/// let morse = ripmors::encode_string_ascii(b"Morse code");
/// assert_eq!(morse, "-- --- .-. ... . / -.-. --- -.. .");
/// ```
pub fn encode_string_ascii(input: &[u8]) -> String {
    let mut output_buf = Vec::new();
    encode_buffer_ascii(input, &mut output_buf);
    if output_buf.last() == Some(&b' ') {
        output_buf.pop();
    }
    // SAFETY: encode_buffer_ascii only outputs ASCII, so it is valid UTF-8
    unsafe { String::from_utf8_unchecked(output_buf) }
}

/// Encode ASCII characters from a [`Read`][std::io::Read] object into a [`Write`][std::io::Write] object.
///
/// Bytes from `input` are interpreted as ASCII characters. The following ASCII characters are used to
/// represent Morse code:
///
/// - Full stop (`.`) represents the Morse dot;
/// - Hyphen (`-`) represents the Morse dash;
/// - Space (` `) represents the letter space;
/// - Slash (`/`) represents the word space;
/// - Tab (`\t`), line feed (`\n`) and carriage return (`\r`) are kept as-is.
///
/// ASCII characters that cannot be converted to Morse and non-ASCII bytes, such as UTF-8 encodings, are ignored.
///
/// **Note:** This will read data from `input` until exhaustion.
///
/// For example, the program below encodes ASCII input.
/// ```no_run
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     let mut stdin = std::io::stdin();
///     let mut stdout = std::io::stdout();
///     ripmors::encode_stream_ascii(&mut stdin, &mut stdout);
/// }
/// ```
pub fn encode_stream_ascii(
    input: &mut impl Read,
    output: &mut impl Write,
) -> Result<(), std::io::Error> {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut output_buf = Vec::new();
    loop {
        let bytes_read = input.read(&mut input_buf)?;
        if bytes_read == 0 {
            break;
        }
        encode_buffer_ascii(&input_buf[..bytes_read], &mut output_buf);
        if output_buf.is_empty() {
        } else if output_buf.last() == Some(&b' ') {
            output_buf.pop();
            output.write_all(&output_buf)?;
            output_buf.clear();
            output_buf.push(b' ');
        } else {
            output.write_all(&output_buf)?;
            output_buf.clear();
        }
    }
    Ok(())
}

#[test]
fn test_ascii_encode_simple() {
    assert_eq!(encode_string_ascii(b"PARIS"), ".--. .- .-. .. ...");
    assert_eq!(
        encode_string_ascii(b"Hello, World!"),
        ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--"
    );
    assert_eq!(
        encode_string_ascii(b"one line\nand  another\tline"),
        "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
    );
}

// short enough to run with Miri
#[test]
fn test_ascii_encode_random_short() {
    use rand::{Rng, distributions::Standard};
    let data: Vec<u8> = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1024)
        .collect();
    encode_string_ascii(&data);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_ascii_encode_random_large() {
    use rand::{Rng, distributions::Standard};
    let data: Vec<u8> = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1048576)
        .collect();
    encode_string_ascii(&data);
}
