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

/// Index in [`DecodeTable`] of the entry that outputs nothing, used for word spaces (`/`), which
/// discard the pending Morse sequence.
const DISCARD: usize = 256;

/// The output of `char_decode` for each binary value, encoded in UTF-8, padded to four bytes, and
/// with its length (zero for `'\0'`), plus an empty entry at [`DISCARD`]. Tabulating it avoids an
/// indirect call and a conversion to UTF-8 for each decoded character.
struct DecodeTable {
    utf8: [[u8; 4]; 257],
    len: [u8; 257],
}

impl DecodeTable {
    fn new(char_decode: fn(u8) -> char) -> Self {
        let mut table = DecodeTable {
            utf8: [[0; 4]; 257],
            len: [0; 257],
        };
        for binary in 0..256 {
            let decoded = char_decode(binary as u8);
            if decoded != '\0' {
                table.len[binary] = decoded.encode_utf8(&mut table.utf8[binary]).len() as u8;
            }
        }
        table
    }
}

/// Maximum number of bytes written to the output for each byte of input: a decoded character
/// (written as four bytes) followed by the separator.
const MAX_OUTPUT_PER_INPUT_BYTE: usize = 5;

/// Bytes ending a Morse sequence: whitespace and control characters (`<= b' '`), and `/`.
#[inline(always)]
fn is_separator(c: u8) -> bool {
    c <= b' ' || c == b'/'
}

/// Bitmask of the separators in a 64-byte block (bit `j` is set when `block[j]` is a separator).
#[inline(always)]
fn separator_mask(block: &[u8; 64]) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::*;
        let mut mask = 0;
        for k in 0..4 {
            // SAFETY: SSE2 is part of the x86_64 baseline, and `block` has 64 bytes, so the
            // 16-byte load at `16 * k` is in bounds
            let bits = unsafe {
                let v = _mm_loadu_si128(block.as_ptr().add(16 * k).cast());
                let le_space = _mm_cmpeq_epi8(_mm_min_epu8(v, _mm_set1_epi8(b' ' as i8)), v);
                let slash = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'/' as i8));
                _mm_movemask_epi8(_mm_or_si128(le_space, slash)) as u16
            };
            mask |= (bits as u64) << (16 * k);
        }
        mask
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let mut mask = 0;
        for (j, &c) in block.iter().enumerate() {
            mask |= (is_separator(c) as u64) << j;
        }
        mask
    }
}

/// Decode the Morse sequence starting at `chunk_start` and ending with the separator at
/// `input[i]`, and write it to `out + cur`, followed by the separator; return the new `cur`.
///
/// # Safety
///
/// - `input[i]` must be a separator;
/// - `chunk_start <= i` and `i + 8 <= input.len()`;
/// - `out` must be valid for writes of `cur + MAX_OUTPUT_PER_INPUT_BYTE` bytes.
#[inline(always)]
unsafe fn decode_separator(
    input: &[u8],
    chunk_start: usize,
    i: usize,
    table: &DecodeTable,
    out: *mut u8,
    mut cur: usize,
) -> usize {
    // SAFETY: `chunk_start <= i` and `i + 8 <= input.len()` are guaranteed by the caller
    let c = unsafe { *input.get_unchecked(i) };
    // SAFETY: see above; this points to eight initialized bytes
    let eight_bytes: &[u8; 8] = unsafe { &*input.as_ptr().add(chunk_start).cast() };
    let binary = morse_to_binary_fast(eight_bytes, i - chunk_start) as usize;
    // NOTE: written so as to be compiled into conditional moves rather than branches, since
    // separators are not predictable
    let is_slash = c == b'/';
    let index = if is_slash { DISCARD } else { binary };
    let separator = if is_slash { b' ' } else { c };
    // SAFETY: `index <= 256`, and the caller guarantees there is room for five bytes at `out + cur`
    unsafe {
        out.add(cur)
            .cast::<[u8; 4]>()
            .write_unaligned(*table.utf8.get_unchecked(index));
        cur += *table.len.get_unchecked(index) as usize;
        out.add(cur).write(separator);
    }
    cur + (c != b' ') as usize
}

fn decode_buffer(input: &[u8], table: &DecodeTable, output_buf: &mut Vec<u8>) -> usize {
    let mut chunk_start = 0;
    let mut i = 0;
    // process 64-byte blocks, as long as there are 7 bytes after the block for
    // `morse_to_binary_fast`
    while i + 64 + 7 <= input.len() {
        // reserve output space for up to 64 blocks at a time
        let blocks = ((input.len() - 7 - i) / 64).min(64);
        output_buf.reserve(blocks * 64 * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        for _ in 0..blocks {
            let block: &[u8; 64] = input[i..i + 64].try_into().unwrap();
            let mut mask = separator_mask(block);
            while mask != 0 {
                let j = i + mask.trailing_zeros() as usize;
                mask &= mask - 1;
                // SAFETY: `input[j]` is a separator per `mask`, `chunk_start <= j` since
                // `chunk_start` is either 0 or right after a previous separator, and
                // `j + 8 <= i + 64 + 7 <= input.len()`; we reserved `MAX_OUTPUT_PER_INPUT_BYTE`
                // bytes per input byte of the blocks
                cur = unsafe { decode_separator(input, chunk_start, j, table, out, cur) };
                chunk_start = j + 1;
            }
            i += 64;
        }
        // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase
        // `cur` after writing to `output_buf`
        unsafe { output_buf.set_len(cur) };
    }
    // remaining bytes
    for i in i..input.len() {
        let c = input[i];
        if is_separator(c) {
            if c == b'/' {
                output_buf.push(b' ');
            } else {
                let binary = morse_to_binary(&input[chunk_start..], i - chunk_start);
                let len = table.len[binary as usize] as usize;
                output_buf.extend_from_slice(&table.utf8[binary as usize][..len]);
                if c != b' ' {
                    output_buf.push(c);
                }
            }
            chunk_start = i + 1;
        }
    }
    chunk_start
}

fn decode_buffer_end(input: &[u8], table: &DecodeTable, output_buf: &mut Vec<u8>) {
    let chunk_start = decode_buffer(input, table, output_buf);
    let binary = morse_to_binary(&input[chunk_start..], input.len() - chunk_start);
    let len = table.len[binary as usize] as usize;
    output_buf.extend_from_slice(&table.utf8[binary as usize][..len]);
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
    let table = DecodeTable::new(char_decode);
    let mut output_buf = Vec::new();
    decode_buffer_end(input, &table, &mut output_buf);
    // SAFETY: the output only contains UTF-8 encodings of characters from `table`, and bytes up to
    // `b' '`, which are ASCII
    unsafe { String::from_utf8_unchecked(output_buf) }
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
    let table = DecodeTable::new(char_decode);
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut output_buf = Vec::new();
    loop {
        let bytes_read = input.read(&mut input_buf[bytes_available..])?;
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;

        let bytes_used = decode_buffer(&input_buf[..bytes_available], &table, &mut output_buf);

        // flush buffer
        if !output_buf.is_empty() {
            output.write_all(&output_buf)?;
            output_buf.clear();
        }

        input_buf.copy_within(bytes_used..bytes_available, 0);
        bytes_available -= bytes_used;
    }

    if bytes_available != 0 {
        decode_buffer_end(&input_buf[..bytes_available], &table, &mut output_buf);
        if !output_buf.is_empty() {
            output.write_all(&output_buf)?;
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
