// International Morse code, as per ITU-R M.1677-1

#[repr(align(64))]
struct AlignedBytes([u8; 8]);

macro_rules! from_ascii {
    ($($letter:expr => $elements:literal),+ $(,)? ) => {
        pub const ASCII_TO_BYTES: [(&'static [u8], usize); 256] = {
            let mut x: [(&'static [u8], usize); 256] = [(&[0u8; 8], 0); 256];
            $(
                assert!($letter >= 0);
                assert!($letter <= 255);
                let (elements, len) = match $elements.len() {
                    1 => (concat!($elements, " \0\0\0\0\0\0").as_bytes(), 2),
                    2 => (concat!($elements, " \0\0\0\0\0").as_bytes(), 3),
                    3 => (concat!($elements, " \0\0\0\0").as_bytes(), 4),
                    4 => (concat!($elements, " \0\0\0").as_bytes(), 5),
                    5 => (concat!($elements, " \0\0").as_bytes(), 6),
                    6 => (concat!($elements, " \0").as_bytes(), 7),
                    _ => (concat!($elements, " ").as_bytes(), $elements.len() + 1),
                };
                x[$letter as usize] = (elements, len);
            )+
            x[b'\t' as usize] = (b"\t\0\0\0\0\0\0\0", 1);
            x[b'\n' as usize] = (b"\n\0\0\0\0\0\0\0", 1);
            x[b'\r' as usize] = (b"\r\0\0\0\0\0\0\0", 1);
            x
        };
        pub const ASCII_TO_QWORD: [(u64, usize); 256] = {
            let mut x: [(u64, usize); 256] = [(0, 0); 256];
            $(
                assert!($letter >= 0);
                assert!($letter <= 255);
                let (elements, len) = match $elements.len() {
                    1 => (concat!($elements, " \0\0\0\0\0\0"), 2),
                    2 => (concat!($elements, " \0\0\0\0\0"), 3),
                    3 => (concat!($elements, " \0\0\0\0"), 4),
                    4 => (concat!($elements, " \0\0\0"), 5),
                    5 => (concat!($elements, " \0\0"), 6),
                    6 => (concat!($elements, " \0"), 7),
                    7 => (concat!($elements, " "), 8),
                    _ => ("\0\0\0\0\0\0\0\0", 18)
                };
                let eight_bytes = unsafe { &*(elements.as_ptr() as *const [u8; 8]) };
                let aligned_bytes = AlignedBytes(*eight_bytes);
                let one_qword = unsafe { *(aligned_bytes.0.as_ptr() as *const u64) };
                x[$letter as usize] = (one_qword, len);
            )+
            // TODO: this assumes little endian
            x[b'\t' as usize] = (b'\t' as u64, 1);
            x[b'\n' as usize] = (b'\n' as u64, 1);
            x[b'\r' as usize] = (b'\r' as u64, 1);
            x
        };
    };
}

from_ascii! {
    // NOTE: look for `=> "",` for missing characters
    b'\t' => "\t",
    b'\n' => "\n",
    b'\r' => "\r",
    b' ' => "/",

    b'!' => "-.-.--", // non standard
    b'"' => ".-..-.", // Straight quotes (1.1.3)
    b'$' => "...-..-",           // non standard
    // NOTE: % is actually handled in code
    b'%' => "----- -..-. -----", // Mapped to "0/0" (3.3.1)
    b'&' => ". ...",             // non standard: mapped to "es"

    b'\'' => ".----.", // Apostrophe (1.1.3)
    b'(' => "-.--.",   // Left-hand bracket (parenthesis) (1.1.3)
    b')' => "-.--.-",  // Right-hand bracket (parenthesis) (1.1.3)
    b'*' => "-..-",    // Multiplication sign (same as letter X) (1.1.3)
    b'+' => ".-.-.",   // Cross or addition sign (1.1.3)
    b',' => "--..--",  // Comma (1.1.3)
    b'-' => "-....-",  // Hyphen (1.1.3)
    b'.' => ".-.-.-",  // Full stop (period) (1.1.3)
    b'/' => "-..-.",   // Fraction bar or division sign (1.1.3)

    // 1.1.2. Figures (Hindu-Arab digits)
    b'0' => "-----",
    b'1' => ".----",
    b'2' => "..---",
    b'3' => "...--",
    b'4' => "....-",
    b'5' => ".....",
    b'6' => "-....",
    b'7' => "--...",
    b'8' => "---..",
    b'9' => "----.",

    b':' => "---...", // Colon r division sign (1.1.3)
    b';' => "-.-.-.", // non standard
    b'<' => "-.--.",  // non standard: mapped to (
    b'=' => "-...-",  // Double hyphen (1.1.3)
    b'>' => "-.--.-", // non standard: mapped to )
    b'?' => "..--..", // Question mark (1.1.3)
    b'@' => ".--.-.", // Commercial at (1.1.3)

    // 1.1.1. Letters (Latin script)
    // Uppercase
    b'A' => ".-",
    b'B' => "-...",
    b'C' => "-.-.",
    b'D' => "-..",
    b'E' => ".",
    b'F' => "..-.",
    b'G' => "--.",
    b'H' => "....",
    b'I' => "..",
    b'J' => ".---",
    b'K' => "-.-",
    b'L' => ".-..",
    b'M' => "--",
    b'N' => "-.",
    b'O' => "---",
    b'P' => ".--.",
    b'Q' => "--.-",
    b'R' => ".-.",
    b'S' => "...",
    b'T' => "-",
    b'U' => "..-",
    b'V' => "...-",
    b'W' => ".--",
    b'X' => "-..-",
    b'Y' => "-.--",
    b'Z' => "--..",

    b'[' => "-.--.",  // non standard: mapped to (
    b'\\' => "-..-.", // non standard: mapped to /
    b']' => "-.--.-", // non standard: mapped to )
    //'^' => "",
    b'_' => "..--.-",  // non standard
    b'`' => ".-----.", // non standard

    // 1.1.1. Letters (Latin script)
    // Lowercase
    b'a' => ".-",
    b'b' => "-...",
    b'c' => "-.-.",
    b'd' => "-..",
    b'e' => ".",
    b'f' => "..-.",
    b'g' => "--.",
    b'h' => "....",
    b'i' => "..",
    b'j' => ".---",
    b'k' => "-.-",
    b'l' => ".-..",
    b'm' => "--",
    b'n' => "-.",
    b'o' => "---",
    b'p' => ".--.",
    b'q' => "--.-",
    b'r' => ".-.",
    b's' => "...",
    b't' => "-",
    b'u' => "..-",
    b'v' => "...-",
    b'w' => ".--",
    b'x' => "-..-",
    b'y' => "-.--",
    b'z' => "--..",

    b'{' => "-.--.",  // non standard: mapped to (
    b'|' => "-..-.",  // non standard: mapped to /
    b'}' => "-.--.-", // non standard: mapped to )
    //'~' => "",
}
