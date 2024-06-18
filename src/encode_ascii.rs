use std::io::{BufWriter, Read, Write};

use crate::encode_ascii_mapping::ASCII_TO_QWORD;

fn encode_buffer_ascii(
    output: &mut impl Write,
    input: &[u8],
    need_separator: &mut bool,
    output_buf: &mut [u8; 1 << 15],
) -> Result<(), std::io::Error> {
    let mut cur = 0;
    if *need_separator {
        output_buf[cur] = b' ';
        cur += 1;
    }
    for chunk in input.chunks(1 << 10) {
        for c in chunk {
            let (bytes, len) = ASCII_TO_QWORD[*c as usize];
            if len == 0 {
            } else if len <= 8 {
                if (*c == b'\t' || *c == b'\n' || *c == b'\r')
                    && cur > 0
                    && output_buf[cur - 1] == b' '
                {
                    cur -= 1;
                }
                // SAFETY: each byte of the chunk might advance `cur` by up to 18; we flush the
                // buffer after each chunk if we cannot guarantee 18 bytes per input byte for the
                // next chunk; thus, there is at least 8 available bytes for writing after
                // `output_buf + cur`.
                unsafe {
                    let dst = output_buf.as_mut_ptr().add(cur) as *mut u64;
                    dst.write_unaligned(bytes);
                }
            } else {
                // handle only ASCII character encoded as more than 7 elements + space
                assert_eq!(*c, b'%');
                output_buf[cur..cur + 18].copy_from_slice(b"----- -..-. ----- ");
            }
            cur += len;
        }
        // flush buffer
        if cur >= output_buf.len() - (1 << 10) * 18 {
            if output_buf[cur - 1] == b' ' {
                cur -= 1;
                output.write_all(&output_buf[..cur])?;
                output_buf[0] = b' ';
                cur = 1;
            } else {
                output.write_all(&output_buf[..cur])?;
                cur = 0;
            }
        }
    }
    // flush buffer
    if cur != 0 {
        if output_buf[cur - 1] == b' ' {
            cur -= 1;
            *need_separator = true;
        } else {
            *need_separator = false;
        }
        output.write_all(&output_buf[..cur])?;
    }
    Ok(())
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
    let mut writer = BufWriter::new(Vec::new());
    let mut output_buf = [0u8; 1 << 15];
    encode_buffer_ascii(&mut writer, input, &mut false, &mut output_buf).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

/// Encode ASCII characters from a [Read][std::io::Read] object into a [Write][std::io::Write] object.
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
pub fn encode_stream_ascii(input: &mut impl Read, output: &mut impl Write) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut need_separator = false;
    let mut output_buf = [0u8; 1 << 15];
    loop {
        let bytes_read = input.read(&mut input_buf).unwrap();
        if bytes_read == 0 {
            break;
        }
        encode_buffer_ascii(
            output,
            &input_buf[..bytes_read],
            &mut need_separator,
            &mut output_buf,
        )
        .unwrap();
    }
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
    use rand::{distributions::Standard, Rng};
    let data: Vec<u8> = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1024)
        .collect();
    encode_string_ascii(&data);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_ascii_encode_random_large() {
    use rand::{distributions::Standard, Rng};
    let data: Vec<u8> = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1048576)
        .collect();
    encode_string_ascii(&data);
}
