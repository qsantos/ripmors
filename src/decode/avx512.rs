//! Decoding with AVX-512 (VBMI, VBMI2 and GFNI).
//!
//! The input is processed in blocks of 64 bytes, with no overlap. The character decoded from the
//! Morse sequence ending with a separator at `b + k + 1` is put in lane `k` of the block at `b`
//! (the lane of the last element of the sequence, which produces no output), and each separator
//! in its own lane, so that each lane produces at most one character or one separator, in order.
//! For each block:
//!
//! 1. we classify bytes with a lookup (`vpermb`), which yields the output of separators (nothing
//!    for letter spaces, a space for word spaces, the separator itself otherwise), and flags
//!    invalid bytes; from it, we transpose two bitplanes with a single `vgf2p8affineqb`: the
//!    "code" bits (1 for `.`, 0 for `-`) and the separators (invalid bytes are not separators);
//! 2. with `vpermt2b` and `vpmultishiftqb`, we extract for each lane `k` the bits of the bytes
//!    `k + 1, k, ..., k - 6` (in this order), from the bitplanes of the block and of the previous
//!    and next ones (which is why the work is pipelined over three blocks);
//! 3. the lanes followed by a separator are characters; for those, the first separator before
//!    the lane (the lowest set bit of the window of separators, after the first one) marks the
//!    start of the sequence, and we keep the code bits up to it with a subtraction and
//!    `vpternlogd`; this is an index (the elements in reverse order, with a leading one) in
//!    lookup tables with the bytes of the UTF-8 encodings of the decoded characters;
//! 4. when the decoded characters are ASCII, we merge them with the separators and pack them with
//!    a single `vpcompressb`; otherwise, we interleave the two (or three) bytes of each lane and
//!    pack them with two (or three) `vpcompressb`.
//!
//! The first byte of each lane (before interleaving) has its most significant bit set for
//! non-ASCII characters, and is at least 0xf0 for invalid bytes, sequences of 7 elements or
//! more (whose start is not in the window), and characters of 4 bytes (only possible with a custom
//! `char_decode`). Invalid bytes at the end of a block are also flagged for the next block, whose
//! first sequences may include them. Since moving mask registers to general-purpose registers is
//! slow, we check these flags once per batch of blocks, with the maximum of these bytes (the
//! start of the current Morse sequence is likewise only computed when the scalar code needs it).
//! When a flag is set, we
//! decode the blocks of the batch again with [`decode_block_slow`], which chooses the output path
//! of each block, and uses the scalar code for blocks that need it.
//!
//! The blocks of a group of batches use the same output path: one byte per lane when there were
//! few non-ASCII characters in the previous group, two or three bytes otherwise; this avoids
//! mispredicting a branch for each block.

use std::arch::x86_64::*;

use super::{
    DecodeTable, MAX_OUTPUT_PER_INPUT_BYTE, decode_byte_checked, decode_separators, is_separator,
    separator_mask, separator_output, sequence_index_fast,
};

/// Number of blocks decoded before checking whether they need another path.
const BATCH: usize = 4;

/// Number of bytes in a batch.
const BATCH_LEN: usize = 64 * BATCH;

/// Number of batches in a group, which use the same output path, and between two reservations of
/// output space.
const BATCHES_PER_GROUP: usize = 16;

/// Number of bytes read after a block: we compute the bitplanes two blocks ahead.
const LOOK_AHEAD: usize = 128;

/// Minimum input length for [`decode_blocks`] to process at least one batch, after the 8 bytes
/// it looks back.
pub(super) const MIN_INPUT_LEN: usize = 8 + BATCH_LEN + LOOK_AHEAD;

/// Output space needed for each block: the scalar fallback for 64 separators and an extra
/// character, or up to three 64-byte stores, each at most 64 bytes after the previous one.
const OUTPUT_PER_BLOCK: usize = 65 * MAX_OUTPUT_PER_INPUT_BYTE;

/// A group uses the paths for non-ASCII characters when more than one in `DENSE_RATIO` blocks of
/// the previous group had non-ASCII characters.
const DENSE_RATIO: usize = 8;

