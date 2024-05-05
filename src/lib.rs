mod mappings;

use mappings::{ascii_to_morse, unicode_to_morse};

pub fn ascii_encode(s: &[u8]) -> String {
    let parts: Vec<&str> = s
        .iter()
        .map(|b| ascii_to_morse(*b))
        .filter(|&x| x != "")
        .collect();
    parts.join(" ")
}

pub fn unicode_encode(s: &str) -> String {
    let parts: Vec<&str> = s
        .chars()
        .map(|b| unicode_to_morse(b))
        .filter(|&x| x != "")
        .collect();
    parts.join(" ")
}
