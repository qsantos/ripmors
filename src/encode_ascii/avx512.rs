//! Encoding of ASCII with AVX-512 (VBMI).
//!
//! The input is processed in blocks of 64 bytes. For each block:
//!
//! 1. we classify the bytes with `vpermi2b` (encoded as nothing, whitespace, `%`) to detect the
//!    blocks that need the scalar code;
//! 2. we map each character to the index of its encoding among the (less than 64) distinct
//!    encodings with `vpermi2b`, plus 64 when it is followed by whitespace (it then loses its
//!    letter space); then, for each `j`, we look up byte `j` of the 8-byte encoding (padded with
//!    NUL bytes, see [`ENCODE_QWORD`]) with `vpermi2b`, giving 8 vectors;
//! 3. we transpose them with three rounds of in-lane unpacks (the characters are permuted
//!    beforehand to make up for the lanes), so that the encodings of each group of 8 characters
//!    are in the 8 qwords of a vector (`vpgatherqq` is much slower on Zen 4, and so is a
//!    transposition with `vpermt2b`);
//! 4. for each group, we pack the non-NUL bytes with `vpcompressb`, and store the 64 bytes; only
//!    the packed bytes are kept, the rest is overwritten by the next store.
//!
//! Blocks containing `%` (encoded as more than 8 bytes), or whitespace preceded by a character
//! encoded as nothing (the letter space to drop is then further back) are handled by the scalar
//! code.

use std::arch::x86_64::*;

use super::{ENCODE_QWORD, encode_ascii_byte};

pub(crate) fn is_available() -> bool {
    // Miri does not support most AVX-512 instructions
    !cfg!(miri)
        && is_x86_feature_detected!("avx512f")
        && is_x86_feature_detected!("avx512bw")
        && is_x86_feature_detected!("avx512vbmi")
        && is_x86_feature_detected!("avx512vbmi2")
        && is_x86_feature_detected!("bmi2")
        && is_x86_feature_detected!("popcnt")
}

/// Class bit: the character is encoded as at least one byte.
const NON_EMPTY: u8 = 1;
/// Class bit: the character is whitespace, and replaces a preceding letter space.
const WHITESPACE: u8 = 2;
/// Class bit: the character must be handled by the scalar code.
const SPECIAL: u8 = 4;

/// Classes of ASCII characters.
static CLASS: [u8; 128] = {
    let mut x = [0; 128];
    let mut c = 0;
    while c < 128 {
        let (qword, len) = crate::encode_ascii_mapping::ASCII_TO_QWORD[c];
        if c == b'\t' as usize || c == b'\n' as usize || c == b'\r' as usize {
            x[c] = NON_EMPTY | WHITESPACE;
        } else if len > 8 {
            x[c] = NON_EMPTY | SPECIAL;
        } else if len > 0 {
            // the vector code relies on non-empty encodings having no NUL byte
            assert!(qword.to_le_bytes()[len - 1] != 0);
            x[c] = NON_EMPTY;
        } else {
            assert!(qword == 0);
        }
        c += 1;
    }
    x
};

/// Distinct encodings of ASCII characters of at most 8 bytes, as qwords (the first one is
/// empty), and the index of the encoding of each ASCII character (zero for `%`).
static ENCODINGS: ([u64; 64], [u8; 128]) = {
    let mut encodings = [0; 64];
    let mut ids = [0; 128];
    let mut count = 1;
    let mut c = 0;
    while c < 128 {
        let qword = ENCODE_QWORD[c];
        let mut id = 0;
        while id < count && encodings[id] != qword {
            id += 1;
        }
        if id == count {
            encodings[id] = qword;
            count += 1;
        }
        ids[c] = id as u8;
        c += 1;
    }
    (encodings, ids)
};

/// Whether an encoding is whitespace.
const fn is_whitespace(qword: u64) -> bool {
    qword == b'\t' as u64 || qword == b'\n' as u64 || qword == b'\r' as u64
}

