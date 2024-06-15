macro_rules! element_to_binary_digit {
    ($binary_value:ident, $elements:expr, $offset:expr) => {
        let elements = $elements;
        let offset = $offset;
        if elements.len() > offset {
            $binary_value *= 2;
            if elements[offset] == b'.' {
                $binary_value += 0;
            } else if elements[offset] == b'-' {
                $binary_value += 1;
            } else {
                panic!("Unexpected element");
            }
        }
    };
}

macro_rules! to_script {
    ($(#[$outer:meta])* $array_name:ident, $function_name:ident, $($elements:expr => $character:expr),+ $(,)? ) => {
        const $array_name: [char; 256] = {
            let mut x = ['\0'; 256];
            $(
                let elements = $elements.as_bytes();
                let mut binary_value = 1;
                if elements.len() > 8 { panic!("Too many elements"); }
                element_to_binary_digit!(binary_value, elements, 7);
                element_to_binary_digit!(binary_value, elements, 6);
                element_to_binary_digit!(binary_value, elements, 5);
                element_to_binary_digit!(binary_value, elements, 4);
                element_to_binary_digit!(binary_value, elements, 3);
                element_to_binary_digit!(binary_value, elements, 2);
                element_to_binary_digit!(binary_value, elements, 1);
                element_to_binary_digit!(binary_value, elements, 0);
                if x[binary_value as usize] != '\0' {
                    panic!("Conflict between binary values");
                }
                x[binary_value as usize] = $character;
            )+
            x[0] = ' ';
            x
        };
        $(#[$outer])*
        ///
        /// This function should not be called directly, but passed as an argument to either
        /// [decode_stream][crate::decode_stream] or [decode_string][crate::decode_string].
        ///
        /// The `elements` parameter represents up to 7 Morse code elements. There is one bit per
        /// element (0 for dot, 1 for dash), and a leading bit do detect the number of elements.
        pub fn $function_name(elements: u8) -> char {
            $array_name[elements as usize]
        }
    };
}

to_script! {
    /// Mapping from Morse code to Latin text
    TO_STANDARD,
    to_standard,
    // NOTE: Mappings are sorted like a complete binary tree in array representation. In other
    // words, they are sorted by length, then in lexicographic order. For lengths up to 5, all
    // possible combinations of Morse symbols are listed.
    //
    // This is the same order as in
    // The ARRL handbook for the Radio Amateur (1985), 19-20 https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n433/mode/2up

    // One element
    "." => 'E',
    "-" => 'T',

    // Two elements
    ".." => 'I',
    ".-" => 'A',
    "-." => 'N',
    "--" => 'M',

    // Three elements
    "..." => 'S',
    "..-" => 'U',
    ".-." => 'R',
    ".--" => 'W',
    "-.." => 'D',
    "-.-" => 'K',
    "--." => 'G',
    "---" => 'O',

    // Four elements
    "...." => 'H',
    "...-" => 'V',
    "..-." => 'F',
    "..--" => 'Ü',
    ".-.." => 'L',
    ".-.-" => 'Ä',
    ".--." => 'P',
    ".---" => 'J',
    "-..." => 'B',
    "-..-" => 'X',
    "-.-." => 'C',
    "-.--" => 'Y',
    "--.." => 'Z',
    "--.-" => 'Q',
    "---." => 'Ö',
    // "----" => 'CH', // no single Unicode codepoint for this digraph

    // Five elements
    "....." => '5',
    "....-" => '4',
    // "...-." => '<SN>', // prosign
    "...--" => '3',
    "..-.." => 'É', // or 'È' or 'Ę'
    // "..-.-" => '<INT>', // prosign
    "..--." => '!', // ??
    "..---" => '2',
    // ".-..." => '<AS>',
    // ".-..-" => '<AU>',
    ".-.-." => '+', // or '<AR>'
    // .-.--
    // .--..
    ".--.-" => 'Á', // or 'À' or 'Ä' or 'Å'
    // .---.
    ".----" => '1',
    "-...." => '6',
    "-...-" => '=', // or '<BT>'
    "-..-." => '/', // or '<DN>'
    // -..--
    "-.-.." => 'Ç',
    // "-.-.-" => '<KA>', // prosign
    "-.--." => '(', // or '<KN>'
    // -.---
    "--..." => '7',
    "--..-" => 'Ź', // or 'Ż', probably, but not listed in ARRL handbook
    // --.-. // NOTE: missing from the ARRL handbook
    "--.--" => 'Ñ', // NOTE: typo in the ARRL handbook incorrectly shows it encoded as --..--
    "---.." => '8',
    // ---.-
    "----." => '9',
    "-----" => '0',

    // Six elements (only mapped)
    // "....--" => '<HM>', // prosign
    // "...-.-" => '<SK>', // prosign
    // "..-..-" => '<IX>', // prosign
    "..--.." => '?', // or '<IMI>'
    "..--.-" => '_', // or '<IQ>'
    ".-..-." => '"', // or '<AF>'
    ".-.-.-" => '.', // or '<AAA>'
    ".--.-." => '@', // becaome official after 1985 ARRL handbook was published
    ".----." => '\'', // or '<WG>'
    "-....-" => '-', // or '<DU>'
    "-.-.-." => ';', // or '<KR>'
    "-.--.-" => ')', // or '<KK>'
    "--..--" => ',', // or '<MIM>'
    "---..." => ':', // or '<OS>'

    // Seven elements (only mapped)
    "...-..-" => '$', // or '<SX>'
    ".-----." => '`',
}

to_script! {
    /// Mapping from Morse code to Greek text
    TO_GREEK,
    to_greek,
    // Greek Morse code
    // Wikipedia: The Greek Morse code alphabet is very similar to the
    //            Latin alphabet. It uses one extra letter for Greek
    //            letter Χ and no longer uses the codes for Latin
    //            letters "J ", "U" and "V".
    // https://en.wikipedia.org/wiki/Morse_code_for_non-Latin_alphabets#Greek
    ".-" => 'Α',
    "-..." => 'Β',
    "--." => 'Γ',
    "-.." => 'Δ',
    "." => 'Ε',
    "--.." => 'Ζ',
    "...." => 'Η',
    "-.-." => 'Θ',
    ".." => 'Ι',
    "-.-" => 'Κ',
    ".-.." => 'Λ',
    "--" => 'Μ',
    "-." => 'Ν',
    "-..-" => 'Ξ',
    "---" => 'Ο',
    ".--." => 'Π',
    ".-." => 'Ρ',
    "..." => 'Σ',
    "-" => 'Τ',
    "-.--" => 'Υ',
    "..-." => 'Φ',
    "----" => 'Χ',
    "--.-" => 'Ψ',
}

to_script! {
    /// Mapping from Morse code to Russian (Cyrillic) text
    TO_RUSSIAN,
    to_russian,
    // Russian Morse code for Cyrillic
    // https://en.wikipedia.org/wiki/Russian_Morse_code (1857)
    // Полное собрание законов Российской Империи. Собрание Второе
    // These are listed in the order of the Wikipedia page (alphabetical
    // order of the corresponding latin script character)
    ".-" => 'А',   // a
    "-..." => 'Б', // be
    ".--" => 'В',  // ve
    "--." => 'Г',  // ghe
    "-.." => 'Д',  // de
    "." => 'Е',    // ie
    "...-" => 'Ж', // zhe
    "--.." => 'З', // ze
    ".." => 'И',   // i
    ".---" => 'Й', // short i
    "-.-" => 'К',  // ka
    ".-.." => 'Л', // el
    "--" => 'М',   // em
    "-." => 'Н',   // en
    "---" => 'О',  // o
    ".--." => 'П', // pe
    ".-." => 'Р',  // er
    "..." => 'С',  // es
    "-" => 'Т',
    "..-" => 'У',   // u
    "..-." => 'Ф',  // ef
    "...." => 'Х',  // ha
    "-.-." => 'Ц',  // tse
    "---." => 'Ч',  // che
    "----" => 'Ш',  // sha
    "--.-" => 'Щ',  // shcha
    "-..-" => 'Ъ',  // hard sign
    "-.--" => 'Ы',  // yeru
    "..-.." => 'Ѣ', // yat  in Wikipedia article and in Russian law document
    "..--" => 'Ю',  // yu
    ".-.-" => 'Я',  // ya
}

to_script! {
    /// Mapping from Morse code to Japanese (Katakana) text
    TO_JAPANESE,
    to_japanese,
    // Wabun code for Japanese, tnx JE1TRV
    // https://en.wikipedia.org/wiki/Wabun_code
    // https://www.rfcafe.com/references/qst/japanese-morse-telegraph-code-sep-1942-qst.htm (1942)
    // https://web.archive.org/web/20220129114408/https://elaws.e-gov.go.jp/data/325M50080000017_20200622_502M60000008061/pict/S25F30901000017-001.pdf (1945?)
    // 1. Kanas without any diacritics (dakuten or handakuten)
    ".-" => 'イ',    // i
    ".-.-" => 'ロ',  // ro
    "-..." => 'ハ',  // ha
    "-.-." => 'ニ',  // ni
    "-.." => 'ホ',   // ho
    "." => 'ヘ',     // he
    "..-.." => 'ト', // to
    "..-." => 'チ',  // ti
    "--." => 'リ',   // ri
    "...." => 'ヌ',  // nu
    "-.--." => 'ル', // ru
    ".---" => 'ヲ',  // wo
    "-.-" => 'ワ',   // wa
    ".-.." => 'カ',  // ka
    "--" => 'ヨ',    // yo
    "-." => 'タ',    // ta
    "---" => 'レ',   // re
    "---." => 'ソ',  // so
    ".--." => 'ツ',  // tu
    "--.-" => 'ネ',  // ne
    ".-." => 'ナ',   // na
    "..." => 'ラ',   // ra
    "-" => 'ム',     // mu
    "..-" => 'ウ',   // u
    ".-..-" => 'ヰ', // yi
    "..--" => 'ノ',  // no
    ".-..." => 'オ', // o
    "...-" => 'ク',  // ku
    ".--" => 'ヤ',   // ya
    "-..-" => 'マ',  // ma
    "-.--" => 'ケ',  // ke
    "--.." => 'フ',  // fu
    "----" => 'コ',  // ko
    "-.---" => 'エ', // e
    ".-.--" => 'テ', // te
    "--.--" => 'ア', // a
    "-.-.-" => 'サ', // sa
    "-.-.." => 'キ', // ki
    "-..--" => 'ユ', // yu
    "-...-" => 'メ', // me
    "..-.-" => 'ミ', // mi
    "--.-." => 'シ', // si
    ".--.." => 'ヱ', // ye
    "--..-" => 'ヒ', // hi
    "-..-." => 'モ', // mo
    ".---." => 'セ', // se
    "---.-" => 'ス', // su
    ".-.-." => 'ン', // n
    ".." => '゛',    // dakuten modifier
    "..--." => '゜', // handakuten modifier
    ".--.-" => 'ー',
    "-.--.-" => '（',
    ".-..-." => '）',
    ".-.-.-" => '、',
    ".-.-.." => '。',
}

to_script! {
    /// Mapping from Morse code to Korean (Hangul) text
    TO_KOREAN,
    to_korean,
    // SKATS for Korean
    // The ARRL handbook for the radio amateur, 19-3 (1985)
    // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
    ".-.." => 'ㄱ',    // kiyeok
    "..-." => 'ㄴ',    // nieun
    "-..." => 'ㄷ',    // tikeut
    "...-" => 'ㄹ',    // rieul
    "--" => 'ㅁ',      // mieum
    ".--" => 'ㅂ',     // pieup
    "--." => 'ㅅ',     // sios
    "-.-" => 'ㅇ',     // ieung
    ".--." => 'ㅈ',    // cieuc
    "-.-." => 'ㅊ',    // chieuch
    "-..-" => 'ㅋ',    // khieukh
    "--.." => 'ㅌ',    // thieuth
    "---" => 'ㅍ',     // phieuph
    ".---" => 'ㅎ',    // hieuh
    "." => 'ㅏ',       // a
    "--.-" => 'ㅐ',    // ae
    ".." => 'ㅑ',      // ya
    //".. ..-" => 'ㅒ',  // yae
    "-" => 'ㅓ',       // eo
    "-.--" => 'ㅔ',    // e
    "..." => 'ㅕ',     // yeo
    //"... ..-" => 'ㅖ', // ye
    ".-" => 'ㅗ',      // o
    "-." => 'ㅛ',      // yo
    "...." => 'ㅜ',    // u
    ".-." => 'ㅠ',     // yu
    "-.." => 'ㅡ',     // eu
    "..-" => 'ㅣ',     // i
}

to_script! {
    /// Mapping from Morse code to Hebrew text
    TO_HEBREW,
    to_hebrew,
    // Hebrew
    // The ARRL handbook for the radio amateur, 19-3 (1985)
    // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
    ".-" => 'א',   // alef
    "-..." => 'ב', // bet
    "--." => 'ג',  // gimel
    "-.." => 'ד',  // dalet
    "---" => 'ה',  // he
    "." => 'ו',    // vav
    "--.." => 'ז', // zayin
    "...." => 'ח', // chet
    "..-" => 'ט',  // tet
    ".." => 'י',   // yod
    "-.-" => 'כ',  // kaf
    ".-.." => 'ל', // lamed
    "--" => 'מ',   // mem
    "-." => 'נ',   // nun
    "-.-." => 'ס', // samekh
    ".---" => 'ע', // ayin
    ".--." => 'פ', // pe
    ".--" => 'צ',  // tsadi
    "--.-" => 'ק', // qof
    ".-." => 'ר',  // resh
    "..." => 'ש',  // dotless shin
    "-" => 'ת',    // dotless tav
}

to_script! {
    /// Mapping from Morse code to Arabic text
    TO_ARABIC,
    to_arabic,
    // Arabic
    // The ARRL handbook for the radio amateur, 19-3 (1985)
    // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
    // Unicode points were copied from “Isolated form”, and names from “Letter name” in
    // https://en.wikipedia.org/wiki/Arabic_alphabet#Table_of_basic_letters
    // TODO: add contextual forms
    ".-" => 'ا',    // ʾalif
    "-..." => 'ب',  // bāʾ/bah
    "-" => 'ت',     // tāʾ/tah
    "-.-." => 'ث',  // thāʾ/thah
    ".---" => 'ج',  // jīm
    "...." => 'ح',  // ḥāʾ/ḥah
    "---" => 'خ',   // khāʾ/khah
    "-.." => 'د',   // dāl/dāʾ/dah
    "--.." => 'ذ',  // dhāl/dhāʾ/dhah
    ".-." => 'ر',   // rāʾ/rah
    "---." => 'ز',  // zāy/zayn/zāʾ/zah
    "..." => 'س',   // sīn
    "----" => 'ش',  // shīn
    "-..-" => 'ص',  // ṣād
    "...-" => 'ض',  // ḍād/ḍāʾ/ḍah
    "..-" => 'ط',   // ṭāʾ/ṭah
    "-.--" => 'ظ',  // ẓāʾ/ẓah
    ".-.-" => 'ع',  // ʿayn
    "--." => 'غ',   // ghayn
    "..-." => 'ف',  // fāʾ/fah
    "--.-" => 'ق',  // qāf
    "-.-" => 'ڪ',   // kāf/kāʾ/kah
    ".-.." => 'ل',  // lām
    "--" => 'م',    // mīm
    "-." => 'ن',    // nūn
    "..-.." => 'ه', // hāʾ/hah
    ".--" => 'و',   // wāw
    ".." => 'ے',    // yāʾ/yah
    //".-...-" => 'لا', // lām-alif (ligature)
    // other characters without a reference
    "." => 'ء', // hamzah
}
