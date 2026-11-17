//! Decoding with AVX-512 (VBMI and VBMI2).
//!
//! The input is processed in blocks of [`STEP`] bytes. For each block, we load 64 bytes starting 8
//! bytes before it, so that every Morse sequence of up to 7 elements ending in the block is fully
//! visible. Then:
//!
//! 1. we compute the bitmask of separators, and the "code" bitmask, where bit `k` is the least
//!    significant bit of byte `k` (1 for `-`, 0 for `.`), or 1 for separators;
//! 2. for each byte, we compute the start of the sequence it ends (one past the previous
//!    separator) with a running maximum over the 8 previous bytes;
//! 3. with `vpmultishiftqb`, we extract the 8 bits of the code bitmask from this start, and mask
//!    them according to the length of the sequence; for separators, this is the same value as
//!    `morse_to_binary_fast`, since the separator itself provides the leading one;
//! 4. we look up the decoded ASCII character with `vpermi2b`;
//! 5. we interleave decoded characters and separators with `vpermt2b`, and pack the ones we keep
//!    with `vpcompressb`.
//!
//! Blocks with sequences of more than 7 elements, or decoding to non-ASCII characters, are
//! handled by the scalar code.

use std::arch::x86_64::*;

use super::{DecodeTable, MAX_OUTPUT_PER_INPUT_BYTE, NON_ASCII, decode_separators};

/// Number of input bytes consumed per block.
const STEP: usize = 56;

/// Number of bytes before a block that are loaded along with it.
const LOOK_BACK: usize = 64 - STEP;

/// Minimum input length for [`decode_blocks`] to process at least one block.
pub(super) const MIN_INPUT_LEN: usize = LOOK_BACK + 64;

/// Output space needed for each block: either the scalar fallback for 64 bytes, or two 64-byte
/// stores.
const OUTPUT_PER_BLOCK: usize = 64 * MAX_OUTPUT_PER_INPUT_BYTE;

/// Number of blocks processed between two reservations of output space.
const BLOCKS_PER_RESERVE: usize = 64;

pub(super) fn is_available() -> bool {
    is_x86_feature_detected!("avx512f")
        && is_x86_feature_detected!("avx512bw")
        && is_x86_feature_detected!("avx512vbmi")
        && is_x86_feature_detected!("avx512vbmi2")
        && is_x86_feature_detected!("bmi2")
        && is_x86_feature_detected!("popcnt")
}

const fn iota(offset: u8) -> [u8; 64] {
    let mut x = [0; 64];
    let mut i = 0;
    while i < 64 {
        x[i] = (i as u8).wrapping_add(offset);
        i += 1;
    }
    x
}

/// Index for `vpermb` to move each byte `k` lanes up (lanes below `k` must be masked).
const fn shift_up(k: u8) -> [u8; 64] {
    let mut x = iota(0u8.wrapping_sub(k));
    let mut i = 0;
    while i < 64 {
        x[i] &= 63;
        i += 1;
    }
    x
}

/// Index for `vpermt2b` interleaving bytes `half * 32..half * 32 + 32` of two vectors.
const fn interleave(half: u8) -> [u8; 64] {
    let mut x = [0; 64];
    let mut k = 0;
    while k < 32 {
        x[2 * k] = half * 32 + k as u8;
        x[2 * k + 1] = 64 + half * 32 + k as u8;
        k += 1;
    }
    x
}

/// For each length `n` of a Morse sequence, the mask for its `n` elements and the separator.
const MASK_BY_LEN: [u8; 64] = {
    let mut x = [0; 64];
    let mut i = 0;
    while i < 64 {
        if i % 16 < 8 {
            x[i] = ((2u32 << (i % 16)) - 1) as u8;
        }
        i += 1;
    }
    x
};

static IOTA: [u8; 64] = iota(0);
static IOTA_PLUS_ONE: [u8; 64] = iota(1);
static SHIFT_UP_1: [u8; 64] = shift_up(1);
static SHIFT_UP_2: [u8; 64] = shift_up(2);
static SHIFT_UP_4: [u8; 64] = shift_up(4);
static INTERLEAVE_LO: [u8; 64] = interleave(0);
static INTERLEAVE_HI: [u8; 64] = interleave(1);

/// Load 64 bytes.
///
/// # Safety
///
/// `p` must be valid for reads of 64 bytes.
#[inline(always)]
unsafe fn load(p: *const u8) -> __m512i {
    // SAFETY: forwarded from the caller
    unsafe { _mm512_loadu_si512(p.cast()) }
}

