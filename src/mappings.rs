pub const ASCII_TO_MORSE: [&str; 256] = {
    // NOTE: ASCII only covers bytes values from 0 to 127, but we make the mapping from 0 to 255
    // for convenience when iterating over bytes
    let mut v = [""; 256];

    // NOTE: look for `= "";" for missing characters

    // 32 first control characters

    v[b' ' as usize] = "/";

    v[b'!' as usize] = "..--."; // non standard: mapped to interrogation mark 
    v[b'"' as usize] = ".-..-."; // Straight quotes (1.1.3)
    v[b'#' as usize] = "";
    v[b'$' as usize] = "...-..-"; // non standard
    v[b'%' as usize] = "----- -..-. -----"; // (3.3.1)
    v[b'&' as usize] = ". ..."; // non standard: mapped to "es"

    v[b'\'' as usize] = ".----."; // Apostrophe (1.1.3)
    v[b'(' as usize] = "-.--."; // Left-hand bracket (parenthesis) (1.1.3)
    v[b')' as usize] = "-.--.-"; // Right-hand bracket (parenthesis) (1.1.3)
    v[b'*' as usize] = "-..-"; // Multiplication sign (same as letter X) (1.1.3)
    v[b'+' as usize] = ".-.-."; // Cross or addition sign (1.1.3)
    v[b',' as usize] = "--..--"; // Comma (1.1.3)
    v[b'-' as usize] = "-....-"; // Hyphen (1.1.3)
    v[b'.' as usize] = ".-.-.-"; // Full stop (period) (1.1.3)
    v[b'/' as usize] = "-..-."; // Fraction bar or division sign (1.1.3)

    // International Morse code, as per ITU-R M.1677-1
    //
    // 1.1.2. Figures (Hindu-Arab digits)
    v[b'0' as usize] = "-----";
    v[b'1' as usize] = ".----";
    v[b'2' as usize] = "..---";
    v[b'3' as usize] = "...--";
    v[b'4' as usize] = "....-";
    v[b'5' as usize] = ".....";
    v[b'6' as usize] = "-....";
    v[b'7' as usize] = "--...";
    v[b'8' as usize] = "---..";
    v[b'9' as usize] = "----.";

    v[b':' as usize] = "---..."; // Colon r division sign (1.1.3)
    v[b';' as usize] = "-.-.-."; // non standard
    v[b'<' as usize] = "";
    v[b'=' as usize] = "-...-";  // Double hyphen (1.1.3)
    v[b'>' as usize] = "";
    v[b'?' as usize] = "..--.."; // Question mark (1.1.3)
    v[b'@' as usize] = ".--.-."; // Commercial at (1.1.3)

    // 1. Morse code signals
    // 1.1.1. Letters (Latins cript)
    // Uppercase
    v[b'A' as usize] = ".-";
    v[b'B' as usize] = "-...";
    v[b'C' as usize] = "-.-.";
    v[b'D' as usize] = "-..";
    v[b'E' as usize] = ".";
    v[b'F' as usize] = "..-.";
    v[b'G' as usize] = "--.";
    v[b'H' as usize] = "....";
    v[b'I' as usize] = "..";
    v[b'J' as usize] = ".---";
    v[b'K' as usize] = "-.-";
    v[b'L' as usize] = ".-..";
    v[b'M' as usize] = "--";
    v[b'N' as usize] = "-.";
    v[b'O' as usize] = "---";
    v[b'P' as usize] = ".--.";
    v[b'Q' as usize] = "--.-";
    v[b'R' as usize] = ".-.";
    v[b'S' as usize] = "...";
    v[b'T' as usize] = "-";
    v[b'U' as usize] = "..-";
    v[b'V' as usize] = "...-";
    v[b'W' as usize] = ".--";
    v[b'X' as usize] = "-..-";
    v[b'Y' as usize] = "-.--";
    v[b'Z' as usize] = "--..";

    v[b'[' as usize] = "";
    v[b'\\' as usize] = "";
    v[b']' as usize] = "";
    v[b'^' as usize] = "";
    v[b'_' as usize] = "";
    v[b'`' as usize] = ".-----."; // non standard

    // Lowercase
    v[b'a' as usize] = ".-";
    v[b'b' as usize] = "-...";
    v[b'c' as usize] = "-.-.";
    v[b'd' as usize] = "-..";
    v[b'e' as usize] = ".";
    v[b'f' as usize] = "..-.";
    v[b'g' as usize] = "--.";
    v[b'h' as usize] = "....";
    v[b'i' as usize] = "..";
    v[b'j' as usize] = ".---";
    v[b'k' as usize] = "-.-";
    v[b'l' as usize] = ".-..";
    v[b'm' as usize] = "--";
    v[b'n' as usize] = "-.";
    v[b'o' as usize] = "---";
    v[b'p' as usize] = ".--.";
    v[b'q' as usize] = "--.-";
    v[b'r' as usize] = ".-.";
    v[b's' as usize] = "...";
    v[b't' as usize] = "-";
    v[b'u' as usize] = "..-";
    v[b'v' as usize] = "...-";
    v[b'w' as usize] = ".--";
    v[b'x' as usize] = "-..-";
    v[b'y' as usize] = "-.--";
    v[b'z' as usize] = "--..";

    v[b'{' as usize] = "";
    v[b'|' as usize] = "";
    v[b'}' as usize] = "";
    v[b'~' as usize] = "";

    // last control character (DEL)

    v
};
