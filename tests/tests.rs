use std::io::BufWriter;

use ripmors::{decode_stream, encode_stream_ascii, encode_stream_unicode, to_standard};

fn compare_output_to_oracle(writer: BufWriter<Vec<u8>>, expected_filename: &str) {
    let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();
    let expected = std::fs::read_to_string(expected_filename).unwrap();
    let mut output_lines = output.split('\n');
    let mut expected_lines = expected.split('\n');
    loop {
        match (expected_lines.next(), output_lines.next()) {
            (Some(l1), Some(l2)) => {
                assert_eq!(l1, l2, "Second-string should be same as first string");
            }
            (Some(l1), None) => {
                panic!("Output stops before expected line: {:?}", l1);
            }
            (None, Some(l2)) => {
                panic!("Output has extra lines starting with: {:?}", l2);
            }
            (None, None) => break,
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_encode_stream_ascii() {
    let mut f = std::fs::File::open("1-original.txt").unwrap();
    let mut writer = BufWriter::new(Vec::new());
    encode_stream_ascii(&mut f, &mut writer);
    compare_output_to_oracle(writer, "2-encoded.txt");
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_encode_stream_unicode() {
    let mut f = std::fs::File::open("1-original.txt").unwrap();
    let mut writer = BufWriter::new(Vec::new());
    encode_stream_unicode(&mut f, &mut writer);
    compare_output_to_oracle(writer, "2-encoded.txt");
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_decode_stream() {
    let mut f = std::fs::File::open("2-encoded.txt").unwrap();
    let mut writer = BufWriter::new(Vec::new());
    decode_stream(&mut f, &mut writer, to_standard);
    compare_output_to_oracle(writer, "3-decoded.txt");
}
