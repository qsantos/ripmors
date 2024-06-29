# Changelog

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
