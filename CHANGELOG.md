# Changelog

## Unreleased

- New API: `Encoder` (with a `Charset`) and `Decoder` (with an `Alphabet`, or a custom mapping
  with `Decoder::from_fn`), with:
    - `encode`/`decode` to a new `String`, and `encode_into`/`decode_into` to append to an
      existing one,
    - `encode_stream`/`decode_stream` from a `Read` to a `Write` object,
    - `stream()`, returning an `EncodeStream`/`DecodeStream` to feed input in pieces (bring your
      own I/O),
    - `encode_strict`/`decode_strict`, which report the position of the first problem.
- Deprecate `encode_string`, `encode_string_ascii`, `encode_stream`, `encode_stream_ascii`,
  `decode_string` and `decode_stream` in favor of the new API.
- Change the decoding rules (also for the deprecated functions):
    - only ` `, `/`, `\t`, `\n` and `\r` are separators; other control characters are no
      longer copied to the output,
    - `/` decodes the pending Morse sequence (`-/-` was decoded as ` T`, it is now `T T`),
    - Morse sequences containing other bytes, or with more than 7 elements, are dropped instead
      of being decoded to arbitrary characters.
- Fix `encode_stream` stopping at the first invalid UTF-8 byte: invalid bytes are now skipped.
- Fix `encode_stream` dropping the end of the output when it did not end with a letter space
  (e.g. when the input ended with a newline).
- Optimize encoding (1.4x to 1.8x) and decoding (10x with AVX-512, 2.5x without).
- The command-line tool exits quietly when its output is closed (e.g. `ripmors | head`).

## v0.2.0

- Fix `」` instead of Japanese full stop `。` being mapped to `.-.-..`
- Fix `!` being encoded as `..--.` instead of `-.-.--`
- Fix `Ð` being encoded to `..-.` instead of `..--.`
- Add decoding of Latin extension based on Wikipedia article:
    - `----` → `Ĥ`
    - `...-.` → `Ŝ`
    - `..--.` → `Ð`
    - `.-..-` → `È`
    - `.--..` → `Þ`
    - `.---.` → `Ĵ`
    - `--.-.` → `Ĝ`
    - `...-...` → `Ś`
    - `...--..` → `ß`
- Add decoding of numbers (same as standard) and punctuation in Russian:
    - `......` → `.`
    - `.-.-.-` → `,`
    - `---...` → `:`
    - `-.-.-` → `;`
    - `-.--.-` → `(`
    - `.----.` → `'`
    - `.-..-.` → `"`
    - `-....-` → `—`
    - `-..-.` → `/`
    - `..--..` → `?`
    - `--..--` → `!`
    - `-...-` → `-`
    - `.--.-.` → `@`
- `encode_stream`, `encode_stream_ascii` and `decode_stream` now return `Result<(), std::io::Error>`
- Code refactor and optimizations:
    - `encode_string_ascii`: 0.97 GiB/s → 1.35 GiB/s (+42%)
    - `encode_string`: 0.75 GiB/s → 0.82 GB/s (+12%)
    - `encode_stream`: 0.65 GiB/s → 0.75 GiB/s (+17%)

## v0.1.0

- Encoding from Unicode to Morse code
- Encoding from ASCII to Morse code
- Decoding from standard Morse code and variants:
    - Arabic
    - Greek
    - Hebrew
    - Japanese (Wabun code)
    - Korean (SKATS)
    - Russian
