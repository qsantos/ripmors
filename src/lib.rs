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

pub fn ascii_encode_vec_u8(s: &[u8], buf: &mut Vec<u8>) {
    buf.extend(ascii_to_morse(s[0]).as_bytes());
    for c in &s[1..] {
        buf.push(b' ');
        buf.extend(ascii_to_morse(*c).as_bytes());
    }
}

pub fn unicode_encode(s: &str) -> String {
    let parts: Vec<&str> = s
        .chars()
        .map(|b| unicode_to_morse(b))
        .filter(|&x| x != "")
        .collect();
    parts.join(" ")
}
