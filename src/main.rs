use ripmors::*;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Default, Clone, Copy, Eq, Parser, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum EncodeVariant {
    #[default]
    Unicode,
    Ascii,
}

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
    #[arg(short, long, num_args = 0..=1, default_missing_value = "unicode")]
    encode: Option<EncodeVariant>,
}

fn main() {
    let args = Args::parse();

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    if let Some(variant) = args.decode {
        let char_decode = match variant {
            DecodeVariant::Standard => to_standard,
            DecodeVariant::Greek => to_greek,
            DecodeVariant::Russian => to_russian,
            DecodeVariant::Japanese => to_japanese,
            DecodeVariant::Korean => to_korean,
            DecodeVariant::Hebrew => to_hebrew,
            DecodeVariant::Arabic => to_arabic,
        };
        decode_stream(&mut stdin, &mut stdout, char_decode);
    } else if args.encode == Some(EncodeVariant::Ascii) {
        encode_stream_ascii(&mut stdin, &mut stdout);
    } else {
        encode_stream(&mut stdin, &mut stdout);
    }
}