/// Byte `j` of each of [`ENCODINGS`], for each `j`; followed by the same with the letter space
/// dropped, for characters followed by whitespace.
static ENCODING_BYTES: [[u8; 128]; 8] = {
    let mut x = [[0; 128]; 8];
    let mut id = 0;
    while id < 64 {
        let qword = ENCODINGS.0[id];
        let bytes = qword.to_le_bytes();
        let mut dropped = bytes;
        if qword != 0 && !is_whitespace(qword) {
            // the vector code relies on non-empty encodings having no NUL byte, and ending with a
            // letter space (note that `&` is encoded as `. ...`, with another space)
            let mut last = 7;
            while bytes[last] == 0 {
                last -= 1;
            }
            assert!(bytes[last] == b' ');
            dropped[last] = 0;
        }
        let mut j = 0;
        while j < 8 {
            x[j][id] = bytes[j];
            x[j][64 + id] = dropped[j];
            j += 1;
        }
        id += 1;
    }
    x
};

/// Permutation of the characters of a block such that, after the transposition by
/// [`transpose`], the qwords of vector `g` are the encodings of characters `8 * g..8 * g + 8`.
static TRANSPOSE_ORDER: [u8; 64] = {
    let mut x = [0; 64];
    let mut p = 0;
    while p < 64 {
        let (lane, pair, member) = (p / 16, (p % 16) / 2, p % 2);
        x[p] = (8 * pair + 2 * lane + member) as u8;
        p += 1;
    }
    x
};

/// Index of the next character for each character.
static NEXT: [u8; 64] = {
    let mut x = [0; 64];
    let mut c = 0;
    while c < 64 {
        x[c] = (c as u8 + 1) % 64;
        c += 1;
    }
    x
};

/// For each of [`ENCODINGS`], 0x80 for whitespace, zero otherwise.
static WHITESPACE_BY_ID: [u8; 64] = {
    let mut x = [0; 64];
    let mut id = 0;
    while id < 64 {
        if is_whitespace(ENCODINGS.0[id]) {
            x[id] = 0x80;
        }
        id += 1;
    }
    x
};

/// Transpose the 8 vectors of bytes `j` of the encodings of characters with in-lane unpacks; the
/// characters must have been permuted according to [`TRANSPOSE_ORDER`].
#[inline]
#[target_feature(enable = "avx512f,avx512bw")]
fn transpose(r: [__m512i; 8]) -> [__m512i; 8] {
    // words: bytes `2 * k` and `2 * k + 1` for `w[2 * k + h]`
    let w: [__m512i; 8] = std::array::from_fn(|i| {
        let (a, b) = (r[i & !1], r[i | 1]);
        if i % 2 == 0 {
            _mm512_unpacklo_epi8(a, b)
        } else {
            _mm512_unpackhi_epi8(a, b)
        }
    });
    // dwords: bytes `4 * m..4 * m + 4` for `d[4 * h + 2 * m + _]`
    let d: [__m512i; 8] = std::array::from_fn(|i| {
        let (h, m) = (i / 4, (i / 2) % 2);
        let (a, b) = (w[4 * m + h], w[4 * m + 2 + h]);
        if i % 2 == 0 {
            _mm512_unpacklo_epi16(a, b)
        } else {
            _mm512_unpackhi_epi16(a, b)
        }
    });
    // qwords
    std::array::from_fn(|i| {
        let (h, hh) = (i / 4, (i / 2) % 2);
        let (a, b) = (d[4 * h + hh], d[4 * h + 2 + hh]);
        if i % 2 == 0 {
            _mm512_unpacklo_epi32(a, b)
        } else {
            _mm512_unpackhi_epi32(a, b)
        }
    })
}

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

