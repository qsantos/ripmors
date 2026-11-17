//! Decoding of Morse code.
//!
//! The input is a sequence of bytes:
//!
//! - `.` and `-` are Morse elements (dot and dash);
//! - separators end the current Morse sequence (which is decoded) and are output as follows:
//!   - ` ` (letter space) produces nothing,
//!   - `/` (word space) produces a space,
//!   - `\t`, `\n` and `\r` are copied as-is.
//!
//! A Morse sequence is dropped (it produces nothing) when it contains a byte that is neither an
//! element nor a separator, when it has more than 7 elements, or when it does not correspond to a
//! character of the selected alphabet.

use std::borrow::Cow;
use std::io::{Read, Write};

use crate::decode_mapping::{
    TO_ARABIC, TO_GREEK, TO_HEBREW, TO_JAPANESE, TO_KOREAN, TO_RUSSIAN, TO_STANDARD,
};

#[cfg(target_arch = "x86_64")]
mod avx512;

/// Maximum number of elements in a Morse sequence.
const MAX_ELEMENTS: usize = 7;

/// Size of the input buffer used by [`Decoder::decode_stream`].
const STREAM_BUFFER_SIZE: usize = 1 << 17;

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

/// Offset added to an index in [`DecodeTable`] for sequences that must be dropped; the entries
/// from this offset are empty.
const DROP: usize = 256;

/// Index in [`DecodeTable`] of the Morse sequence made of the first `len` bytes of `bytes`, or
/// [`DROP`] or more when the sequence must be dropped. This has no branches.
#[inline(always)]
fn sequence_index_fast(bytes: &[u8; 8], len: usize) -> usize {
    let too_long = len > MAX_ELEMENTS;
    let len = len.min(MAX_ELEMENTS);
    // SAFETY: `bytes` is a reference to 8 initialized bytes
    let a = unsafe { (bytes.as_ptr() as *const u64).read_unaligned() };
    // each byte becomes 0x00 for `-` and 0x03 for `.`: any other value is an invalid byte
    let t = a ^ 0x2d2d_2d2d_2d2d_2d2d;
    let invalid = (t & 0xfcfc_fcfc_fcfc_fcfc) | ((t ^ (t >> 1)) & 0x0101_0101_0101_0101);
    let invalid = invalid & ((1 << (8 * len)) - 1);
    let drop = too_long | (invalid != 0);
    morse_to_binary_fast(bytes, len) as usize | ((drop as usize) << 8)
}

/// Same as [`sequence_index_fast`], for a sequence of any length.
fn sequence_index(sequence: &[u8]) -> usize {
    if sequence.len() > MAX_ELEMENTS || !sequence.iter().all(|&c| c == b'.' || c == b'-') {
        return DROP;
    }
    let mut binary = 1;
    for c in sequence.iter().rev() {
        binary = binary * 2 + (c & 1) as usize;
    }
    binary
}

#[test]
fn test_sequence_index() {
    fn fast(s: &[u8]) -> usize {
        let mut bytes = [b'x'; 8];
        bytes[..s.len().min(8)].copy_from_slice(&s[..s.len().min(8)]);
        sequence_index_fast(&bytes, s.len())
    }
    for s in [
        &b""[..],
        b".",
        b"-",
        b"..-.",
        b"-------",
        b".......",
        b"--.--.-",
    ] {
        assert!(sequence_index(s) < DROP, "{s:?}");
        assert_eq!(fast(s), sequence_index(s), "{s:?}");
    }
    for s in [
        &b"........"[..],
        b"x",
        b".x",
        b"-.-.-.x",
        b"..,",
        b"\x01",
        b"--------------",
    ] {
        assert!(sequence_index(s) >= DROP, "{s:?}");
        assert!(fast(s) >= DROP, "{s:?}");
    }
    assert_eq!(sequence_index(b"..-."), 0b10100);
}

/// Bytes ending a Morse sequence.
#[inline(always)]
pub(crate) fn is_separator(c: u8) -> bool {
    matches!(c, b' ' | b'/' | b'\t' | b'\n' | b'\r')
}

