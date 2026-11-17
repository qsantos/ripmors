# Changelog

## v0.3.0

- New API: `Encoder` (with a `Charset`) and `Decoder` (with an `Alphabet`, or a custom mapping
  with `Decoder::from_fn`), with:
    - `encode`/`decode` to a new `String`, and `encode_into`/`decode_into` to append to an
      existing one,
    - `encode_stream`/`decode_stream` from a `Read` to a `Write` object,
    - `stream()`, returning an `EncodeStream`/`DecodeStream` to feed input in pieces (bring your
      own I/O),
    - `encode_strict`/`decode_strict`, which report the position of the first problem.
- Remove `encode_string`, `encode_string_ascii`, `encode_stream`, `encode_stream_ascii`,
  `decode_string` and `decode_stream`, replaced by the new API.
- Change the decoding rules:
    - only ` `, `/`, `\t`, `\n` and `\r` are separators; other control characters are no
      longer copied to the output,
    - `/` decodes the pending Morse sequence (`-/-` was decoded as ` T`, it is now `T T`),
    - Morse sequences containing other bytes, or with more than 7 elements, are dropped instead
      of being decoded to arbitrary characters.
- Fix encoding streams stopping at the first invalid UTF-8 byte: invalid bytes are now skipped.
- Fix encoding streams dropping the end of the output when it did not end with a letter space
  (e.g. when the input ended with a newline).
- Add multi-threaded encoding and decoding: `encode_parallel`/`decode_parallel`,
  `encode_parallel_write`/`decode_parallel_write` and `encode_stream_parallel`/
  `decode_stream_parallel`.
- Optimize encoding (2.7x for ASCII, 4.5x for Russian or Japanese) and decoding (19x to 27x),
  with AVX-512 when available (F, BW, VBMI, VBMI2 and GFNI) and lookup tables for non-ASCII
  characters.
- Command-line tool:
    - use all the available cores; set the number of threads with `-j`/`--threads`,
    - memory-map the input when it is a regular file, and enlarge pipes (`fast-io` feature,
      enabled by default),
    - exit quietly when its output is closed (e.g. `ripmors | head`).

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
