//! Encoding of text to Morse code.
//!
//! The output uses:
//!
//! - `.` and `-` for Morse elements (dot and dash);
//! - ` ` (space) after each character, as letter space;
//! - `/` for spaces in the input (word space), followed by a letter space;
//! - `\t`, `\n` and `\r`, which are copied as-is, and replace a preceding letter space.
//!
//! Characters that cannot be encoded are ignored. The output does not end with a letter space.

use std::io::{Read, Write};

use crate::encode_ascii::{ENCODE_LEN, encode_buffer_ascii};
use crate::encode_unicode::encode_buffer_utf8;
use crate::encode_unicode_mapping::from_unicode;

/// Size of the input buffer used by [`Encoder::encode_stream`].
const STREAM_BUFFER_SIZE: usize = 1 << 17;

/// How an [`Encoder`] interprets its input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Charset {
    /// UTF-8 text, in any of the supported scripts; invalid UTF-8 bytes are skipped.
    #[default]
    Utf8,
    /// ASCII text: only the Latin alphabet, digits and punctuation are encoded, and non-ASCII
    /// bytes are skipped. This is faster than [`Charset::Utf8`] for ASCII text.
    Ascii,
}

/// Encodes text to Morse code.
///
/// See the [crate documentation](crate) for the format of the output.
///
/// ```
/// use ripmors::{Charset, Encoder};
///
/// let encoder = Encoder::new(Charset::Utf8);
/// assert_eq!(encoder.encode("Morse code".as_bytes()), "-- --- .-. ... . / -.-. --- -.. .");
/// assert_eq!(encoder.encode("télégraphie".as_bytes()), "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. .");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Encoder {
    pub(crate) charset: Charset,
}

impl Encoder {
    /// Create an encoder for the given input charset.
    pub const fn new(charset: Charset) -> Self {
        Encoder { charset }
    }

    /// Encode text into a new [`String`].
    pub fn encode(&self, input: &[u8]) -> String {
        let mut output = match self.charset {
            // NOTE: for ASCII, reserving for the worst case upfront is faster than growing
            Charset::Ascii => {
                String::with_capacity(input.len() * crate::encode_ascii::MAX_OUTPUT_PER_INPUT_BYTE)
            }
            Charset::Utf8 => String::new(),
        };
        self.encode_into(input, &mut output);
        output
    }

    /// Encode text, appending the result to `output`.
    ///
    /// Reusing the same `output` across calls avoids allocations.
    ///
    /// ```
    /// use ripmors::Encoder;
    ///
    /// let encoder = Encoder::default();
    /// let mut output = String::new();
    /// for text in ["SOS", "Paris"] {
    ///     output.clear();
    ///     encoder.encode_into(text.as_bytes(), &mut output);
    ///     println!("{output}");
    /// }
    /// ```
    pub fn encode_into(&self, input: &[u8], output: &mut String) {
        let mut stream = self.stream();
        stream.feed(input, output);
        stream.finish(output);
    }

    /// Encode text into a new [`String`], failing on the first character that cannot be
    /// encoded, or on invalid UTF-8.
    ///
    /// ```
    /// use ripmors::{EncodeErrorKind, Encoder};
    ///
    /// let encoder = Encoder::default();
    /// assert_eq!(encoder.encode_strict(b"SOS").unwrap(), "... --- ...");
    /// let error = encoder.encode_strict("S☺S".as_bytes()).unwrap_err();
    /// assert_eq!(error.position(), 1);
    /// assert_eq!(error.kind(), EncodeErrorKind::UnsupportedCharacter('☺'));
    /// ```
    pub fn encode_strict(&self, input: &[u8]) -> Result<String, EncodeError> {
        let error = |position, kind| Err(EncodeError { position, kind });
        match self.charset {
            Charset::Ascii => {
                for (position, &c) in input.iter().enumerate() {
                    if !c.is_ascii() || ENCODE_LEN[c as usize] == 0 {
                        return error(position, EncodeErrorKind::UnsupportedByte(c));
                    }
                }
            }
            Charset::Utf8 => {
                let text = match std::str::from_utf8(input) {
                    Ok(text) => text,
                    Err(e) => return error(e.valid_up_to(), EncodeErrorKind::InvalidUtf8),
                };
                for (position, c) in text.char_indices() {
                    let supported = if c.is_ascii() {
                        ENCODE_LEN[c as usize] != 0
                    } else {
                        from_unicode(c).1 != 0
                    };
                    if !supported {
                        return error(position, EncodeErrorKind::UnsupportedCharacter(c));
                    }
                }
            }
        }
        Ok(self.encode(input))
    }