/// The output of each separator: nothing for a letter space, a space for a word space, the
/// separator itself otherwise; returns the byte to write and whether to keep it. This has no
/// branches.
#[inline(always)]
fn separator_output(c: u8) -> (u8, bool) {
    let is_slash = c == b'/';
    (c ^ (is_slash as u8 * (b'/' ^ b' ')), c != b' ')
}

/// An alphabet for decoding Morse code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Alphabet {
    /// [International Morse code and Latin extensions](https://en.wikipedia.org/wiki/Morse_code#Letters,_numbers,_punctuation,_prosigns_for_Morse_code_and_non-Latin_variants)
    #[default]
    Standard,
    /// Greek
    Greek,
    /// [Russian](https://en.wikipedia.org/wiki/Russian_Morse_code) (Cyrillic)
    Russian,
    /// [Japanese](https://en.wikipedia.org/wiki/Wabun_code) (Katakana)
    Japanese,
    /// [Korean](https://en.wikipedia.org/wiki/SKATS) (Hangul)
    Korean,
    /// Hebrew
    Hebrew,
    /// Arabic
    Arabic,
}

/// The decoded character for each binary value (see [`morse_to_binary_fast`]), encoded in UTF-8,
/// padded to four bytes, and with its length (zero when there is none), followed by 256 empty
/// entries for [`DROP`].
#[derive(Clone)]
struct DecodeTable {
    utf8: [[u8; 4]; 512],
    len: [u8; 512],
    /// The first three bytes of the UTF-8 encodings (zero when there is none) by window index,
    /// for the AVX-512 kernel: the elements of the sequence in reverse order (the last one is the
    /// least significant bit), with 1 for `.` and 0 for `-`, plus a leading one; the first byte
    /// for index 0, which is not a sequence, is 0xff.
    window: [[u8; 128]; 3],
}

impl std::fmt::Debug for DecodeTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodeTable").finish_non_exhaustive()
    }
}

impl DecodeTable {
    const fn from_chars(chars: &[char; 256]) -> Self {
        let mut table = DecodeTable {
            utf8: [[0; 4]; 512],
            len: [0; 512],
            window: [[0; 128]; 3],
        };
        // binary values 0 and 1 do not correspond to sequences with at least one element
        let mut binary = 2;
        while binary < 256 {
            let c = chars[binary];
            if c != '\0' {
                table.len[binary] = c.encode_utf8(&mut table.utf8[binary]).len() as u8;
            }
            binary += 1;
        }
        table.window[0][0] = 0xff;
        let mut index = 1;
        while index < 128 {
            let len = 7 - (index as u8).leading_zeros() as usize;
            let mut binary = 1 << len;
            let mut t = 0;
            while t < len {
                // element `t` (from 0) is a dash when bit `len - 1 - t` is 0
                binary |= (1 - ((index >> (len - 1 - t)) & 1)) << t;
                t += 1;
            }
            let mut n = 0;
            while n < 3 {
                table.window[n][index] = table.utf8[binary][n];
                n += 1;
            }
            index += 1;
        }
        table
    }

    fn from_fn(char_decode: fn(u8) -> char) -> Self {
        let mut chars = ['\0'; 256];
        for (binary, c) in chars.iter_mut().enumerate() {
            *c = char_decode(binary as u8);
        }
        Self::from_chars(&chars)
    }

    /// Append the character of entry `index` to `output_buf`.
    #[inline(always)]
    fn push(&self, index: usize, output_buf: &mut Vec<u8>) {
        output_buf.extend_from_slice(&self.utf8[index][..self.len[index] as usize]);
    }
}

static STANDARD_TABLE: DecodeTable = DecodeTable::from_chars(&TO_STANDARD);
static GREEK_TABLE: DecodeTable = DecodeTable::from_chars(&TO_GREEK);
static RUSSIAN_TABLE: DecodeTable = DecodeTable::from_chars(&TO_RUSSIAN);
static JAPANESE_TABLE: DecodeTable = DecodeTable::from_chars(&TO_JAPANESE);
static KOREAN_TABLE: DecodeTable = DecodeTable::from_chars(&TO_KOREAN);
static HEBREW_TABLE: DecodeTable = DecodeTable::from_chars(&TO_HEBREW);
static ARABIC_TABLE: DecodeTable = DecodeTable::from_chars(&TO_ARABIC);

