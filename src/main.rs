use std::io::{BufWriter, Read, Write};

use ripmors::*;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Default, Clone, Copy, Eq, Parser, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum MorseVariant {
    #[default]
    Standard,
    Greek,
    Russian,
    Japanese,
    Korean,
    Hebrew,
    Arabic,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, num_args = 0..=1, default_missing_value = "standard")]
    decode: Option<MorseVariant>,
}

fn encode_stream<R: Read, W: Write>(i: &mut R, o: &mut W) {
    let mut input_buf = vec![0u8; 1 << 15];
    loop {
        let n = i.read(&mut input_buf).unwrap();
        if n == 0 {
            break;
        }
        ascii_encode_to_writer(o, &input_buf[..n]).unwrap();
    }
}

fn decode_stream<R: Read, W: Write, F: Fn(&str) -> char>(i: &mut R, o: &mut W, char_decode: &F) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_read = 0;
    loop {
        bytes_read += i.read(&mut input_buf[bytes_read..]).unwrap();
        if bytes_read == 0 {
            break;
        }
        let s = match std::str::from_utf8(&input_buf[..bytes_read]) {
            Ok(s) => s,
            Err(e) => {
                let bytes_decoded = e.valid_up_to();
                unsafe { std::str::from_utf8_unchecked(&input_buf[..bytes_decoded]) }
            }
        };

        morse_decode_to_writer(o, s, char_decode).unwrap();

        let bytes_decoded = s.bytes().len();
        input_buf.copy_within(bytes_decoded..bytes_read, 0);
        bytes_read -= bytes_decoded;
    }
}

fn main() {
    let args = Args::parse();

    let mut stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut buf_writer = BufWriter::new(stdout);

    if let Some(variant) = args.decode {
        let char_decode = match variant {
            MorseVariant::Standard => morse_to_standard,
            MorseVariant::Greek => morse_to_greek,
            MorseVariant::Russian => morse_to_russian,
            MorseVariant::Japanese => morse_to_japanese,
            MorseVariant::Korean => morse_to_korean,
            MorseVariant::Hebrew => morse_to_hebrew,
            MorseVariant::Arabic => morse_to_arabic,
        };
        decode_stream(&mut stdin, &mut buf_writer, &char_decode);
    } else {
        encode_stream(&mut stdin, &mut buf_writer);
    }
}