/// The CPU features used by this module.
pub(super) fn is_available() -> bool {
    // Miri does not support most AVX-512 instructions
    !cfg!(miri)
        && is_x86_feature_detected!("avx512f")
        && is_x86_feature_detected!("avx512bw")
        && is_x86_feature_detected!("avx512vbmi")
        && is_x86_feature_detected!("avx512vbmi2")
        && is_x86_feature_detected!("gfni")
        && is_x86_feature_detected!("popcnt")
}

/// Whether `c` is a Morse element or a separator.
fn is_valid(c: u8) -> bool {
    c == b'.' || c == b'-' || is_separator(c)
}

/// For each byte below 64 (the other bytes, which are all invalid, are clamped to 63), the
/// output of separators (see the module documentation), 0 for elements, and a value of at least
/// 0xfc for invalid bytes. Then, the bits 0 and 1 of `x ^ c`, where `x` is this value and `c` the
/// byte, are 1 or 2 for elements and invalid bytes, and 0 or 3 for separators; the bit 0 is 0 for
/// `.`, 1 for `-`.
static SEPARATOR_OUTPUT: [u8; 64] = {
    let mut x = [0; 64];
    let mut c = 0;
    while c < 64 {
        let byte = c as u8;
        x[c] = match byte {
            b' ' | b'.' | b'-' => 0,
            b'/' => b' ',
            b'\t' | b'\n' | b'\r' => byte,
            _ => 0xfc | ((byte ^ 1) & 3),
        };
        c += 1;
    }
    x
};

/// Operand for `vgf2p8affineqb` (applied with the classified bytes as the matrix, and with all
/// the bits complemented) yielding the bitplanes of each 64-bit lane: byte 0 is the code (1 for
/// `.`) and byte 1 the separators, with bit `j` for byte `7 - j` of the lane.
static SELECT: [u8; 64] = {
    let mut x = [0; 64];
    let mut q = 0;
    while q < 8 {
        x[8 * q] = 1;
        x[8 * q + 1] = 3;
        q += 1;
    }
    x
};

/// Index for `vpermt2b` gathering the bitplanes (the bytes 0 and 1 of each 64-bit lane after
/// [`SELECT`]): each 64-bit lane `q` gets the code bits of the lanes `q + 1`, `q` and `q - 1` in
/// its bytes 0, 1 and 2, and their separator bits in its bytes 4, 5 and 6, so that bit `b` (and
/// `32 + b`) is for the byte `8 * q + 15 - b` of the block. The second table has the bitplanes of
/// the next block in its first lane, and those of the previous block in its last lane.
static GATHER: [u8; 64] = {
    let mut x = [0; 64];
    let mut q = 0;
    while q < 8 {
        let mut pos = 0;
        while pos < 8 {
            let p = (pos / 4) as u8;
            x[8 * q + pos] = match pos % 4 {
                0 if q == 7 => 64 + p,
                0 => 8 * (q as u8 + 1) + p,
                2 if q == 0 => 64 + 56 + p,
                2 => 8 * (q as u8 - 1) + p,
                _ => 8 * q as u8 + p,
            };
            pos += 1;
        }
        q += 1;
    }
    x
};

/// Control for `vpmultishiftqb` extracting, for lane `k`, the bits of the bytes `k + 1` (bit 0)
/// down to `k - 6` (bit 7), from the code bits (`offset == 0`) or from the separator bits
/// (`offset == 32`), given the layout of [`GATHER`].
const fn window(offset: u8) -> [u8; 64] {
    let mut x = [0; 64];
    let mut k = 0;
    while k < 64 {
        x[k] = offset + 14 - (k % 8) as u8;
        k += 1;
    }
    x
}

static WINDOW_CODE: [u8; 64] = window(0);
static WINDOW_SEPARATORS: [u8; 64] = window(32);