/// Maximum number of bytes written to the output for each byte of input: a decoded character
/// (written as four bytes) followed by the separator.
const MAX_OUTPUT_PER_INPUT_BYTE: usize = 5;

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
                let eq = |c: u8| _mm_cmpeq_epi8(v, _mm_set1_epi8(c as i8));
                let separators = _mm_or_si128(
                    _mm_or_si128(eq(b' '), eq(b'/')),
                    _mm_or_si128(_mm_or_si128(eq(b'\t'), eq(b'\n')), eq(b'\r')),
                );
                _mm_movemask_epi8(separators) as u16
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
    let index = sequence_index_fast(eight_bytes, i - chunk_start);
    let (separator, keep) = separator_output(c);
    // SAFETY: `index < 512` since `sequence_index_fast` returns a byte value plus at most `DROP`,
    // and the caller guarantees there is room for five bytes at `out + cur`
    unsafe {
        out.add(cur)
            .cast::<[u8; 4]>()
            .write_unaligned(*table.utf8.get_unchecked(index));
        cur += *table.len.get_unchecked(index) as usize;
        out.add(cur).write(separator);
    }
    cur + keep as usize
}

/// Decode the separators at `base + j` for each bit `j` set in `mask`, writing to `out + cur`;
/// return the new values of `cur` and `chunk_start`.
///
/// # Safety
///
/// - each `base + j` must be a separator, and `chunk_start <= base + j`;
/// - `base + 64 + 7 <= input.len()`;
/// - `out` must be valid for writes of `cur + 64 * MAX_OUTPUT_PER_INPUT_BYTE` bytes.
#[inline(always)]
unsafe fn decode_separators(
    input: &[u8],
    base: usize,
    mut mask: u64,
    mut chunk_start: usize,
    table: &DecodeTable,
    out: *mut u8,
    mut cur: usize,
) -> (usize, usize) {
    while mask != 0 {
        let j = base + mask.trailing_zeros() as usize;
        mask &= mask - 1;
        // SAFETY: forwarded from the caller; `chunk_start <= j` since `chunk_start` is either
        // unchanged or right after a previous separator, and `j + 8 <= base + 64 + 7`
        cur = unsafe { decode_separator(input, chunk_start, j, table, out, cur) };
        chunk_start = j + 1;
    }
    (cur, chunk_start)
}

/// Decode the separator `input[i]` (if it is one), with bounds checks.
fn decode_byte_checked(
    input: &[u8],
    i: usize,
    chunk_start: &mut usize,
    table: &DecodeTable,
    output_buf: &mut Vec<u8>,
) {
    let c = input[i];
    if is_separator(c) {
        table.push(sequence_index(&input[*chunk_start..i]), output_buf);
        let (separator, keep) = separator_output(c);
        if keep {
            output_buf.push(separator);
        }
        *chunk_start = i + 1;
    }
}

/// Decode `input`, which must start with a new Morse sequence, and append the result to
/// `output_buf`; return the start of the last Morse sequence, which is not decoded since it may
/// continue after `input`.
fn decode_buffer(input: &[u8], table: &DecodeTable, output_buf: &mut Vec<u8>) -> usize {
    decode_buffer_with(input, table, output_buf, true)
}