/// A block of 64 characters, ready to be written.
struct Block {
    /// Whether the block must be handled by the scalar code.
    scalar: bool,
    /// Whether the first character is whitespace.
    whitespace_first: bool,
    /// Encodings of characters `8 * g..8 * g + 8` in the qwords of `qwords[g]`, with NUL bytes for
    /// padding and dropped letter spaces.
    qwords: [__m512i; 8],
}

/// Compute the encodings of a block of 64 bytes; bytes that are not ASCII are encoded as nothing.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
fn prepare(chars: __m512i) -> Block {
    // SAFETY: `CLASS` has 128 bytes
    let (class_lo, class_hi) = unsafe { (load(CLASS.as_ptr()), load(CLASS.as_ptr().add(64))) };
    let ascii = !_mm512_movepi8_mask(chars);
    let class = _mm512_maskz_permutex2var_epi8(ascii, class_lo, chars, class_hi);
    let non_empty = _mm512_test_epi8_mask(class, _mm512_set1_epi8(NON_EMPTY as i8));
    let whitespace = _mm512_test_epi8_mask(class, _mm512_set1_epi8(WHITESPACE as i8));
    let special = _mm512_test_epi8_mask(class, _mm512_set1_epi8(SPECIAL as i8));

    // SAFETY: the arrays have 64 bytes
    let (ids_lo, ids_hi, order, next, whitespace_by_id) = unsafe {
        (
            load(ENCODINGS.1.as_ptr()),
            load(ENCODINGS.1.as_ptr().add(64)),
            load(TRANSPOSE_ORDER.as_ptr()),
            load(NEXT.as_ptr()),
            load(WHITESPACE_BY_ID.as_ptr()),
        )
    };
    let ids = _mm512_maskz_permutex2var_epi8(ascii, ids_lo, chars, ids_hi);
    // the next character of the last one is in the next block
    let ids_next = _mm512_maskz_permutexvar_epi8(!(1 << 63), next, ids);
    // characters followed by whitespace lose their letter space: use the second half of the tables
    let followed_by_whitespace =
        _mm512_movepi8_mask(_mm512_permutexvar_epi8(ids_next, whitespace_by_id));
    let index = _mm512_mask_add_epi8(ids, followed_by_whitespace, ids, _mm512_set1_epi8(64));
    let index = _mm512_permutexvar_epi8(order, index);

    // byte `j` of the encoding of each character
    let bytes: [__m512i; 8] = std::array::from_fn(|j| {
        // SAFETY: `ENCODING_BYTES[j]` has 128 bytes
        let (lo, hi) = unsafe {
            (
                load(ENCODING_BYTES[j].as_ptr()),
                load(ENCODING_BYTES[j].as_ptr().add(64)),
            )
        };
        _mm512_permutex2var_epi8(lo, index, hi)
    });
    Block {
        // `%` is encoded as more than 8 bytes; when whitespace follows a character encoded as
        // nothing, the letter space to drop is further back
        scalar: special | (whitespace & !(non_empty << 1) & !1) != 0,
        whitespace_first: whitespace & 1 != 0,
        qwords: transpose(bytes),
    }
}

/// Write the encodings of a block at `out + cur`, and return the new value of `cur`; see
/// [`encode_ascii_byte`] for `floor`.
///
/// # Safety
///
/// - the CPU must support the features checked by [`is_available`];
/// - `block.scalar` must be false;
/// - `out` must be valid for writes of `cur + 64 * MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `cur` bytes of `out` must be initialized.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi2,popcnt")]
unsafe fn write(block: &Block, out: *mut u8, mut cur: usize, floor: usize) -> usize {
    debug_assert!(!block.scalar);
    if block.whitespace_first {
        // whitespace replaces the preceding letter space, if any (but not before `floor`)
        // SAFETY: `cur > floor >= 0` is checked first and the caller guarantees the first `cur`
        // bytes are initialized
        if cur > floor && unsafe { *out.add(cur - 1) } == b' ' {
            cur -= 1;
        }
    }
    for &qwords in &block.qwords {
        let keep = _mm512_test_epi8_mask(qwords, qwords);
        let packed = _mm512_maskz_compress_epi8(keep, qwords);
        // SAFETY: we wrote at most 8 bytes for each previous character of the block, so
        // `cur + 64` is within the space the caller guarantees
        unsafe { _mm512_storeu_si512(out.add(cur).cast(), packed) };
        cur += keep.count_ones() as usize;
    }
    cur
}