/// Index for `vpermt2b` interleaving bytes `32 * half..32 * half + 32` of two vectors.
const fn interleave2(half: u8) -> [u8; 64] {
    let mut x = [0; 64];
    let mut k = 0;
    while k < 32 {
        x[2 * k] = 32 * half + k as u8;
        x[2 * k + 1] = 64 + 32 * half + k as u8;
        k += 1;
    }
    x
}

/// Indices for two `vpermt2b` taking the bytes `64 * part..64 * part + 64` of the interleaving of
/// three vectors: the first takes the bytes from the first two vectors, the second keeps them and
/// takes the bytes from the third.
const fn interleave3(part: usize) -> [[u8; 64]; 2] {
    let mut x = [[0; 64]; 2];
    let mut k = 0;
    while k < 64 {
        let p = 64 * part + k;
        x[0][k] = (p / 3 + 64 * (p % 3 == 1) as usize) as u8;
        x[1][k] = if p % 3 == 2 { 64 + p / 3 } else { k } as u8;
        k += 1;
    }
    x
}

static INTERLEAVE2: [[u8; 64]; 2] = [interleave2(0), interleave2(1)];
static INTERLEAVE3: [[[u8; 64]; 2]; 3] = [interleave3(0), interleave3(1), interleave3(2)];

/// Mask of the last 8 lanes.
static LAST_LANES: [u8; 64] = {
    let mut x = [0; 64];
    let mut k = 56;
    while k < 64 {
        x[k] = 0xff;
        k += 1;
    }
    x
};

/// Matrix for `vgf2p8affineqb` copying bit 0 to all the bits.
const BROADCAST_BIT_0: i64 = 0x0101_0101_0101_0101;

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

/// Vectors used by all the blocks.
struct Constants {
    separator_output: __m512i,
    /// The bytes of the UTF-8 encodings of the decoded characters, see [`DecodeTable::window`].
    bytes: [[__m512i; 2]; 3],
    select: __m512i,
    gather: __m512i,
    window_code: __m512i,
    window_separators: __m512i,
    last_lanes: __m512i,
    interleave2: [__m512i; 2],
    interleave3: [[__m512i; 2]; 3],
}

impl Constants {
    #[inline]
    #[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
    fn new(table: &DecodeTable) -> Self {
        let load64 = |x: &[u8; 64]| {
            // SAFETY: `x` has 64 bytes
            unsafe { load(x.as_ptr()) }
        };
        let load128 = |x: &[u8; 128]| {
            // SAFETY: `x` has 128 bytes
            unsafe { [load(x.as_ptr()), load(x.as_ptr().add(64))] }
        };
        Constants {
            separator_output: load64(&SEPARATOR_OUTPUT),
            bytes: table.window.each_ref().map(load128),
            select: load64(&SELECT),
            gather: load64(&GATHER),
            window_code: load64(&WINDOW_CODE),
            window_separators: load64(&WINDOW_SEPARATORS),
            last_lanes: load64(&LAST_LANES),
            interleave2: INTERLEAVE2.each_ref().map(load64),
            interleave3: INTERLEAVE3.each_ref().map(|x| x.each_ref().map(load64)),
        }
    }
}

/// Look up each byte of `index` in a 128-entry table (ignoring the most significant bit).
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
fn lookup(lut: &[__m512i; 2], index: __m512i) -> __m512i {
    _mm512_permutex2var_epi8(lut[0], index, lut[1])
}

/// The first stage of a block: its bitplanes, and the output of its separators.
#[derive(Clone, Copy)]
struct Stage1 {
    planes: __m512i,
    separator_output: __m512i,
}

#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
fn stage1(v: __m512i, k: &Constants) -> Stage1 {
    let low = _mm512_min_epu8(v, _mm512_set1_epi8(63));
    let separator_output = _mm512_permutexvar_epi8(low, k.separator_output);
    let classified = _mm512_xor_si512(separator_output, low);
    Stage1 {
        planes: _mm512_gf2p8affine_epi64_epi8::<0xff>(k.select, classified),
        separator_output,
    }
}