/// See [`decode_buffer`]; the AVX-512 kernel is only used when `allow_avx512` is true (for
/// tests).
#[inline(always)]
#[cfg_attr(not(target_arch = "x86_64"), allow(unused_variables))]
fn decode_buffer_with(
    input: &[u8],
    table: &DecodeTable,
    output_buf: &mut Vec<u8>,
    allow_avx512: bool,
) -> usize {
    let mut chunk_start = 0;
    let mut i = 0;
    #[cfg(target_arch = "x86_64")]
    if allow_avx512 && input.len() >= avx512::MIN_INPUT_LEN && avx512::is_available() {
        // the AVX-512 path looks back 8 bytes
        while i < 8 {
            decode_byte_checked(input, i, &mut chunk_start, table, output_buf);
            i += 1;
        }
        // SAFETY: we just checked that the CPU supports the required features, and the other
        // requirements hold since we processed exactly the first 8 bytes
        (i, chunk_start) =
            unsafe { avx512::decode_blocks(input, i, chunk_start, table, output_buf) };
    }
    // process 64-byte blocks, as long as there are 7 bytes after the block for
    // `sequence_index_fast`
    while i + 64 + 7 <= input.len() {
        // reserve output space for up to 64 blocks at a time
        let blocks = ((input.len() - 7 - i) / 64).min(64);
        output_buf.reserve(blocks * 64 * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        for _ in 0..blocks {
            let block: &[u8; 64] = input[i..i + 64].try_into().unwrap();
            let mask = separator_mask(block);
            // SAFETY: the bits of `mask` are separators, `chunk_start <= i` and we checked
            // `i + 64 + 7 <= input.len()`; we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes per
            // input byte of the blocks
            (cur, chunk_start) =
                unsafe { decode_separators(input, i, mask, chunk_start, table, out, cur) };
            i += 64;
        }
        // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase
        // `cur` after writing to `output_buf`
        unsafe { output_buf.set_len(cur) };
    }
    // remaining bytes
    for i in i..input.len() {
        decode_byte_checked(input, i, &mut chunk_start, table, output_buf);
    }
    chunk_start
}

/// Decodes Morse code in a given [`Alphabet`].
///
/// See the [crate documentation](crate) for the format of Morse code.
///
/// ```
/// use ripmors::{Alphabet, Decoder};
///
/// let decoder = Decoder::new(Alphabet::Standard);
/// assert_eq!(decoder.decode(b"-- --- .-. ... . / -.-. --- -.. ."), "MORSE CODE");
/// ```
#[derive(Clone)]
pub struct Decoder {
    table: Cow<'static, DecodeTable>,
}

impl std::fmt::Debug for Decoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder").finish_non_exhaustive()
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new(Alphabet::default())
    }
}

impl Decoder {
    /// Create a decoder for the given alphabet. This is free: the decoding tables are built at
    /// compile time.
    pub const fn new(alphabet: Alphabet) -> Self {
        let table = match alphabet {
            Alphabet::Standard => &STANDARD_TABLE,
            Alphabet::Greek => &GREEK_TABLE,
            Alphabet::Russian => &RUSSIAN_TABLE,
            Alphabet::Japanese => &JAPANESE_TABLE,
            Alphabet::Korean => &KOREAN_TABLE,
            Alphabet::Hebrew => &HEBREW_TABLE,
            Alphabet::Arabic => &ARABIC_TABLE,
        };
        Decoder {
            table: Cow::Borrowed(table),
        }
    }

    /// Create a decoder for a custom alphabet.
    ///
    /// `char_decode` maps a Morse sequence to a character, or to `'\0'` when there is none. A
    /// sequence of `n` elements (at most 7) is represented by `n` bits (0 for a dot, 1 for a
    /// dash, the first element being the least significant bit), plus a leading 1 bit at
    /// position `n`. For instance, `.-` (A) is represented as `0b110`. It is called once for each
    /// possible value when creating the decoder.
    ///
    /// ```
    /// use ripmors::Decoder;
    ///
    /// let decoder = Decoder::from_fn(|binary| if binary == 0b110 { 'α' } else { '\0' });
    /// assert_eq!(decoder.decode(b".- .- -"), "αα");
    /// ```
    pub fn from_fn(char_decode: fn(u8) -> char) -> Self {
        Decoder {
            table: Cow::Owned(DecodeTable::from_fn(char_decode)),
        }
    }

    /// Decode Morse code from a byte slice into a new [`String`].
    pub fn decode(&self, input: &[u8]) -> String {
        let mut output = String::new();
        self.decode_into(input, &mut output);
        output
    }

    /// Decode Morse code from a byte slice, appending the result to `output`.
    ///
    /// Reusing the same `output` across calls avoids allocations.
    ///
    /// ```
    /// use ripmors::Decoder;
    ///
    /// let decoder = Decoder::default();
    /// let mut output = String::new();
    /// for morse in [&b"... --- ..."[..], b".--. .- .-. .. ..."] {
    ///     output.clear();
    ///     decoder.decode_into(morse, &mut output);
    ///     println!("{output}");
    /// }
    /// ```
    pub fn decode_into(&self, input: &[u8], output: &mut String) {
        let mut stream = self.stream();
        stream.feed(input, output);
        stream.finish(output);
    }

