use std::io::{BufWriter, Read};

use ripmors::ascii_encode_to_writer;

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

fn main() {
    let args = Args::parse();
    dbg!(args);

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