/// The bitplanes of the 8 bytes before `p`, in the last 64-bit lane (see [`GATHER`]).
///
/// # Safety
///
/// `p - 8` must be valid for reads of 64 bytes.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn previous_planes(p: *const u8, k: &Constants) -> __m512i {
    // SAFETY: forwarded from the caller
    let planes = stage1(unsafe { load(p.sub(8)) }, k).planes;
    _mm512_alignr_epi64::<1>(planes, planes)
}

/// The windows of a block: for each lane `k`, the code bits and the separator bits of the bytes
/// `k + 1, k, ..., k - 6`.
#[derive(Clone, Copy)]
struct Windows {
    code: __m512i,
    separators: __m512i,
}

#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
fn stage2(prev: __m512i, planes: __m512i, next: __m512i, k: &Constants) -> Windows {
    let other = _mm512_mask_blend_epi8(0xff, prev, next);
    let planes = _mm512_permutex2var_epi8(planes, k.gather, other);
    Windows {
        code: _mm512_multishift_epi64_epi8(k.window_code, planes),
        separators: _mm512_multishift_epi64_epi8(k.window_separators, planes),
    }
}

/// The index of the decoded character of each lane in [`DecodeTable::window`], and the lanes
/// followed by a separator (0xff), which are the only ones outputting a character.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
fn index(w: Windows) -> (__m512i, __m512i) {
    // the bits up to the lowest set bit of the separators after bit 0 (excluded), and that
    // bit, are set in `w.separators ^ m`; if there is no such bit, `m` and the index are 0
    let m = _mm512_subs_epu8(w.separators, _mm512_set1_epi8(2));
    // `(code & !separators & m) | (separators & !m)`: the code bits strictly between bit 0
    // and the marker, and the marker; then, remove bit 0 (`vpermi2b` ignores bit 7, which
    // gets a bit of the next byte)
    let index = _mm512_srli_epi16::<1>(_mm512_ternarylogic_epi32::<0x64>(w.code, w.separators, m));
    let chars =
        _mm512_gf2p8affine_epi64_epi8::<0>(w.separators, _mm512_set1_epi64(BROADCAST_BIT_0));
    (index, chars)
}

/// The first byte of the output of each lane: the first byte of the decoded character, or the
/// output of the separator.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
fn first_bytes(
    index: __m512i,
    chars: __m512i,
    separator_output: __m512i,
    k: &Constants,
) -> __m512i {
    // `(decoded & chars) | separator_output`
    _mm512_ternarylogic_epi32::<0xea>(lookup(&k.bytes[0], index), chars, separator_output)
}

/// The second and third bytes of the decoded character of each lane, or 0.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
fn next_bytes(index: __m512i, chars: __m512i, k: &Constants) -> [__m512i; 2] {
    // the entry 0 is empty in these tables
    let index = _mm512_and_si512(index, chars);
    [lookup(&k.bytes[1], index), lookup(&k.bytes[2], index)]
}

/// Write the non-zero bytes of `v` to `out + cur`; return the new `cur`.
///
/// # Safety
///
/// `out` must be valid for writes of `cur + 64` bytes.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn store(v: __m512i, out: *mut u8, cur: usize) -> usize {
    let keep = _mm512_test_epi8_mask(v, v);
    // NOTE: we use register compression and a full store, since compressing to memory is very
    // slow on Zen 4
    // SAFETY: forwarded from the caller
    unsafe { _mm512_storeu_si512(out.add(cur).cast(), _mm512_maskz_compress_epi8(keep, v)) };
    cur + keep.count_ones() as usize
}

/// Write the non-zero bytes of the two bytes of each lane; return the new `cur`.
///
/// # Safety
///
/// `out` must be valid for writes of `cur + 128` bytes.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn store2(b0: __m512i, b1: __m512i, k: &Constants, out: *mut u8, mut cur: usize) -> usize {
    for index in k.interleave2 {
        // SAFETY: forwarded from the caller, and each store writes at most 64 bytes
        cur = unsafe { store(_mm512_permutex2var_epi8(b0, index, b1), out, cur) };
    }
    cur
}

