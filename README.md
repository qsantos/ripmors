# ripmors

ripmors is a Rust library for encoding and decoding [International Morse code](https://en.wikipedia.org/wiki/Morse_code) and several variants.

It is _fast_:

- Encoding ASCII text to Morse code: **1.5 GiB/s**
- Encoding Unicode text to Morse code: 730 MiB/s
- Decoding Morse code: 570 MiB/s

```shell
$ echo 'Hello, World!' | ripmors
.... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--
$ echo '-- --- .-. ... . / -.-. --- -.. .' | ripmors -d
MORSE CODE
```

[Documentation](https://docs.rs/ripmors/latest/ripmors/)

If you want to uwu-encode instead, use [this project instead](https://github.com/Daniel-Liu-c0deb0t/uwu).

## Usage

Or, in Rust:

```rust
use ripmors::{Alphabet, Decoder, Encoder};

let encoder = Encoder::default();
assert_eq!(encoder.encode(b"Hello, World!"), ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--");

let decoder = Decoder::new(Alphabet::Standard);
assert_eq!(decoder.decode(b"-- --- .-. ... . / -.-. --- -.. ."), "MORSE CODE");
```

To avoid allocations, use `encode_into` and `decode_into` with a reused `String`. To bring your own
I/O, feed input in pieces to `Encoder::stream` and `Decoder::stream`. To detect characters that
cannot be encoded and invalid Morse code, use `encode_strict` and `decode_strict`.

In addition to the standard International Morse Code and its Latin extensions,
the following variants are supported:

- Greek
- [Russian](https://en.wikipedia.org/wiki/Russian_Morse_code) (Cyrillic)
- [Japanese](https://en.wikipedia.org/wiki/Wabun_code) (Hiragana, Katakana)
- [Korean](https://en.wikipedia.org/wiki/SKATS) (Hangul)
- Hebrew
- Arabic

```shell
$ echo 'モールスふごう' | ripmors
-..-. .--.- -.--. ---.- --.. ---- .. ..-
$ echo '-..-. .--.- -.--. ---.- --.. ---- .. ..-' | ripmors -d japanese
モールスフコ゛ウ
```

Or, in Rust:

```rust
use ripmors::{Alphabet, Decoder, Encoder};

assert_eq!(Encoder::default().encode("モールスふごう".as_bytes()), "-..-. .--.- -.--. ---.- --.. ---- .. ..-");
assert_eq!(Decoder::new(Alphabet::Japanese).decode(b"-..-. .--.- -.--. ---.- --.. ---- .. ..-"), "モールスフコ゛ウ");
```
