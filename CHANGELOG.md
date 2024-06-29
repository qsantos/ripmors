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