/// Write the non-zero bytes of the three bytes of each lane; return the new `cur`.
///
/// # Safety
///
/// `out` must be valid for writes of `cur + 192` bytes.
#[inline]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn store3(
    b0: __m512i,
    b1: __m512i,
    b2: __m512i,
    k: &Constants,
    out: *mut u8,
    mut cur: usize,
) -> usize {
    for [index01, index2] in k.interleave3 {
        let part = _mm512_permutex2var_epi8(_mm512_permutex2var_epi8(b0, index01, b1), index2, b2);
        // SAFETY: forwarded from the caller, and each store writes at most 64 bytes
        cur = unsafe { store(part, out, cur) };
    }
    cur
}

/// Start of the current Morse sequence at `i`, knowing that it is at least `floor`.
fn sequence_start(input: &[u8], i: usize, floor: usize) -> usize {
    let mut j = i;
    while j > floor && !is_separator(input[j - 1]) {
        j -= 1;
    }
    j
}

/// Decode the block at `b` with scalar code; return the new values of `cur` and `chunk_start`.
///
/// # Safety
///
/// - `chunk_start <= b` must be the start of the current Morse sequence (before a separator at
///   `b`, whose character, if any, was already output), and `b + 64 + 8 <= input.len()`;
/// - `out` must be valid for writes of `cur + OUTPUT_PER_BLOCK` bytes.
unsafe fn decode_block_scalar(
    input: &[u8],
    b: usize,
    mut chunk_start: usize,
    table: &DecodeTable,
    out: *mut u8,
    mut cur: usize,
) -> (usize, usize) {
    let block: &[u8; 64] = input[b..b + 64].try_into().unwrap();
    let separators = separator_mask(block);
    if separators & 1 != 0 {
        // the character of a separator at the start of the block is output with the previous
        // block
        let (separator, keep) = separator_output(block[0]);
        // SAFETY: `out` is valid for writes of `cur + 1` bytes
        unsafe { out.add(cur).write(separator) };
        cur += keep as usize;
        chunk_start = b + 1;
    }
    // SAFETY: the bits of `separators & !1` are separators after `chunk_start`,
    // `b + 64 + 7 <= input.len()`, and there is room for `64 * MAX_OUTPUT_PER_INPUT_BYTE` bytes
    (cur, chunk_start) =
        unsafe { decode_separators(input, b, separators & !1, chunk_start, table, out, cur) };
    // the character of a separator right after the block
    let j = b + 64;
    if is_separator(input[j]) {
        let eight_bytes: &[u8; 8] = input[chunk_start..chunk_start + 8].try_into().unwrap();
        let index = sequence_index_fast(eight_bytes, j - chunk_start);
        // SAFETY: `index < 512` (see `decode_separator`), and there is room for 4 more bytes
        unsafe {
            out.add(cur)
                .cast::<[u8; 4]>()
                .write_unaligned(*table.utf8.get_unchecked(index));
            cur += *table.len.get_unchecked(index) as usize;
        }
        chunk_start = j + 1;
    }
    (cur, chunk_start)
}

/// The output path taken by [`decode_block_slow`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Path {
    Ascii,
    TwoBytes,
    ThreeBytes,
    Scalar,
}

