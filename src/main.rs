use std::io::{BufWriter, Read};

use ripmors::ascii_encode_to_writer;

fn main() {
    let mut stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut buf_writer = BufWriter::new(stdout);

    let mut input_buf = [0u8; 1 << 15];
    loop {
        let n = stdin.read(&mut input_buf).unwrap();
        if n == 0 {
            break;
        }
        ascii_encode_to_writer(&mut buf_writer, &input_buf[..n]).unwrap();
    }
}