/// Decode blocks of input with AVX-512, starting at `i`, as long as there is enough input; return
/// the position of the first byte not processed, and the start of the current Morse sequence.
///
/// # Safety
///
/// - the CPU must support the features checked by [`is_available`];
/// - `LOOK_BACK <= i`, all the bytes before `i` must have been decoded, and `chunk_start` must be
///   the start of the current Morse sequence.
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,bmi2,popcnt")]
pub(super) unsafe fn decode_blocks(
    input: &[u8],
    mut i: usize,
    mut chunk_start: usize,
    table: &DecodeTable,
    output_buf: &mut Vec<u8>,
) -> (usize, usize) {
    debug_assert!(i >= LOOK_BACK);
    let ptr = input.as_ptr();
    // SAFETY: all these arrays have at least 64 bytes
    let (lut0, lut1, lut2, lut3, iota, iota_plus_one, shift1, shift2, shift4, inter_lo, inter_hi) = unsafe {
        (
            load(table.ascii.as_ptr()),
            load(table.ascii.as_ptr().add(64)),
            load(table.ascii.as_ptr().add(128)),
            load(table.ascii.as_ptr().add(192)),
            load(IOTA.as_ptr()),
            load(IOTA_PLUS_ONE.as_ptr()),
            load(SHIFT_UP_1.as_ptr()),
            load(SHIFT_UP_2.as_ptr()),
            load(SHIFT_UP_4.as_ptr()),
            load(INTERLEAVE_LO.as_ptr()),
            load(INTERLEAVE_HI.as_ptr()),
        )
    };
    // SAFETY: `MASK_BY_LEN` has 64 bytes
    let mask_by_len = unsafe { load(MASK_BY_LEN.as_ptr()) };
    let space = _mm512_set1_epi8(b' ' as i8);
    let slash = _mm512_set1_epi8(b'/' as i8);
    let one = _mm512_set1_epi8(1);
    let non_ascii = _mm512_set1_epi8(NON_ASCII as i8);

    // the scalar fallback needs 7 bytes after the 64 loaded bytes
    while i + STEP + 7 < input.len() {
        let blocks = ((input.len() - i - STEP - 8) / STEP + 1).min(BLOCKS_PER_RESERVE);
        output_buf.reserve(blocks * OUTPUT_PER_BLOCK);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        for _ in 0..blocks {
            let base = i - LOOK_BACK;
            // SAFETY: `base + 64 = i + STEP <= input.len()`
            let v = unsafe { load(ptr.add(base)) };
            let slash_mask = _mm512_cmpeq_epi8_mask(v, slash);
            let separators = _mm512_cmple_epu8_mask(v, space) | slash_mask;
            // separators in the look-back bytes were already processed
            let own = separators & !((1 << LOOK_BACK) - 1);
            if own == 0 {
                i += STEP;
                continue;
            }
            let code = _mm512_test_epi8_mask(v, one) | separators;

            // start of the sequence ending at each byte (one past the previous separator) when
            // there is a separator in the 8 previous bytes, zero otherwise
            let x = _mm512_maskz_mov_epi8(separators, iota_plus_one);
            let y = _mm512_maskz_permutexvar_epi8(!1, shift1, x);
            let y = _mm512_max_epu8(y, _mm512_maskz_permutexvar_epi8(!1, shift1, y));
            let y = _mm512_max_epu8(y, _mm512_maskz_permutexvar_epi8(!3, shift2, y));
            let start = _mm512_max_epu8(y, _mm512_maskz_permutexvar_epi8(!0xf, shift4, y));
            let short = _mm512_test_epi8_mask(start, start);

            // decode
            let bits = _mm512_multishift_epi64_epi8(start, _mm512_set1_epi64(code as i64));
            let len = _mm512_sub_epi8(iota, start);
            let binary = _mm512_and_si512(bits, _mm512_shuffle_epi8(mask_by_len, len));
            let decoded = _mm512_mask_blend_epi8(
                _mm512_movepi8_mask(binary),
                _mm512_permutex2var_epi8(lut0, binary, lut1),
                _mm512_permutex2var_epi8(lut2, binary, lut3),
            );
            let decoding = own & !slash_mask;

            if own & !short != 0 || _mm512_mask_cmpeq_epi8_mask(decoding, decoded, non_ascii) != 0 {
                // SAFETY: the bits of `own` are separators after `chunk_start`,
                // `base + 64 + 7 <= input.len()`, and we reserved `OUTPUT_PER_BLOCK` bytes for
                // each block
                (cur, chunk_start) =
                    unsafe { decode_separators(input, base, own, chunk_start, table, out, cur) };
            } else {
                // output the decoded character (if any), then the separator, unless it is a
                // letter space; word spaces are output as spaces
                let char_mask = decoding & _mm512_test_epi8_mask(decoded, decoded);
                let sep_mask = own & !_mm512_cmpeq_epi8_mask(v, space);
                let sep = _mm512_mask_mov_epi8(v, slash_mask, space);
                let lo = _mm512_permutex2var_epi8(decoded, inter_lo, sep);
                let hi = _mm512_permutex2var_epi8(decoded, inter_hi, sep);
                let lo_mask = _pdep_u64(char_mask, 0x5555_5555_5555_5555)
                    | _pdep_u64(sep_mask, 0xaaaa_aaaa_aaaa_aaaa);
                let hi_mask = _pdep_u64(char_mask >> 32, 0x5555_5555_5555_5555)
                    | _pdep_u64(sep_mask >> 32, 0xaaaa_aaaa_aaaa_aaaa);
                // NOTE: we use register compression and full stores, since compressing to
                // memory is very slow on Zen 4
                // SAFETY: we reserved `OUTPUT_PER_BLOCK >= 128` bytes for each block
                unsafe {
                    _mm512_storeu_si512(
                        out.add(cur).cast(),
                        _mm512_maskz_compress_epi8(lo_mask, lo),
                    );
                    cur += lo_mask.count_ones() as usize;
                    _mm512_storeu_si512(
                        out.add(cur).cast(),
                        _mm512_maskz_compress_epi8(hi_mask, hi),
                    );
                    cur += hi_mask.count_ones() as usize;
                }
                chunk_start = base + 64 - own.leading_zeros() as usize;
            }
            i += STEP;
        }
        // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase
        // `cur` after writing to `output_buf`
        unsafe { output_buf.set_len(cur) };
    }
    (i, chunk_start)
}
