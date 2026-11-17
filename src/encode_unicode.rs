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

/// Encode valid UTF-8 and append the result to `output_buf`; see
/// [`encode_ascii_byte`][crate::encode_ascii::encode_ascii_byte] for `floor`.
fn encode_buffer(input: &str, output_buf: &mut Vec<u8>, floor: usize) {
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // a multi-byte character or a group of four ASCII bytes starting before `end` may overrun
        // it by at most 3 bytes; we reserve room for them as well
        let end = bytes.len().min(i + ENCODE_CHUNK);
        output_buf.reserve((end - i + 3) * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        while i < end {
            // fast path for runs of ASCII bytes, unrolled by four
            if let Some(quad) = bytes.get(i..i + 4)
                && u32::from_ne_bytes(quad.try_into().unwrap()) & 0x80808080 == 0
            {
                for &c in quad {
                    // SAFETY: we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the
                    // chunk plus 3, and the first `cur` bytes are initialized since we only
                    // increase `cur` after writing
                    cur = unsafe { encode_ascii_byte(c, out, cur, floor) };
                }
                i += 4;
                continue;
            }
            let c = bytes[i];
            if c < 0x80 {
                // SAFETY: we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the
                // chunk, and the first `cur` bytes are initialized since we only increase `cur`
                // after writing
                cur = unsafe { encode_ascii_byte(c, out, cur, floor) };
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

/// Encode UTF-8 and append the result to `output_buf`, skipping invalid bytes; see
/// [`encode_ascii_byte`][crate::encode_ascii::encode_ascii_byte] for `floor`.
///
/// Return the position of the first byte not processed: a truncated character at the end of
/// `input`, which may be completed by the next input.
pub(crate) fn encode_buffer_utf8(input: &[u8], output_buf: &mut Vec<u8>, floor: usize) -> usize {
    let mut pos = 0;
    loop {
        match simdutf8::compat::from_utf8(&input[pos..]) {
            Ok(valid) => {
                encode_buffer(valid, output_buf, floor);
                return input.len();
            }
            Err(e) => {
                let valid_up_to = pos + e.valid_up_to();
                // SAFETY: the input is valid UTF-8 up to `valid_up_to`
                let valid = unsafe { std::str::from_utf8_unchecked(&input[pos..valid_up_to]) };
                encode_buffer(valid, output_buf, floor);
                match e.error_len() {
                    // skip the invalid bytes
                    Some(n) => pos = valid_up_to + n,
                    // truncated character
                    None => return valid_up_to,
                }
            }
        }
    }
}
