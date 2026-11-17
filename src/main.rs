use ripmors::{Alphabet, Charset, Decoder, Encoder};

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

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let result = if let Some(variant) = args.decode {
        let alphabet = match variant {
            DecodeVariant::Standard => Alphabet::Standard,
            DecodeVariant::Greek => Alphabet::Greek,
            DecodeVariant::Russian => Alphabet::Russian,
            DecodeVariant::Japanese => Alphabet::Japanese,
            DecodeVariant::Korean => Alphabet::Korean,
            DecodeVariant::Hebrew => Alphabet::Hebrew,
            DecodeVariant::Arabic => Alphabet::Arabic,
        };
        Decoder::new(alphabet).decode_stream(&mut stdin, &mut stdout)
    } else {
        let charset = match args.encode.unwrap_or_default() {
            EncodeVariant::Unicode => Charset::Utf8,
            EncodeVariant::Ascii => Charset::Ascii,
        };
        Encoder::new(charset).encode_stream(&mut stdin, &mut stdout)
    };
    match result {
        Ok(()) => (),
        // the reader went away (e.g. `ripmors | head`): this is not an error
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => (),
        Err(e) => {
            eprintln!("ripmors: {e}");
            std::process::exit(1);
        }
    }
}