    /// Decode Morse code from a byte slice into a new [`String`], failing on the first invalid
    /// byte, too long sequence, or sequence that does not correspond to a character of the
    /// alphabet.
    ///
    /// ```
    /// use ripmors::{DecodeErrorKind, Decoder};
    ///
    /// let decoder = Decoder::default();
    /// assert_eq!(decoder.decode_strict(b"... --- ...").unwrap(), "SOS");
    /// let error = decoder.decode_strict(b"... -x- ...").unwrap_err();
    /// assert_eq!(error.position(), 5);
    /// assert_eq!(error.kind(), DecodeErrorKind::InvalidByte(b'x'));
    /// ```
    pub fn decode_strict(&self, input: &[u8]) -> Result<String, DecodeError> {
        let mut start = 0;
        for i in 0..=input.len() {
            if i < input.len() && !is_separator(input[i]) {
                continue;
            }
            let sequence = &input[start..i];
            if let Some(j) = sequence.iter().position(|&c| c != b'.' && c != b'-') {
                return Err(DecodeError {
                    position: start + j,
                    kind: DecodeErrorKind::InvalidByte(sequence[j]),
                });
            }
            if sequence.len() > MAX_ELEMENTS {
                return Err(DecodeError {
                    position: start,
                    kind: DecodeErrorKind::TooManyElements,
                });
            }
            if !sequence.is_empty() && self.table.len[sequence_index(sequence)] == 0 {
                return Err(DecodeError {
                    position: start,
                    kind: DecodeErrorKind::UnknownSequence,
                });
            }
            start = i + 1;
        }
        Ok(self.decode(input))
    }

    /// Decode Morse code from a [`Read`] object into a [`Write`] object, until the end of the
    /// input.
    ///
    /// ```no_run
    /// let mut stdin = std::io::stdin().lock();
    /// let mut stdout = std::io::stdout().lock();
    /// ripmors::Decoder::default().decode_stream(&mut stdin, &mut stdout).unwrap();
    /// ```
    pub fn decode_stream(
        &self,
        input: &mut impl Read,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        let mut input_buf = vec![0u8; STREAM_BUFFER_SIZE];
        let mut output_buf = String::new();
        let mut stream = self.stream();
        loop {
            let bytes_read = match input.read(&mut input_buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            stream.feed(&input_buf[..bytes_read], &mut output_buf);
            output.write_all(output_buf.as_bytes())?;
            output_buf.clear();
        }
        stream.finish(&mut output_buf);
        output.write_all(output_buf.as_bytes())
    }

    /// Start decoding Morse code incrementally, see [`DecodeStream`].
    pub fn stream(&self) -> DecodeStream<'_> {
        DecodeStream {
            table: &self.table,
            pending: [0; 8],
            pending_len: 0,
        }
    }
}

/// Incremental decoding of Morse code, created by [`Decoder::stream`].
///
/// The input can be fed in arbitrary pieces; the output is the same as decoding the
/// concatenation of all the pieces at once.
///
/// ```
/// use ripmors::Decoder;
///
/// let decoder = Decoder::default();
/// let mut stream = decoder.stream();
/// let mut output = String::new();
/// stream.feed(b"... -", &mut output);
/// stream.feed(b"-- ..", &mut output);
/// stream.feed(b".", &mut output);
/// stream.finish(&mut output);
/// assert_eq!(output, "SOS");
/// ```
#[derive(Clone, Debug)]
pub struct DecodeStream<'a> {
    table: &'a DecodeTable,
    /// The beginning of the last Morse sequence, which may continue in the next input
    pending: [u8; 8],
    /// Length of the last Morse sequence, capped to `pending.len()`; since this is more than
    /// [`MAX_ELEMENTS`], longer sequences are still dropped
    pending_len: usize,
}

