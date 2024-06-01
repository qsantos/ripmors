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

fn main() {
    let args = Args::parse();

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

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
        decode_stream(&mut stdin, &mut stdout, &char_decode);
    } else {
        encode_stream(&mut stdin, &mut stdout);
    }
}
