use std::io::{Read, Write};

use crate::encode_ascii_mapping::ASCII_TO_QWORD;
use crate::encode_unicode_mapping::from_unicode;

fn encode_buffer(input: &str, output_buf: &mut Vec<u8>) {
    // SAFETY: `output_buf[cur]`
    // Accessing the element `cur` of `output_buf` is safe because
    // - `cur <= 18 * input_buf.len() + 1` because we increment `cur` by at most 18 for each byte read
    // - `18 * input_buf.len() + 1 <= output_buf` as check by the `assert!` below
    let mut cur = output_buf.len();
    output_buf.reserve(input.len() * 18 + cur);
    for c in input.chars() {
        if c.is_ascii() {
            let (bytes, len) = ASCII_TO_QWORD[c as usize];
            if len == 0 {
            } else if len <= 8 {
                if (c == '\t' || c == '\n' || c == '\r')
                    && cur > 0
                    // SAFETY: see `output_buf[cur]` above
                    && unsafe { *output_buf.as_ptr().add(cur - 1) } == b' '
                {
                    cur -= 1;
                }
                // SAFETY: we flush the buffer after each byte when we are below 18 free bytes
                // next chunk; thus, there is at least 8 available bytes for writing after
                // `output_buf + cur`.
                unsafe {
                    let dst = output_buf.as_mut_ptr().add(cur) as *mut u64;
                    dst.write_unaligned(bytes);
                }
            } else {
                // handle only ASCII character encoded as more than 7 elements + space
                assert_eq!(c, '%');
                // SAFETY: `output_buf[cur]` above
                unsafe { std::slice::from_raw_parts_mut(output_buf.as_mut_ptr().add(cur), 18) }
                    .copy_from_slice(b"----- -..-. ----- ");
            }
            cur += len;
        } else {
            let (bytes, len) = from_unicode(c);
            if len == 0 {
            } else if len <= 8 {
                // SAFETY: we flush the buffer after each byte when we are below 18 free bytes
                // next chunk; thus, there is at least 8 available bytes for writing after
                // `output_buf + cur`.
                let buf8 =
                    unsafe { std::slice::from_raw_parts_mut(output_buf.as_mut_ptr().add(cur), 8) };
                // SAFETY: the slice is exactly 8 bytes long per construction
                let bytes8 = unsafe { &*(bytes.as_ptr() as *const [u8; 8]) };
                buf8.copy_from_slice(bytes8);
            } else {
                // SAFETY: `output_buf[cur]` above
                unsafe { std::slice::from_raw_parts_mut(output_buf.as_mut_ptr().add(cur), len) }
                    .copy_from_slice(bytes);
            }
            cur += len;
        }
    }
    // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase cur
    // after writing to `output_buf`
    unsafe { output_buf.set_len(cur) };
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
    let mut output_buf = Vec::new();
    encode_buffer(input, &mut output_buf);
    output_buf.pop_if(|c| *c == b' ');
    // SAFETY: encode_buffer_ascii only outputs ASCII, so it is valid UTF-8
    unsafe { String::from_utf8_unchecked(output_buf) }
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
/// ```no_run
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     let mut stdin = std::io::stdin();
///     let mut stdout = std::io::stdout();
///     ripmors::encode_stream(&mut stdin, &mut stdout);
/// }
/// ```
pub fn encode_stream(input: &mut impl Read, output: &mut impl Write) -> Result<(), std::io::Error> {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut output_buf = Vec::new();
    loop {
        let bytes_read = input.read(&mut input_buf[bytes_available..])?;
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;
        let (decoded, bytes_decoded) =
            match simdutf8::compat::from_utf8(&input_buf[..bytes_available]) {
                Ok(decoded) => (decoded, bytes_available),
                Err(e) => {
                    let bytes_decoded = e.valid_up_to();
                    // SAFETY: we already checked that the string was valid UTF-8 up to
                    // `bytes_decoded`
                    let decoded =
                        unsafe { core::str::from_utf8_unchecked(&input_buf[..bytes_decoded]) };
                    (decoded, bytes_decoded)
                }
            };
        encode_buffer(decoded, &mut output_buf);
        match output_buf.last() {
            Some(&b' ') => {
                output_buf.pop();
                output.write_all(&output_buf)?;
                output_buf.clear();
                output_buf.push(b' ');
            }
            None => {
                output.write_all(&output_buf)?;
                output_buf.clear();
            }
            _ => (),
        }
        input_buf.copy_within(bytes_decoded..bytes_available, 0);
        bytes_available -= bytes_decoded;
    }
    Ok(())
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
    use rand::{Rng, distributions::Standard};
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
    use rand::{Rng, distributions::Standard};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1048576)
        .map(|c| c as char)
        .collect();
    encode_string(&data);
}