impl DecodeStream<'_> {
    fn extend_pending(&mut self, bytes: &[u8]) {
        let n = bytes.len().min(self.pending.len() - self.pending_len);
        self.pending[self.pending_len..self.pending_len + n].copy_from_slice(&bytes[..n]);
        // NOTE: keep counting past capacity, so that too long sequences are dropped
        self.pending_len = (self.pending_len + bytes.len()).min(self.pending.len());
    }

    fn pending_index(&self) -> usize {
        sequence_index(&self.pending[..self.pending_len])
    }

    /// Decode the next part of the input, appending the result to `output`.
    ///
    /// The last Morse sequence of `input` is only decoded when it is followed by a separator,
    /// either in `input` or in the next call to [`feed`][Self::feed], or on
    /// [`finish`][Self::finish].
    pub fn feed(&mut self, mut input: &[u8], output: &mut String) {
        // SAFETY: we only append valid UTF-8 to `output`: UTF-8 encodings of characters from
        // `self.table`, and ASCII separators
        let output_buf = unsafe { output.as_mut_vec() };
        if self.pending_len != 0 {
            // complete the pending sequence
            let Some(k) = input.iter().position(|&c| is_separator(c)) else {
                self.extend_pending(input);
                return;
            };
            self.extend_pending(&input[..k]);
            self.table.push(self.pending_index(), output_buf);
            let (separator, keep) = separator_output(input[k]);
            if keep {
                output_buf.push(separator);
            }
            self.pending_len = 0;
            input = &input[k + 1..];
        }
        let chunk_start = decode_buffer(input, self.table, output_buf);
        self.extend_pending(&input[chunk_start..]);
    }

    /// Decode the last Morse sequence (if any), appending the result to `output`.
    pub fn finish(self, output: &mut String) {
        // SAFETY: we only append the UTF-8 encoding of a character from `self.table`
        let output_buf = unsafe { output.as_mut_vec() };
        self.table.push(self.pending_index(), output_buf);
    }
}

/// The reason of a [`DecodeError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DecodeErrorKind {
    /// A byte that is neither a Morse element nor a separator
    InvalidByte(u8),
    /// A Morse sequence with more than 7 elements
    TooManyElements,
    /// A Morse sequence that does not correspond to any character in the alphabet
    UnknownSequence,
}

/// Error returned by [`Decoder::decode_strict`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DecodeError {
    position: usize,
    kind: DecodeErrorKind,
}

impl DecodeError {
    /// Position in the input of the invalid byte, or of the start of the invalid sequence.
    pub fn position(&self) -> usize {
        self.position
    }

