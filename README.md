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

## Usage

Or, in Rust:

```rust
use ripmors::{decode_string, encode_string, to_standard};

assert_eq!(encode_string("Hello, World!"), ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--");
assert_eq!(decode_string(b"-- --- .-. ... . / -.-. --- -.. .", to_standard), "MORSE CODE");
```

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
use ripmors::{decode_string, encode_string, to_japanese};

assert_eq!(encode_string("モールスふごう"), "-..-. .--.- -.--. ---.- --.. ---- .. ..-");
assert_eq!(decode_string(b"-..-. .--.- -.--. ---.- --.. ---- .. ..-", to_japanese), "モールスフコ゛ウ");
```
