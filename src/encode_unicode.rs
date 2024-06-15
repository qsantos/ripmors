use std::io::{BufWriter, Read, Write};

use crate::encode_ascii_mapping::ASCII_TO_QWORD;
use crate::encode_unicode_mapping::from_unicode;

fn encode_buffer(
    output: &mut impl Write,
    input: &str,
    need_separator: &mut bool,
    output_buf: &mut [u8; 1 << 15],
) -> Result<(), std::io::Error> {
    let mut cur = 0;
    if *need_separator {
        output_buf[cur] = b' ';
        cur += 1;
    }
    for c in input.chars() {
        if c.is_ascii() {
            let (bytes, len) = ASCII_TO_QWORD[c as usize];
            if len == 0 {
            } else if len <= 8 {
                if (c == '\t' || c == '\n' || c == '\r') && cur > 0 && output_buf[cur - 1] == b' ' {
                    cur -= 1;
                }
                unsafe {
                    let dst = output_buf.as_mut_ptr().add(cur) as *mut u64;
                    dst.write_unaligned(bytes);
                }
            } else {
                // handle only ASCII character encoded as more than 7 elements + space
                assert_eq!(c, '%');
                output_buf[cur..cur + 18].copy_from_slice(b"----- -..-. ----- ");
            }
            cur += len;
        } else {
            let (bytes, len) = from_unicode(c);
            if len == 0 {
            } else if len <= 8 {
                let buf8 = unsafe { output_buf.get_unchecked_mut(cur..cur + 8) };
                let bytes8 = unsafe { &*(bytes.as_ptr() as *const [u8; 8]) };
                buf8.copy_from_slice(bytes8);
            } else {
                output_buf[cur..cur + len].copy_from_slice(bytes);
            }
            cur += len;
        }
        // flush buffer
        if cur >= output_buf.len() - 25 {
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

/// Encode characters from a [string slice][&str] into a [String].
///
/// The following ASCII characters are used to represent Morse code:
///
/// - Full stop (.) represents the Morse dot;
/// - Hyphen (-) represents the Morse dash;
/// - Space ( ) represents the letter space;
/// - Slash (/) represents the word space;
/// - Tab (\t), line feed (\n) and carriage return (\r) are kept as-is.
///
/// Characters that cannot be converted to Morse are ignored.
///
/// For example, the program below encodes a string slice.
/// ```
/// let morse = ripmors::encode_string("télégraphie");
/// assert_eq!(morse, "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. .");
/// ```
pub fn encode_string(input: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    let mut output_buf = [0u8; 1 << 15];
    encode_buffer(&mut writer, input, &mut false, &mut output_buf).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

/// Encode Unicode characters from a [Read][std::io::Read] object into a [Write][std::io::Write] object.
///
/// Bytes from `input` are interpreted as Unicode characters encoded in UTF-8. The following ASCII
/// characters are used to represent Morse code:
///
/// - Full stop (.) represents the Morse dot;
/// - Hyphen (-) represents the Morse dash;
/// - Space ( ) represents the letter space;
/// - Slash (/) represents the word space;
/// - Tab (\t), line feed (\n) and carriage return (\r) are kept as-is.
///
/// Unicode characters that cannot be converted to Morse are ignored.
///
/// **Note:** This will read data from `input` until exhaustion.
///
/// For example, the program below encodes an UTF-8 input.
/// ```
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     let mut stdin = std::io::stdin();
///     let mut stdout = std::io::stdout();
///     ripmors::encode_stream(&mut stdin, &mut stdout);
/// }
/// ```
pub fn encode_stream(input: &mut impl Read, output: &mut impl Write) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut need_separator = false;
    let mut output_buf = [0u8; 1 << 15];
    loop {
        let bytes_read = input.read(&mut input_buf[bytes_available..]).unwrap();
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;
        let (decoded, bytes_decoded) =
            match simdutf8::compat::from_utf8(&input_buf[..bytes_available]) {
                Ok(decoded) => (decoded, bytes_available),
                Err(e) => {
                    let bytes_decoded = e.valid_up_to();
                    let decoded =
                        unsafe { core::str::from_utf8_unchecked(&input_buf[..bytes_decoded]) };
                    (decoded, bytes_decoded)
                }
            };
        encode_buffer(output, decoded, &mut need_separator, &mut output_buf).unwrap();
        input_buf.copy_within(bytes_decoded..bytes_available, 0);
        bytes_available -= bytes_decoded;
    }
}

#[test]
fn test_unicode_encode() {
    assert_eq!(
        encode_string("télégraphie"),
        "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. ."
    );
    assert_eq!(encode_string("でんしん"), ".-.-- .. .-.-. --.-. .-.-.");
    assert_eq!(encode_string("تلغراف"), "- .-.. --. .-. .- ..-.");
    assert_eq!(
        encode_string("телеграфия"),
        "- . .-.. . --. .-. .- ..-. .. .-.-"
    );
    assert_eq!(
        encode_string("τηλεγραφία"),
        "- .... .-.. . --. .-. .- ..-. .. .-"
    );
    assert_eq!(
        encode_string("one line\nand  another\tline"),
        "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
    );
}

// short enough to run with Miri
#[test]
fn test_unicode_encode_random_short() {
    use rand::{distributions::Standard, Rng};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1024)
        .map(|c| c as char)
        .collect();
    encode_string(&data);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_unicode_encode_random_large() {
    use rand::{distributions::Standard, Rng};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1048576)
        .map(|c| c as char)
        .collect();
    encode_string(&data);
}