    /// The reason of the error.
    pub fn kind(&self) -> DecodeErrorKind {
        self.kind
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            DecodeErrorKind::InvalidByte(c) => {
                write!(f, "invalid byte {c:#04x} at position {}", self.position)
            }
            DecodeErrorKind::TooManyElements => write!(
                f,
                "Morse sequence with more than {MAX_ELEMENTS} elements at position {}",
                self.position
            ),
            DecodeErrorKind::UnknownSequence => {
                write!(f, "unknown Morse sequence at position {}", self.position)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Run the decoding kernel with and without AVX-512; return the output and the start of the
/// undecoded tail of each.
#[cfg(fuzzing)]
pub(crate) fn kernels(input: &[u8], alphabet: Alphabet) -> [(Vec<u8>, usize); 2] {
    let decoder = Decoder::new(alphabet);
    [true, false].map(|allow_avx512| {
        let mut output = Vec::new();
        let chunk_start = decode_buffer_with(input, &decoder.table, &mut output, allow_avx512);
        (output, chunk_start)
    })
}

#[cfg(any(test, fuzzing))]
/// Straightforward implementation of the decoding rules.
pub(crate) fn reference(input: &[u8], alphabet: Alphabet) -> String {
    let table = Decoder::new(alphabet).table;
    let mut output = Vec::new();
    let mut start = 0;
    for i in 0..=input.len() {
        if i < input.len() && !is_separator(input[i]) {
            continue;
        }
        table.push(sequence_index(&input[start..i]), &mut output);
        match input.get(i) {
            Some(b'/') => output.push(b' '),
            Some(b' ') | None => (),
            Some(&c) => output.push(c),
        }
        start = i + 1;
    }
    String::from_utf8(output).unwrap()
}

#[cfg(test)]
mod differential_tests;

#[cfg(test)]
mod tests {
    use super::*;

    const ALPHABETS: [Alphabet; 7] = [
        Alphabet::Standard,
        Alphabet::Greek,
        Alphabet::Russian,
        Alphabet::Japanese,
        Alphabet::Korean,
        Alphabet::Hebrew,
        Alphabet::Arabic,
    ];

    /// Random Morse code with occasional invalid bytes and long sequences.
    fn random_morse(rng: &mut impl rand::Rng, len: usize, noise: f64) -> Vec<u8> {
        const BYTES: &[u8] = b"..........----------      //\n\t\r";
        (0..len)
            .map(|_| {
                if rng.gen_bool(noise) {
                    rng.r#gen()
                } else {
                    BYTES[rng.gen_range(0..BYTES.len())]
                }
            })
            .collect()
    }

    #[test]
    fn test_standard_decode() {
        let f = |s| Decoder::new(Alphabet::Standard).decode(s);
        assert_eq!(f(b".--. .- .-. .. ..."), "PARIS");
        assert_eq!(
            f(b".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--"),
            "HELLO, WORLD!",
        );
        // the word space decodes the pending sequence
        assert_eq!(f(b"-/-"), "T T");
        // invalid and too long sequences are dropped
        assert_eq!(f(b"-x- ........ . -.-.-.-.-"), "E");
        assert_eq!(f(b"\0.-\x01"), "");
        assert_eq!(f(b"... \n--- \r\t..."), "S\nO\r\tS");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_decode_matches_reference() {
        let mut rng = rand::thread_rng();
        for &alphabet in &ALPHABETS {
            let decoder = Decoder::new(alphabet);
            for &noise in &[0.0, 0.001, 0.05, 0.5] {
                for len in (0..300).chain([1000, 4096, 70_000]) {
                    let input = random_morse(&mut rng, len, noise);
                    assert_eq!(
                        decoder.decode(&input),
                        reference(&input, alphabet),
                        "{alphabet:?} {noise} {:?}",
                        String::from_utf8_lossy(&input)
                    );
                }
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_stream_matches_reference() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for &alphabet in &ALPHABETS {
            let decoder = Decoder::new(alphabet);
            for &noise in &[0.0, 0.01] {
                let input = random_morse(&mut rng, 20_000, noise);
                let expected = reference(&input, alphabet);
                for max_piece in [1, 2, 7, 9, 100, 5000] {
                    let mut stream = decoder.stream();
                    let mut output = String::new();
                    let mut rest = &input[..];
                    while !rest.is_empty() {
                        let n = rng.gen_range(0..=max_piece.min(rest.len()));
                        stream.feed(&rest[..n], &mut output);
                        rest = &rest[n..];
                    }
                    stream.finish(&mut output);
                    assert_eq!(output, expected, "{alphabet:?} {noise} {max_piece}");
                }
            }
        }
    }

    #[test]
    fn test_decode_strict() {
        let decoder = Decoder::default();
        assert_eq!(decoder.decode_strict(b"").unwrap(), "");
        assert_eq!(decoder.decode_strict(b"... / ...\n").unwrap(), "S S\n");
        let error = |s| decoder.decode_strict(s).unwrap_err();
        assert_eq!(
            error(b"... ..x"),
            DecodeError {
                position: 6,
                kind: DecodeErrorKind::InvalidByte(b'x')
            }
        );
        assert_eq!(
            error(b"... ........"),
            DecodeError {
                position: 4,
                kind: DecodeErrorKind::TooManyElements
            }
        );
        assert_eq!(
            error(b"... ......."),
            DecodeError {
                position: 4,
                kind: DecodeErrorKind::UnknownSequence
            }
        );
    }

    #[test]
    fn test_from_fn() {
        let decoder = Decoder::from_fn(crate::to_russian);
        assert_eq!(
            decoder.decode(b"- . .-.. . --. .-. .- ..-. .. .-.-"),
            "ТЕЛЕГРАФИЯ"
        );
    }
}