/// Encode a block of 64 bytes at `out + cur`, and return the new value of `cur`; bytes that are not
/// ASCII are encoded as nothing. See [`encode_ascii_byte`] for `floor`.
///
/// # Safety
///
/// - the CPU must support the features checked by [`is_available`];
/// - `out` must be valid for writes of `cur + 64 * MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `cur` bytes of `out` must be initialized.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,popcnt")]
unsafe fn finish(block: &Block, bytes: &[u8], out: *mut u8, mut cur: usize, floor: usize) -> usize {
    if block.scalar {
        for &c in bytes {
            // SAFETY: forwarded from the caller
            cur = unsafe { encode_ascii_byte(c, out, cur, floor) };
        }
        cur
    } else {
        // SAFETY: forwarded from the caller
        unsafe { write(block, out, cur, floor) }
    }
}

/// Encode a block of 64 bytes, the first `len` of which are ASCII, at `out + cur`, and return the
/// new value of `cur`; see [`encode_ascii_byte`] for `floor`.
///
/// # Safety
///
/// - the CPU must support the features checked by [`is_available`];
/// - `len <= 64`, and the first `len` bytes of `block` must be ASCII;
/// - `out` must be valid for writes of `cur + 64 * MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `cur` bytes of `out` must be initialized.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,bmi2,popcnt")]
pub(crate) unsafe fn encode_ascii_prefix(
    block: &[u8; 64],
    len: usize,
    out: *mut u8,
    cur: usize,
    floor: usize,
) -> usize {
    // SAFETY: `block` has 64 bytes
    let chars = unsafe { load(block.as_ptr()) };
    // NUL bytes are encoded as nothing, and are not whitespace
    let chars = _mm512_maskz_mov_epi8(_bzhi_u64(u64::MAX, len as u32), chars);
    // SAFETY: forwarded from the caller
    unsafe { finish(&prepare(chars), &block[..len], out, cur, floor) }
}

/// Encode 64-byte blocks of input with AVX-512, and return the new value of `cur`; the remaining
/// `input.len() % 64` bytes are not processed. See [`encode_ascii_byte`] for `floor`.
///
/// # Safety
///
/// - the CPU must support the features checked by [`is_available`];
/// - `out` must be valid for writes of `cur + input.len() * MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `cur` bytes of `out` must be initialized.
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,bmi2,popcnt")]
pub(super) unsafe fn encode_blocks(
    input: &[u8],
    out: *mut u8,
    mut cur: usize,
    floor: usize,
) -> usize {
    // process two blocks at a time, to give more independent work to the CPU
    let (pairs, rest) = input.as_chunks::<128>();
    for pair in pairs {
        // SAFETY: `pair` has 128 bytes
        let (a, b) = unsafe { (load(pair.as_ptr()), load(pair.as_ptr().add(64))) };
        let (a, b) = (prepare(a), prepare(b));
        // SAFETY: the caller guarantees room for `MAX_OUTPUT_PER_INPUT_BYTE` bytes per input byte
        cur = unsafe { finish(&a, &pair[..64], out, cur, floor) };
        // SAFETY: see above
        cur = unsafe { finish(&b, &pair[64..], out, cur, floor) };
    }
    let (blocks, _) = rest.as_chunks::<64>();
    for block in blocks {
        // SAFETY: `block` has 64 bytes
        let block_vector = prepare(unsafe { load(block.as_ptr()) });
        // SAFETY: see above
        cur = unsafe { finish(&block_vector, block, out, cur, floor) };
    }
    cur
}