/// Decode the block at `b`, choosing the output path from its content; return the new `cur`,
/// and the path taken.
///
/// NOTE: this is out of line to keep the registers of the main loop free.
///
/// # Safety
///
/// - `8 <= b`, `b + LOOK_AHEAD + 8 <= input.len()`, and the bytes before `b` must have been
///   decoded, as well as the character of a separator at `b` (if any);
/// - `floor` must be at most the start of the current Morse sequence; it is updated when
///   using the scalar path;
/// - `out` must be valid for writes of `cur + OUTPUT_PER_BLOCK` bytes.
#[inline(never)]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn decode_block_slow(
    input: &[u8],
    b: usize,
    floor: &mut usize,
    table: &DecodeTable,
    k: &Constants,
    out: *mut u8,
    mut cur: usize,
) -> (usize, Path) {
    let ptr = input.as_ptr();
    // SAFETY: `8 <= b` and `b + 128 <= input.len()`
    let (prev, s1, next) = unsafe {
        (
            previous_planes(ptr.add(b), k),
            stage1(load(ptr.add(b)), k),
            stage1(load(ptr.add(b + 64)), k),
        )
    };
    let (index, chars) = index(stage2(prev, s1.planes, next.planes, k));
    let b0 = first_bytes(index, chars, s1.separator_output, k);
    let at_least = |x: u8| _mm512_cmpge_epu8_mask(b0, _mm512_set1_epi8(x as i8)) != 0;
    // the Morse sequences ending in the block may start with the 7 previous bytes
    let context_valid = input[b - 7..b].iter().all(|&c| is_valid(c));
    let path = if !context_valid || at_least(0xf0) {
        Path::Scalar
    } else if !at_least(0x80) {
        Path::Ascii
    } else if !at_least(0xe0) {
        Path::TwoBytes
    } else {
        Path::ThreeBytes
    };
    // SAFETY: we have room for `OUTPUT_PER_BLOCK >= 192` bytes
    unsafe {
        match path {
            Path::Ascii => cur = store(b0, out, cur),
            Path::TwoBytes => cur = store2(b0, next_bytes(index, chars, k)[0], k, out, cur),
            Path::ThreeBytes => {
                let [b1, b2] = next_bytes(index, chars, k);
                cur = store3(b0, b1, b2, k, out, cur);
            }
            Path::Scalar => {
                let chunk_start = sequence_start(input, b, *floor);
                // SAFETY: `chunk_start` is the start of the current Morse sequence,
                // `b + 72 <= input.len()`, and we have room for `OUTPUT_PER_BLOCK` bytes
                (cur, *floor) = decode_block_scalar(input, b, chunk_start, table, out, cur);
            }
        }
    }
    (cur, path)
}

/// The state of the pipeline between blocks.
#[derive(Clone, Copy)]
struct Pipeline {
    /// The first stage of the current block
    current: Stage1,
    /// The first stage of the next block
    next: Stage1,
    /// The windows of the current block
    windows: Windows,
    /// The first bytes of the previous block (see [`first_bytes`]), whose last lanes flag
    /// invalid bytes before the current block
    previous_first_bytes: __m512i,
}

/// Statistics of a group of blocks, to choose the path of the next group.
#[derive(Default)]
struct Statistics {
    /// Number of blocks decoded by [`decode_block_slow`] with two bytes per lane
    two_bytes: usize,
    /// Number of blocks decoded by [`decode_block_slow`] with three bytes per lane
    three_bytes: usize,
    /// With the paths for non-ASCII characters, the number of batches with non-ASCII characters
    non_ascii_batches: usize,
    /// With the path for three bytes per lane, whether there were no characters of three bytes
    no_three_bytes: bool,
}

/// Context of [`decode_group`].
struct Context<'a> {
    input: &'a [u8],
    table: &'a DecodeTable,
    k: Constants,
    out: *mut u8,
}

