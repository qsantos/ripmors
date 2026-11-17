use crate::encode_ascii_mapping::ASCII_TO_QWORD;

/// Maximum number of bytes written to the output for any input byte (`%` is encoded as 18 bytes).
pub(crate) const MAX_OUTPUT_PER_INPUT_BYTE: usize = 18;

/// Number of input bytes processed between two reservations of output space; this keeps the
/// reservation proportional to the actual output instead of the worst case for the whole input.
pub(crate) const ENCODE_CHUNK: usize = 1 << 12;

/// Value of [`ENCODE_LEN`] for characters that need special handling.
const SPECIAL: u8 = 0xff;

/// Qwords of [`ASCII_TO_QWORD`], as a separate table so that indexing needs no extra arithmetic.
pub(crate) static ENCODE_QWORD: [u64; 256] = {
    let mut table = [0; 256];
    let mut c = 0;
    while c < 256 {
        table[c] = ASCII_TO_QWORD[c].0;
        c += 1;
    }
    table
};

/// Lengths of [`ASCII_TO_QWORD`], except for characters that cannot be handled by writing the
/// qword and advancing by the length, which are set to [`SPECIAL`]: whitespace (which replaces a
/// preceding letter space) and characters encoded as more than 8 bytes. This lets the common path
/// use a single, well-predicted branch.
pub(crate) static ENCODE_LEN: [u8; 256] = {
    let mut table = [0; 256];
    let mut c = 0;
    while c < 256 {
        let len = ASCII_TO_QWORD[c].1;
        table[c] = if c == b'\t' as usize || c == b'\n' as usize || c == b'\r' as usize || len > 8 {
            SPECIAL
        } else {
            len as u8
        };
        c += 1;
    }
    table
};

/// Encode a single ASCII byte at `out + cur`, and return the new value of `cur`.
///
/// Whitespace replaces a preceding letter space, but only after position `floor`, which protects
/// output that does not belong to the current encoding.
///
/// # Safety
///
/// - `out` must be valid for writes of `cur + MAX_OUTPUT_PER_INPUT_BYTE` bytes;
/// - the first `cur` bytes of `out` must be initialized.
#[inline(always)]
pub(crate) unsafe fn encode_ascii_byte(c: u8, out: *mut u8, cur: usize, floor: usize) -> usize {
    let len = ENCODE_LEN[c as usize];
    if len != SPECIAL {
        // NOTE: unknown characters have length zero; writing their (zero) qword is harmless since
        // it will be overwritten or truncated
        // SAFETY: `len <= 8` and the caller guarantees there is room for 18 bytes at `out + cur`
        unsafe {
            out.add(cur)
                .cast::<u64>()
                .write_unaligned(ENCODE_QWORD[c as usize])
        };
        cur + len as usize
    } else {
        // SAFETY: forwarded from the caller
        unsafe { encode_ascii_special(c, out, cur, floor) }
    }
}

/// Slow path of [`encode_ascii_byte`].
///
/// # Safety
///
/// See [`encode_ascii_byte`].
#[cold]
#[inline(never)]
unsafe fn encode_ascii_special(c: u8, out: *mut u8, mut cur: usize, floor: usize) -> usize {
    if c == b'%' {
        // handle only ASCII character encoded as more than 7 elements + space
        // SAFETY: the caller guarantees there is room for 18 bytes at `out + cur`
        unsafe { std::slice::from_raw_parts_mut(out.add(cur), 18) }
            .copy_from_slice(b"----- -..-. ----- ");
        cur + 18
    } else {
        // whitespace replaces the preceding letter space, if any
        // SAFETY: `cur > floor >= 0` is checked first and the caller guarantees the first `cur`
        // bytes are initialized
        if cur > floor && unsafe { *out.add(cur - 1) } == b' ' {
            cur -= 1;
        }
        // SAFETY: the caller guarantees there is room for 18 bytes at `out + cur`
        unsafe { out.add(cur).write(c) };
        cur + 1
    }
}

#[cfg(target_arch = "x86_64")]
pub(crate) mod avx512;

/// Whether to use the AVX-512 code, if the CPU supports it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Simd {
    Auto,
    #[cfg(test)]
    Never,
}

