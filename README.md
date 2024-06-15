# ripmors

ripmors is a Rust library for encoding and decoding international Morse code and several variants.

It is _fast_:

- Encoding ASCII text to Morse code: **1.5 GiB/s**
- Encoding Unicode text to Morse code: 730 MiB/s
- Decoding Morse code: 570 MiB/s

```
$ echo 'Hello, World!' | ripmors
.... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--.
$ echo '-- --- .-. ... . / -.-. --- -.. .' | ripmors -d
MORSE CODE
```

In addition to the standard International Morse Code and its Latin extensions,
the following variants are supported:

- Greek
- Russian (Cyrillic)
- Japanese (Hiragana, Katakana)
- Korean (Hangul)
- Hebrew
- Arabic

```
$ echo 'モールスふごう' | ripmors
-..-. .--.- -.--. ---.- --.. ---- .. ..-
$ echo '-..-. .--.- -.--. ---.- --.. ---- .. ..-' | ripmors -d japanese
モルスフコ゛ウ
```
