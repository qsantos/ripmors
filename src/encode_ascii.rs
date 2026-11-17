use crate::encode_ascii_mapping::ASCII_TO_QWORD;

/// Maximum number of bytes written to the output for any input byte (`%` is encoded as 18 bytes).
pub(crate) const MAX_OUTPUT_PER_INPUT_BYTE: usize = 18;

/// Number of input bytes processed between two reservations of output space; this keeps the
/// reservation proportional to the actual output instead of the worst case for the whole input.
pub(crate) const ENCODE_CHUNK: usize = 1 << 12;

/// Value of [`ENCODE_LEN`] for characters that need special handling.
const SPECIAL: u8 = 0xff;

/// Qwords of [`ASCII_TO_QWORD`], as a separate table so that indexing needs no extra arithmetic.
static ENCODE_QWORD: [u64; 256] = {
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

/// Encode `input` as ASCII and append the result to `output_buf`; see [`encode_ascii_byte`] for
/// `floor`.
pub(crate) fn encode_buffer_ascii(input: &[u8], output_buf: &mut Vec<u8>, floor: usize) {
    for chunk in input.chunks(ENCODE_CHUNK) {
        output_buf.reserve(chunk.len() * MAX_OUTPUT_PER_INPUT_BYTE);
        let out = output_buf.as_mut_ptr();
        let mut cur = output_buf.len();
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
