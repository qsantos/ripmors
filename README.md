# ripmors

ripmors is a Rust library for encoding and decoding [International Morse code](https://en.wikipedia.org/wiki/Morse_code) and several variants.

It is _fast_:

- Encoding ASCII text to Morse code: 15 GiB/s
- Encoding Unicode text to Morse code: 8.3 GiB/s
- Decoding Morse code: 15 GiB/s

Even with a single core:

- Encoding ASCII text to Morse code: 2.5 GiB/s
- Encoding Unicode text to Morse code: 1.2 GiB/s
- Decoding Morse code: 6.1 GiB/s

Measured on AMD Ryzen 9 7950X3D.

```shell
$ echo 'Hello, World!' | ripmors
.... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--
$ echo '-- --- .-. ... . / -.-. --- -.. .' | ripmors -d
MORSE CODE
```

[Documentation](https://docs.rs/ripmors/latest/ripmors/)

If you want to uwu-encode instead, use [this project instead](https://github.com/Daniel-Liu-c0deb0t/uwu).

## Usage

```rust
use ripmors::{Alphabet, Decoder, Encoder};

let encoder = Encoder::default();
assert_eq!(encoder.encode(b"Hello, World!"), ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. -.-.--");

let decoder = Decoder::new(Alphabet::Standard);
assert_eq!(decoder.decode(b"-- --- .-. ... . / -.-. --- -.. ."), "MORSE CODE");
```

Note that the above will instantiate and return `String` instances. To avoid
needless allocations, use the `encode_into`/`decode_into` API:

```rust
use ripmors::{Alphabet, Decoder, Encoder};

let encoder = Encoder::default();
let decoder = Decoder::new(Alphabet::Standard);
let mut morse = String::new();
let mut text = String::new();
for (line, expected) in [("Hello", "HELLO"), ("World", "WORLD")] {
    morse.clear(); // reuse the allocation
    encoder.encode_into(line.as_bytes(), &mut morse);
    text.clear();
    decoder.decode_into(morse.as_bytes(), &mut text);
    assert_eq!(text, expected);
}
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
use ripmors::{Alphabet, Decoder, Encoder};

assert_eq!(Encoder::default().encode("モールスふごう".as_bytes()), "-..-. .--.- -.--. ---.- --.. ---- .. ..-");
assert_eq!(Decoder::new(Alphabet::Japanese).decode(b"-..-. .--.- -.--. ---.- --.. ---- .. ..-"), "モールスフコ゛ウ");
```
