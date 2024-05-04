pub fn ascii_to_morse(c: u8) -> &'static str {
    match c {
        // NOTE: look for `=> "",` for missing characters

        b' ' => "/",

        b'!' => "..--.", // non standard: mapped to interrogation mark 
        b'"' => ".-..-.", // Straight quotes (1.1.3)
        b'#' => "",
        b'$' => "...-..-", // non standard
        b'%' => "----- -..-. -----", // (3.3.1)
        b'&' => ". ...", // non standard: mapped to "es"

        b'\'' => ".----.", // Apostrophe (1.1.3)
        b'(' => "-.--.", // Left-hand bracket (parenthesis) (1.1.3)
        b')' => "-.--.-", // Right-hand bracket (parenthesis) (1.1.3)
        b'*' => "-..-", // Multiplication sign (same as letter X) (1.1.3)
        b'+' => ".-.-.", // Cross or addition sign (1.1.3)
        b',' => "--..--", // Comma (1.1.3)
        b'-' => "-....-", // Hyphen (1.1.3)
        b'.' => ".-.-.-", // Full stop (period) (1.1.3)
        b'/' => "-..-.", // Fraction bar or division sign (1.1.3)

        // International Morse code, as per ITU-R M.1677-1
        //
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
        b'<' => "",
        b'=' => "-...-",  // Double hyphen (1.1.3)
        b'>' => "",
        b'?' => "..--..", // Question mark (1.1.3)
        b'@' => ".--.-.", // Commercial at (1.1.3)

        // 1. Morse code signals
        // 1.1.1. Letters (Latins cript)
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

        b'[' => "",
        b'\\' => "",
        b']' => "",
        b'^' => "",
        b'_' => "",
        b'`' => ".-----.", // non standard

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

        b'{' => "",
        b'|' => "",
        b'}' => "",
        b'~' => "",

        _ => "", // 33 control characters
    }
}
