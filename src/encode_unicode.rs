use std::io::{Read, Write};

use crate::encode_ascii::{ENCODE_CHUNK, MAX_OUTPUT_PER_INPUT_BYTE, encode_ascii_byte};
use crate::encode_unicode_mapping::from_unicode;

/// Encode a non-ASCII character at `out + cur`, and return the new value of `cur`.
///
/// # Safety
///
/// `out` must be valid for writes of `cur + MAX_OUTPUT_PER_INPUT_BYTE` bytes.
#[inline(never)]
unsafe fn encode_non_ascii_char(c: char, out: *mut u8, cur: usize) -> usize {
    let (bytes, len) = from_unicode(c);
    // SAFETY: `len <= bytes.len()` and the caller guarantees there is room for 18 bytes at
    // `out + cur`; `bytes` is padded to at least 8 bytes, so copying 8 bytes is valid when
    // `len <= 8`
    unsafe {
        if len <= 8 {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.add(cur), 8);
        } else {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.add(cur), len);
        }
    }
    cur + len
}

fn encode_buffer(input: &str, output_buf: &mut Vec<u8>) {
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // a multi-byte character starting before `end` may overrun it by at most 3 bytes; we
        // reserve room for them as well
        let end = bytes.len().min(i + ENCODE_CHUNK);
        output_buf.reserve((end - i + 3) * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        while i < end {
            let c = bytes[i];
            if c < 0x80 {
                // SAFETY: we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the
                // chunk, and the first `cur` bytes are initialized since we only increase `cur`
                // after writing
                cur = unsafe { encode_ascii_byte(c, out, cur) };
                i += 1;
            } else {
                // SAFETY: `i` is always on a character boundary since we advance by whole
                // characters, and `i < input.len()` so there is at least one character left
                let c = unsafe { input.get_unchecked(i..).chars().next().unwrap_unchecked() };
                // SAFETY: see above; non-ASCII characters are at least 2 bytes long and encoded
                // as at most 8 bytes
                cur = unsafe { encode_non_ascii_char(c, out, cur) };
                i += c.len_utf8();
            }
        }
        // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase
        // `cur` after writing to `output_buf`
        unsafe { output_buf.set_len(cur) };
    }
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