#[cfg(target_arch = "x86_64")]
impl Simd {
    pub(crate) fn avx512(self) -> bool {
        #[cfg(test)]
        if self == Simd::Never {
            return false;
        }
        avx512::is_available()
    }
}

/// Encode `input` as ASCII and append the result to `output_buf`; see [`encode_ascii_byte`] for
/// `floor`.
pub(crate) fn encode_buffer_ascii(input: &[u8], output_buf: &mut Vec<u8>, floor: usize) {
    encode_buffer_ascii_with(input, output_buf, floor, Simd::Auto);
}

#[cfg_attr(not(target_arch = "x86_64"), allow(unused_variables))]
fn encode_buffer_ascii_with(input: &[u8], output_buf: &mut Vec<u8>, floor: usize, simd: Simd) {
    #[cfg(target_arch = "x86_64")]
    let avx512 = simd.avx512();
    for chunk in input.chunks(ENCODE_CHUNK) {
        output_buf.reserve(chunk.len() * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
        #[cfg(target_arch = "x86_64")]
        let chunk = if avx512 {
            // SAFETY: we checked that the CPU supports the required features, we reserved
            // `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the chunk, and the first `cur`
            // bytes are initialized
            cur = unsafe { avx512::encode_blocks(chunk, out, cur, floor) };
            &chunk[chunk.len() / 64 * 64..]
        } else {
            chunk
        };
        // NOTE: manually unrolled, since the compiler does not unroll it by itself
        let (quads, rest) = chunk.as_chunks::<4>();
        for quad in quads {
            for &c in quad {
                // SAFETY: we reserved `MAX_OUTPUT_PER_INPUT_BYTE` bytes for each byte of the
                // chunk, and the first `cur` bytes are initialized since we only increase `cur`
                // after writing
                cur = unsafe { encode_ascii_byte(c, out, cur, floor) };
            }
        }
        for &c in rest {
            // SAFETY: see above
            cur = unsafe { encode_ascii_byte(c, out, cur, floor) };
        }
        // SAFETY: the first `cur` bytes of `output_buf` are initialized because we only increase
        // `cur` after writing to `output_buf`
        unsafe { output_buf.set_len(cur) };
    }
}

/// Random input biased towards the characters that need special care in the vector code.
#[cfg(test)]
pub(crate) fn tricky_input(rng: &mut impl rand::Rng, len: usize) -> Vec<u8> {
    const ALPHABET: &[u8] = b"\t\n\r %&#^*~\0EeTt0Q$@!/.\xc3\xa9\x80\x89\xff";
    let p_special: f64 = rng.gen_range(0.0..1.0);
    (0..len)
        .map(|_| {
            if rng.gen_bool(p_special) {
                ALPHABET[rng.gen_range(0..ALPHABET.len())]
            } else {
                rng.gen_range(b'a'..=b'z')
            }
        })
        .collect()
}

/// Output prefixes for differential tests, as the part before `floor` (which whitespace must not
/// modify) and the part after (like the letter space held back by `EncodeStream`, which whitespace
/// may remove).
#[cfg(test)]
pub(crate) const TEST_PREFIXES: [(&[u8], &[u8]); 5] = [
    (b"", b""),
    (b"x", b""),
    (b" ", b""),
    (b"", b" "),
    (b"x ", b" "),
];

#[test]
#[cfg_attr(miri, ignore)]
fn test_ascii_encode_simd_matches_scalar() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let iterations = if cfg!(debug_assertions) { 1000 } else { 20000 };
    for iteration in 0..iterations {
        let len = if iteration % 10 == 0 {
            rng.gen_range(0..20000)
        } else {
            rng.gen_range(0..300)
        };
        let data = tricky_input(&mut rng, len);
        for (before_floor, after_floor) in TEST_PREFIXES {
            let run = |simd| {
                let mut output = before_floor.to_vec();
                let floor = output.len();
                output.extend_from_slice(after_floor);
                encode_buffer_ascii_with(&data, &mut output, floor, simd);
                output
            };
            assert_eq!(run(Simd::Auto), run(Simd::Never), "{data:?}");
        }
    }
}
