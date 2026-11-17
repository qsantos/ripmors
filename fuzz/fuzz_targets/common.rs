/// Split `input` into pieces of pseudo-random sizes (including empty pieces) derived from `seed`.
pub fn pieces(input: &[u8], seed: u8) -> Vec<&[u8]> {
    let mut state = u32::from(seed) * 2654435761 + 1;
    let mut pieces = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        // mostly small pieces, sometimes large ones
        let max = if state & 0x100 == 0 { 9 } else { 200 };
        let len = (state as usize % max).min(rest.len());
        let (piece, tail) = rest.split_at(len);
        pieces.push(piece);
        rest = tail;
    }
    pieces
}