    /// Encode text from a [`Read`] object into a [`Write`] object, until the end of the input.
    ///
    /// ```no_run
    /// let mut stdin = std::io::stdin().lock();
    /// let mut stdout = std::io::stdout().lock();
    /// ripmors::Encoder::default().encode_stream(&mut stdin, &mut stdout).unwrap();
    /// ```
    pub fn encode_stream(
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

    /// Start encoding text incrementally, see [`EncodeStream`].
    pub fn stream(&self) -> EncodeStream {
        EncodeStream {
            charset: self.charset,
            pending_space: false,
            truncated: [0; 4],
            truncated_len: 0,
        }
    }
}

/// Incremental encoding of text to Morse code, created by [`Encoder::stream`].
///
/// The input can be fed in arbitrary pieces, even in the middle of a UTF-8 character; the output
/// is the same as encoding the concatenation of all the pieces at once.
///
/// ```
/// use ripmors::Encoder;
///
/// let mut stream = Encoder::default().stream();
/// let mut output = String::new();
/// let text = "télé graphe".as_bytes();
/// stream.feed(&text[..2], &mut output); // the first byte of "é"
/// stream.feed(&text[2..], &mut output);
/// stream.finish(&mut output);
/// assert_eq!(output, "- ..-.. .-.. ..-.. / --. .-. .- .--. .... .");
/// ```
#[derive(Clone, Debug)]
pub struct EncodeStream {
    charset: Charset,
    /// Whether the output so far ends with a letter space that was not written yet, since it
    /// is replaced by following whitespace, and removed at the end
    pending_space: bool,
    /// The beginning of a UTF-8 character truncated at the end of the last input
    truncated: [u8; 4],
    truncated_len: usize,
}

impl EncodeStream {
    /// Whether the output so far ends with a letter space that was held back (see
    /// [`EncodeStream::feed`]).
    pub(crate) fn pending_space(&self) -> bool {
        self.pending_space
    }

    /// Encode the next part of the input, appending the result to `output`.
    pub fn feed(&mut self, mut input: &[u8], output: &mut String) {
        // SAFETY: the encoders only append ASCII to `output`
        let output_buf = unsafe { output.as_mut_vec() };
        let floor = output_buf.len();
        if self.pending_space {
            output_buf.push(b' ');
            self.pending_space = false;
        }
        match self.charset {
            Charset::Ascii => encode_buffer_ascii(input, output_buf, floor),
            Charset::Utf8 => {
                if self.truncated_len != 0 {
                    // complete the truncated character with the first bytes of the input
                    let mut buf = [0; 7];
                    let k = input.len().min(3);
                    buf[..self.truncated_len]
                        .copy_from_slice(&self.truncated[..self.truncated_len]);
                    buf[self.truncated_len..self.truncated_len + k].copy_from_slice(&input[..k]);
                    let buf = &buf[..self.truncated_len + k];
                    let processed = encode_buffer_utf8(buf, output_buf, floor);
                    if processed < self.truncated_len {
                        // the character is still truncated; this only happens when all the input
                        // is part of it
                        debug_assert_eq!(k, input.len());
                        self.truncated_len = buf.len() - processed;
                        self.truncated[..self.truncated_len].copy_from_slice(&buf[processed..]);
                        input = &[];
                    } else {
                        // NOTE: a truncated character in `buf` is processed again with the rest
                        // of the input
                        input = &input[processed - self.truncated_len..];
                        self.truncated_len = 0;
                    }
                }
                let processed = encode_buffer_utf8(input, output_buf, floor);
                let rest = &input[processed..];
                self.truncated[self.truncated_len..self.truncated_len + rest.len()]
                    .copy_from_slice(rest);
                self.truncated_len += rest.len();
            }
        }
        if output_buf.len() > floor && output_buf.last() == Some(&b' ') {
            output_buf.pop();
            self.pending_space = true;
        }
    }

