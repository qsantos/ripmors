mod mappings;

use mappings::ascii_to_morse;

pub fn ascii_encode(s: &str) -> String {
    let parts: Vec<&str> = s.as_bytes().iter().map(|b| ascii_to_morse(*b)).collect();
    parts.join(" ")
}
