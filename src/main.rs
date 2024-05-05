use std::io::{Read, Write};

use ripmors::ascii_encode_vec_u8;

fn main() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut input_buf = [0u8; 1 << 15];
    let mut output_buf = Vec::new();
    loop {
        let n = stdin.read(&mut input_buf).unwrap();
        output_buf.clear();
        ascii_encode_vec_u8(&input_buf[..n], &mut output_buf);
        stdout.write(&output_buf).unwrap();
    }
}