    /// Finish encoding, appending the remaining output (if any) to `output`.
    ///
    /// A UTF-8 character truncated at the end of the input is skipped, and the final letter space
    /// is removed.
    pub fn finish(self, output: &mut String) {
        // nothing is written currently, but this may change
        let _ = output;
    }
}

/// The reason of an [`EncodeError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EncodeErrorKind {
    /// Invalid UTF-8 (with [`Charset::Utf8`])
    InvalidUtf8,
    /// A character that cannot be encoded to Morse code (with [`Charset::Utf8`])
    UnsupportedCharacter(char),
    /// A byte that cannot be encoded to Morse code (with [`Charset::Ascii`])
    UnsupportedByte(u8),
}

/// Error returned by [`Encoder::encode_strict`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EncodeError {
    position: usize,
    kind: EncodeErrorKind,
}

impl EncodeError {
    /// Position in the input of the first byte of the problematic character or UTF-8 sequence.
    pub fn position(&self) -> usize {
        self.position
    }

    /// The reason of the error.
    pub fn kind(&self) -> EncodeErrorKind {
        self.kind
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            EncodeErrorKind::InvalidUtf8 => {
                write!(f, "invalid UTF-8 at position {}", self.position)
            }
            EncodeErrorKind::UnsupportedCharacter(c) => write!(
                f,
                "character {c:?} at position {} cannot be encoded",
                self.position
            ),
            EncodeErrorKind::UnsupportedByte(c) => write!(
                f,
                "byte {c:#04x} at position {} cannot be encoded",
                self.position
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

#[cfg(any(test, fuzzing))]
/// Straightforward implementation of the encoding rules.
pub(crate) fn reference(input: &[u8], charset: Charset) -> String {
    let text: Vec<char> = match charset {
        Charset::Ascii => input
            .iter()
            .filter(|c| c.is_ascii())
            .map(|&c| c as char)
            .collect(),
        Charset::Utf8 => String::from_utf8_lossy(input)
            .chars()
            .filter(|&c| c != char::REPLACEMENT_CHARACTER)
            .collect(),
    };
    let mut output = String::new();
    for c in text {
        if matches!(c, '\t' | '\n' | '\r') {
            if output.ends_with(' ') {
                output.pop();
            }
            output.push(c);
        } else if c == '%' {
            output.push_str("----- -..-. ----- ");
        } else {
            let (bytes, len) = from_unicode(c);
            output.push_str(std::str::from_utf8(&bytes[..len]).unwrap());
        }
    }
    if output.ends_with(' ') {
        output.pop();
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_text(rng: &mut impl rand::Rng, len: usize) -> Vec<u8> {
        const CHARS: &[char] = &[
            'a', 'B', 'z', '0', '%', ' ', ' ', '\n', '\t', '\r', '?', '#', 'é', 'ß', 'Я', 'Ω',
            'モ', 'ㄱ', 'א', 'ب', '☺', '\0',
        ];
        let mut bytes = Vec::new();
        while bytes.len() < len {
            if rng.gen_bool(0.02) {
                bytes.push(rng.r#gen());
            } else {
                let c = CHARS[rng.gen_range(0..CHARS.len())];
                bytes.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
            }
        }
        bytes
    }

    #[test]
    fn test_ascii_encode_simple() {
        let f = |s: &[u8]| Encoder::new(Charset::Ascii).encode(s);
        assert_eq!(f(b"PARIS"), ".--. .- .-. .. ...");
        assert_eq!(
            f(b"Hello, World!"),
            ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--"
        );
        assert_eq!(
            f(b"one line\nand  another\tline"),
            "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
        );
    }

    #[test]
    fn test_unicode_encode() {
        let f = |s: &str| Encoder::default().encode(s.as_bytes());
        assert_eq!(
            f("télégraphie"),
            "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. ."
        );
        assert_eq!(f("でんしん"), ".-.-- .. .-.-. --.-. .-.-.");
        assert_eq!(f("تلغراف"), "- .-.. --. .-. .- ..-.");
        assert_eq!(f("телеграфия"), "- . .-.. . --. .-. .- ..-. .. .-.-");
        assert_eq!(f("τηλεγραφία"), "- .... .-.. . --. .-. .- ..-. .. .-");
        assert_eq!(
            f("one line\nand  another\tline"),
            "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
        );
    }

    #[test]
    fn test_invalid_utf8_is_skipped() {
        let encoder = Encoder::default();
        assert_eq!(encoder.encode(b"ab\xffcd"), ".- -... -.-. -..");
        let mut input = b"ab\xff".to_vec();
        input.extend(std::iter::repeat_n(b'e', 200_000));
        assert_eq!(encoder.encode(&input).len(), 7 + 200_000 * 2 - 1 + 1);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_encode_matches_reference() {
        let mut rng = rand::thread_rng();
        for charset in [Charset::Utf8, Charset::Ascii] {
            let encoder = Encoder::new(charset);
            for len in (0..300).chain([5000, 70_000]) {
                let input = random_text(&mut rng, len);
                assert_eq!(
                    encoder.encode(&input),
                    reference(&input, charset),
                    "{charset:?} {:?}",
                    String::from_utf8_lossy(&input)
                );
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_stream_matches_reference() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for charset in [Charset::Utf8, Charset::Ascii] {
            let encoder = Encoder::new(charset);
            let input = random_text(&mut rng, 20_000);
            let expected = reference(&input, charset);
            for max_piece in [1, 2, 3, 5, 100, 5000] {
                let mut stream = encoder.stream();
                let mut output = String::new();
                let mut rest = &input[..];
                while !rest.is_empty() {
                    let n = rng.gen_range(0..=max_piece.min(rest.len()));
                    stream.feed(&rest[..n], &mut output);
                    rest = &rest[n..];
                }
                stream.finish(&mut output);
                assert_eq!(output, expected, "{charset:?} {max_piece}");
            }
        }
    }

    #[test]
    fn test_encode_into_preserves_existing_output() {
        let mut output = String::from("prefix ");
        Encoder::default().encode_into(b"\nE", &mut output);
        assert_eq!(output, "prefix \n.");
    }

    #[test]
    fn test_encode_strict() {
        let utf8 = Encoder::new(Charset::Utf8);
        assert_eq!(
            utf8.encode_strict("été".as_bytes()).unwrap(),
            "..-.. - ..-.."
        );
        let error = utf8.encode_strict(b"ab\xff").unwrap_err();
        assert_eq!(
            (error.position(), error.kind()),
            (2, EncodeErrorKind::InvalidUtf8)
        );
        let error = utf8.encode_strict(b"a#").unwrap_err();
        assert_eq!(
            (error.position(), error.kind()),
            (1, EncodeErrorKind::UnsupportedCharacter('#'))
        );
        let ascii = Encoder::new(Charset::Ascii);
        let error = ascii.encode_strict("aé".as_bytes()).unwrap_err();
        assert_eq!(
            (error.position(), error.kind()),
            (1, EncodeErrorKind::UnsupportedByte(0xc3))
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ascii_encode_random_large() {
        use rand::{Rng, distributions::Standard};
        let data: Vec<u8> = rand::thread_rng()
            .sample_iter::<u8, _>(Standard)
            .take(1048576)
            .collect();
        Encoder::new(Charset::Ascii).encode(&data);
        Encoder::new(Charset::Utf8).encode(&data);
    }
}
