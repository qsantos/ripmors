#[cfg(target_arch = "x86_64")]
use crate::encode_ascii::avx512;
use crate::encode_ascii::{ENCODE_CHUNK, MAX_OUTPUT_PER_INPUT_BYTE, Simd, encode_ascii_byte};
use crate::encode_unicode_mapping::{Entry2, Entry3, lookup_utf8_2, lookup_utf8_3};

/// Encode a run of mostly non-ASCII characters starting at `input[i]`, stopping when reaching a
/// run of four ASCII bytes or once `end` is reached. Returns the new values of `i` and `cur`. See
/// [`encode_ascii_byte`] for `floor`.
///
/// # Safety
///
/// - `input` must be valid UTF-8, and `i < end <= input.len()` must be on a character boundary;
/// - `out` must be valid for writes of `cur + (end - i + 3) * MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `cur` bytes of `out` must be initialized.
#[inline(never)]
unsafe fn encode_non_ascii_run(
    input: &[u8],
    mut i: usize,
    end: usize,
    out: *mut u8,
    mut cur: usize,
    floor: usize,
) -> (usize, usize) {
    let p = input.as_ptr();
    // NOTE: in the comments below, "reserved" refers to the `MAX_OUTPUT_PER_INPUT_BYTE` bytes per
    // input byte that the caller guarantees for writing at `out + cur`
    loop {
        // SAFETY: `i < end <= input.len()`
        let b0 = unsafe { *p.add(i) };
        if (b0 as i8) < (0xe0u8 as i8) {
            // lead byte of a two-byte sequence (since `i` is on a character boundary)
            // SAFETY: `input` is valid UTF-8 so the sequence is complete; we reserved 36 bytes
            unsafe {
                let entry = lookup_utf8_2(b0, *p.add(i + 1));
                out.add(cur).cast::<Entry2>().write_unaligned(*entry);
                cur += entry.len();
            }
            i += 2;
        } else if b0 < 0x80 {
            // isolated ASCII characters (e.g. spaces and punctuation between words) are handled
            // here to avoid leaving the loop; we leave it once we reach a run of four ASCII bytes
            if let Some(quad) = input.get(i..i + 4)
                && u32::from_ne_bytes(quad.try_into().unwrap()) & 0x80808080 == 0
            {
                return (i, cur);
            }
            // SAFETY: we reserved 18 bytes, and the first `cur` bytes are initialized
            cur = unsafe { encode_ascii_byte(b0, out, cur, floor) };
            i += 1;
        } else if b0 < 0xf0 {
            // lead byte of a three-byte sequence; runs of them (e.g. in Japanese) are handled in a
            // tight loop
            let mut b0 = b0;
            loop {
                // SAFETY: `input` is valid UTF-8 so the sequence is complete; we reserved 54 bytes
                unsafe {
                    let entry = lookup_utf8_3(b0, *p.add(i + 1), *p.add(i + 2));
                    out.add(cur).cast::<Entry3>().write_unaligned(*entry);
                    cur += entry.len();
                }
                i += 3;
                if i >= end {
                    return (i, cur);
                }
                // SAFETY: `i < end <= input.len()`
                b0 = unsafe { *p.add(i) };
                if !(0xe0..0xf0).contains(&b0) {
                    break;
                }
            }
            continue;
        } else {
            // characters outside the Basic Multilingual Plane have no Morse code
            i += 4;
        }
        if i >= end {
            return (i, cur);
        }
    }
}

/// Minimum length of a run of ASCII bytes for the AVX-512 code to be used; shorter runs are
/// faster with the scalar code.
#[cfg(target_arch = "x86_64")]
const MIN_AVX512_RUN: usize = 16;

/// Encode the run of ASCII bytes at the start of `block`, with AVX-512 if it is long enough, and
/// return its length. See [`encode_ascii_byte`] for `floor`.
///
/// # Safety
///
/// - the CPU must support the features checked by [`avx512::is_available`];
/// - `out` must be valid for writes of `*cur + 64 * MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `*cur` bytes of `out` must be initialized.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,bmi2,popcnt")]
unsafe fn ascii_run_avx512(block: &[u8; 64], out: *mut u8, cur: &mut usize, floor: usize) -> usize {
    use std::arch::x86_64::*;
    // SAFETY: `block` has 64 bytes
    let chars = unsafe { _mm512_loadu_si512(block.as_ptr().cast()) };
    let run = _mm512_movepi8_mask(chars).trailing_zeros() as usize;
    if run >= MIN_AVX512_RUN {
        // SAFETY: forwarded from the caller; the first `run` bytes of `block` are ASCII
        *cur = unsafe { avx512::encode_ascii_prefix(block, run, out, *cur, floor) };
        return run;
    }
    for &c in &block[..run] {
        // SAFETY: forwarded from the caller
        *cur = unsafe { encode_ascii_byte(c, out, *cur, floor) };
    }
    run
}