/// Decode `batches` batches of blocks from `i`, with `BYTES` output bytes per lane (unless
/// [`decode_block_slow`] takes another path); return the new values of `i` and `cur`.
///
/// # Safety
///
/// - `8 <= i`, `i + batches * BATCH_LEN + LOOK_AHEAD <= input.len()`, all the bytes before `i`
///   must have been decoded, as well as the character of a separator at `i` (if any), and
///   `pipeline` must be the state for the block at `i`;
/// - `floor` must be at most the start of the current Morse sequence;
/// - `out` must be valid for writes of `cur + batches * BATCH * OUTPUT_PER_BLOCK` bytes.
#[inline(never)]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn decode_group<const BYTES: usize>(
    ctx: &Context,
    mut i: usize,
    batches: usize,
    pipeline: &mut Pipeline,
    floor: &mut usize,
    mut cur: usize,
    stats: &mut Statistics,
) -> (usize, usize) {
    let Context {
        input,
        table,
        ref k,
        out,
    } = *ctx;
    let ptr = input.as_ptr();
    let Pipeline {
        mut current,
        mut next,
        mut windows,
        mut previous_first_bytes,
    } = *pipeline;
    // the first bytes that need another path
    let threshold = _mm512_set1_epi8(match BYTES {
        1 => 0x80,
        2 => 0xe0,
        _ => 0xf0,
    } as u8 as i8);
    // the maximum of the first bytes of the group
    let mut group_max = _mm512_setzero_si512();
    for _ in 0..batches {
        let batch_start = i;
        let batch_cur = cur;
        let batch_previous_first_bytes = previous_first_bytes;
        // invalid bytes at the end of the previous block are flagged in its last lanes
        let mut max = _mm512_and_si512(previous_first_bytes, k.last_lanes);
        let mut batch = [_mm512_setzero_si512(); BATCH];
        for first_bytes_slot in &mut batch {
            // SAFETY: `i + 192 <= input.len()`
            let next_next = stage1(unsafe { load(ptr.add(i + 128)) }, k);
            let next_windows = stage2(current.planes, next.planes, next_next.planes, k);

            let (index, chars) = index(windows);
            let b0 = first_bytes(index, chars, current.separator_output, k);
            max = _mm512_max_epu8(max, b0);
            // SAFETY: we have room for `OUTPUT_PER_BLOCK >= 192` bytes for each block
            unsafe {
                match BYTES {
                    1 => cur = store(b0, out, cur),
                    2 => cur = store2(b0, next_bytes(index, chars, k)[0], k, out, cur),
                    _ => {
                        let [b1, b2] = next_bytes(index, chars, k);
                        cur = store3(b0, b1, b2, k, out, cur);
                    }
                }
            }
            *first_bytes_slot = b0;
            previous_first_bytes = b0;

            current = next;
            next = next_next;
            windows = next_windows;
            i += 64;
        }
        if BYTES > 1 {
            stats.non_ascii_batches += (_mm512_movepi8_mask(max) != 0) as usize;
        }
        if BYTES == 3 {
            group_max = _mm512_max_epu8(group_max, max);
        }

        // NOTE: checking once per batch is cheaper, since mask registers are slow
        if _mm512_cmpge_epu8_mask(max, threshold) != 0 {
            cur = batch_cur;
            let mut previous = batch_previous_first_bytes;
            for (n, b0) in batch.into_iter().enumerate() {
                let b = batch_start + 64 * n;
                // `b0 | (previous & last_lanes)`
                let flags = _mm512_ternarylogic_epi32::<0xf8>(b0, previous, k.last_lanes);
                previous = b0;
                if BYTES == 1 && _mm512_movepi8_mask(flags) == 0 {
                    // SAFETY: we have room for `OUTPUT_PER_BLOCK >= 64` bytes for each block
                    cur = unsafe { store(b0, out, cur) };
                    continue;
                }
                let path;
                // SAFETY: `8 <= b`, `b + LOOK_AHEAD + 8 <= input.len()`, the bytes before
                // `b` were decoded, and we have room for `OUTPUT_PER_BLOCK` bytes
                (cur, path) = unsafe { decode_block_slow(input, b, floor, table, k, out, cur) };
                stats.two_bytes += (path == Path::TwoBytes) as usize;
                stats.three_bytes += (path == Path::ThreeBytes) as usize;
            }
        }
    }
    if BYTES == 3 {
        stats.no_three_bytes =
            _mm512_cmpge_epu8_mask(group_max, _mm512_set1_epi8(0xe0u8 as i8)) == 0;
    }
    *pipeline = Pipeline {
        current,
        next,
        windows,
        previous_first_bytes,
    };
    (i, cur)
}

