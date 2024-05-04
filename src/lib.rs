mod mappings;

use mappings::ASCII_TO_MORSE;

pub fn ascii_encode(s: &str) -> String {
    let parts: Vec<&str> = s.as_bytes().iter().map(|b| ASCII_TO_MORSE[*b as usize]).collect();
    parts.join(" ")
}