/// Encode valid UTF-8 and append the result to `output_buf`; see
/// [`encode_ascii_byte`][crate::encode_ascii::encode_ascii_byte] for `floor`.
#[cfg_attr(not(target_arch = "x86_64"), allow(unused_variables))]
fn encode_buffer(input: &str, output_buf: &mut Vec<u8>, floor: usize, simd: Simd) {
    let bytes = input.as_bytes();
    #[cfg(target_arch = "x86_64")]
    let avx512 = simd.avx512();
    let mut i = 0;
    while i < bytes.len() {
        // a multi-byte character, a group of four ASCII bytes or a block of 64 bytes starting
        // before `end` may overrun it by at most 63 bytes; we reserve room for them as well
        let end = bytes.len().min(i + ENCODE_CHUNK);
        output_buf.reserve((end - i + 63) * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        while i < end {
            #[cfg(target_arch = "x86_64")]
            if avx512 && let Some(block) = bytes.get(i..i + 64) {
                // SAFETY: we checked that the CPU supports the required features, `block` has 64
                // bytes, we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the chunk
                // plus 63, and the first `cur` bytes are initialized
                let run =
                    unsafe { ascii_run_avx512(block.try_into().unwrap(), out, &mut cur, floor) };
                if run == 0 {
                    // SAFETY: `input` is valid UTF-8, `i` is on a character boundary since we
                    // advance by whole characters, and `bytes[i]` is not ASCII; we reserved
                    // `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the chunk plus 63
                    (i, cur) = unsafe { encode_non_ascii_run(bytes, i, end, out, cur, floor) };
                } else {
                    i += run;
                }
                continue;
            }
            // fast path for runs of ASCII bytes, unrolled by four
            if let Some(quad) = bytes.get(i..i + 4)
                && u32::from_ne_bytes(quad.try_into().unwrap()) & 0x80808080 == 0
            {
                for &c in quad {
                    // SAFETY: we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the
                    // chunk plus 63, and the first `cur` bytes are initialized since we only
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
                // SAFETY: `input` is valid UTF-8, `i` is always on a character boundary since we
                // advance by whole characters, and `bytes[i]` is not ASCII; we reserved
                // `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the chunk plus 63
                (i, cur) = unsafe { encode_non_ascii_run(bytes, i, end, out, cur, floor) };
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
    encode_buffer_utf8_with(input, output_buf, floor, Simd::Auto)
}

pub(crate) fn encode_buffer_utf8_with(
    input: &[u8],
    output_buf: &mut Vec<u8>,
    floor: usize,
    simd: Simd,
) -> usize {
    let mut pos = 0;
    loop {
        match simdutf8::compat::from_utf8(&input[pos..]) {
            Ok(valid) => {
                encode_buffer(valid, output_buf, floor, simd);
                return input.len();
            }
            Err(e) => {
                let valid_up_to = pos + e.valid_up_to();
                // SAFETY: the input is valid UTF-8 up to `valid_up_to`
                let valid = unsafe { std::str::from_utf8_unchecked(&input[pos..valid_up_to]) };
                encode_buffer(valid, output_buf, floor, simd);
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

#[test]
#[cfg_attr(miri, ignore)]
fn test_unicode_encode_simd_matches_scalar() {
    use rand::{RngExt, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(43);
    let iterations = if cfg!(debug_assertions) { 1000 } else { 20000 };
    for iteration in 0..iterations {
        let len = if iteration % 10 == 0 {
            rng.random_range(0..20000)
        } else {
            rng.random_range(0..300)
        };
        // mix ASCII runs with non-ASCII characters of every UTF-8 length
        let mut data = Vec::new();
        while data.len() < len {
            if rng.random_bool(0.7) {
                let run = rng.random_range(0..100);
                data.extend(crate::encode_ascii::tricky_input(&mut rng, run));
            } else {
                for _ in 0..rng.random_range(1..20) {
                    let c = ["é", "Ж", "щ", "Љ", "ת", "ي", "が", "東", "‰", "ー", "😀"]
                        [rng.random_range(0..11)];
                    data.extend_from_slice(c.as_bytes());
                }
            }
        }
        for (before_floor, after_floor) in crate::encode_ascii::TEST_PREFIXES {
            let run = |simd| {
                let mut output = before_floor.to_vec();
                let floor = output.len();
                output.extend_from_slice(after_floor);
                let processed = encode_buffer_utf8_with(&data, &mut output, floor, simd);
                (processed, output)
            };
            assert_eq!(run(Simd::Auto), run(Simd::Never), "{data:?}");
        }
    }
}