/// Decode blocks of input with AVX-512, starting at `i`, as long as there is enough input; return
/// the position of the first byte not processed, and the start of the current Morse sequence.
///
/// # Safety
///
/// - the CPU must support the features checked by [`is_available`];
/// - `8 <= i`, all the bytes before `i` must have been decoded, and `chunk_start` must be the
///   start of the current Morse sequence.
pub(super) unsafe fn decode_blocks(
    input: &[u8],
    mut i: usize,
    mut chunk_start: usize,
    table: &DecodeTable,
    output_buf: &mut Vec<u8>,
) -> (usize, usize) {
    debug_assert!(i >= 8);
    // a separator at the start of a block is output without its character, which is output with
    // the previous block
    while i < input.len() && is_separator(input[i]) {
        decode_byte_checked(input, i, &mut chunk_start, table, output_buf);
        i += 1;
    }
    if i + BATCH_LEN + LOOK_AHEAD > input.len() {
        return (i, chunk_start);
    }
    // SAFETY: forwarded from the caller, and we checked the length of the input
    unsafe { decode_blocks_impl(input, i, chunk_start, table, output_buf) }
}

/// See [`decode_blocks`]; `input[i]` must not be a separator, and
/// `i + BATCH_LEN + LOOK_AHEAD <= input.len()`.
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2,gfni,popcnt")]
unsafe fn decode_blocks_impl(
    input: &[u8],
    mut i: usize,
    chunk_start: usize,
    table: &DecodeTable,
    output_buf: &mut Vec<u8>,
) -> (usize, usize) {
    let ptr = input.as_ptr();
    let k = Constants::new(table);
    // SAFETY: `8 <= i` and `i + 128 <= input.len()`
    let mut pipeline = unsafe {
        let previous = load(ptr.add(i - 8));
        let previous_separators = stage1(previous, &k).separator_output;
        let current = stage1(load(ptr.add(i)), &k);
        let next = stage1(load(ptr.add(i + 64)), &k);
        Pipeline {
            current,
            next,
            windows: stage2(
                previous_planes(ptr.add(i), &k),
                current.planes,
                next.planes,
                &k,
            ),
            // flag invalid bytes before the block
            previous_first_bytes: _mm512_alignr_epi64::<1>(
                previous_separators,
                previous_separators,
            ),
        }
    };
    let mut ctx = Context {
        input,
        table,
        k,
        out: output_buf.as_mut_ptr(),
    };
    // lower bound for the start of the current Morse sequence
    let mut floor = chunk_start;
    // number of bytes of the UTF-8 encodings of the characters of a group
    let mut bytes = 1;
    while i + BATCH_LEN + LOOK_AHEAD <= input.len() {
        let batches =
            ((input.len() - i - BATCH_LEN - LOOK_AHEAD) / BATCH_LEN + 1).min(BATCHES_PER_GROUP);
        output_buf.reserve(batches * BATCH * OUTPUT_PER_BLOCK);
        ctx.out = output_buf.as_mut_ptr();
        let cur = output_buf.len();
        let mut stats = Statistics::default();
        let p = &mut pipeline;
        let f = &mut floor;
        // SAFETY: forwarded from the caller; we checked the length of the input and reserved
        // the output space
        let (new_i, cur) = unsafe {
            match bytes {
                1 => decode_group::<1>(&ctx, i, batches, p, f, cur, &mut stats),
                2 => decode_group::<2>(&ctx, i, batches, p, f, cur, &mut stats),
                _ => decode_group::<3>(&ctx, i, batches, p, f, cur, &mut stats),
            }
        };
        i = new_i;
        // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only
        // increase `cur` after writing to `output_buf`
        unsafe { output_buf.set_len(cur) };

        let blocks = batches * BATCH;
        let dense = |n: usize| n * DENSE_RATIO > blocks;
        bytes = match bytes {
            1 if dense(stats.two_bytes + stats.three_bytes) => {
                if dense(stats.three_bytes) {
                    3
                } else {
                    2
                }
            }
            1 => 1,
            _ if !dense(stats.non_ascii_batches * BATCH) => 1,
            2 if dense(stats.three_bytes) => 3,
            3 if stats.no_three_bytes => 2,
            b => b,
        };
    }
    let c = input[i];
    if is_separator(c) {
        // the character of the separator was already output
        let (separator, keep) = separator_output(c);
        if keep {
            output_buf.push(separator);
        }
        i += 1;
        (i, i)
    } else {
        (i, sequence_start(input, i, floor))
    }
}
