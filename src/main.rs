use ripmors::*;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Default, Clone, Copy, Eq, Parser, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum DecodeVariant {
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
    decode: Option<DecodeVariant>,
}

fn main() {
    let args = Args::parse();

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    if let Some(variant) = args.decode {
        let char_decode = match variant {
            DecodeVariant::Standard => morse_to_standard,
            DecodeVariant::Greek => morse_to_greek,
            DecodeVariant::Russian => morse_to_russian,
            DecodeVariant::Japanese => morse_to_japanese,
            DecodeVariant::Korean => morse_to_korean,
            DecodeVariant::Hebrew => morse_to_hebrew,
            DecodeVariant::Arabic => morse_to_arabic,
        };
        decode_stream(&mut stdin, &mut stdout, &char_decode);
    } else {
        encode_stream(&mut stdin, &mut stdout);
    }
}
