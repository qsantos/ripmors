use std::io::BufWriter;

use ripmors::{decode_stream, encode_stream, morse_to_standard};

fn compare_output_to_oracle(writer: BufWriter<Vec<u8>>, expected_filename: &str) {
    let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();
    let expected = std::fs::read_to_string(expected_filename).unwrap();
    for (l1, l2) in expected.lines().zip(output.lines()) {
        assert_eq!(l1, l2, "Second-string should be same as first string");
    }
}

#[test]
fn test_encode_stream() {
    let mut f = std::fs::File::open("1-original.txt").unwrap();
    let mut writer = BufWriter::new(Vec::new());
    encode_stream(&mut f, &mut writer);
    compare_output_to_oracle(writer, "2-encoded.txt");
}

#[test]
fn test_decode_stream() {
    let mut f = std::fs::File::open("2-encoded.txt").unwrap();
    let mut writer = BufWriter::new(Vec::new());
    decode_stream(&mut f, &mut writer, &morse_to_standard);
    compare_output_to_oracle(writer, "3-decoded.txt");
}
