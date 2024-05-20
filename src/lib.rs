mod mappings;

use mappings::{ascii_to_morse, morse_to_ascii, unicode_to_morse};

pub fn ascii_encode(s: &str) -> String {
    let parts: Vec<&str> = s.chars().map(ascii_to_morse).filter(|&x| x != "").collect();
    parts.join(" ")
}

pub fn ascii_encode_vec_u8(s: &[u8], buf: &mut Vec<u8>) {
    buf.extend(ascii_to_morse(s[0] as char).as_bytes());
    for c in &s[1..] {
        let morse = ascii_to_morse(*c as char);
        if morse != "" {
            buf.push(b' ');
            buf.extend(morse.as_bytes());
        }
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

pub fn ascii_decode(s: &str) -> String {
    let parts: Vec<&str> = s
        .split(" ")
        .map(morse_to_ascii)
        .filter(|&x| x != "")
        .collect();
    parts.join("")
}

#[test]
fn test_ascii_encode() {
    assert_eq!(ascii_encode("PARIS"), ".--. .- .-. .. ...");
    assert_eq!(
        ascii_encode("Hello, World!"),
        ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."
    );
}

#[test]
fn test_unicode_encode() {
    assert_eq!(
        unicode_encode("télégraphie"),
        "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. ."
    );
    assert_eq!(unicode_encode("でんしん"), ".-.-- .. .-.-. --.-. .-.-.");
    assert_eq!(unicode_encode("تلغراف"), "- .-.. --. .-. .- ..-.");
    assert_eq!(
        unicode_encode("телеграфия"),
        "- . .-.. . --. .-. .- ..-. .. .-.-"
    );
    assert_eq!(
        unicode_encode("τηλεγραφία"),
        "- .... .-.. . --. .-. .- ..-. .-"
    );
}

#[test]
fn test_ascii_decode() {
    assert_eq!(ascii_decode(".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        ascii_decode(".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."),
        "HELLO, WORLD!",
    );
}
