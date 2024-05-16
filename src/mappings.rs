// International Morse code, as per ITU-R M.1677-1

pub fn ascii_to_morse(c: u8) -> &'static str {
    match c {
        // NOTE: look for `=> "",` for missing characters
        b' ' => "/",

        b'!' => "..--.",  // non standard
        b'"' => ".-..-.", // Straight quotes (1.1.3)
        b'#' => "",
        b'$' => "...-..-",           // non standard
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
        b'<' => "",
        b'=' => "-...-", // Double hyphen (1.1.3)
        b'>' => "",
        b'?' => "..--..", // Question mark (1.1.3)
        b'@' => ".--.-.", // Commercial at (1.1.3)

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

        // 1.1.1. Letters (Latins cript)
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

pub fn unicode_to_morse(c: char) -> &'static str {
    if c.is_ascii() {
        return ascii_to_morse(c as u8);
    }
    match c {
        // Dot-less i (see https://en.wikipedia.org/wiki/Dotless_I)
        'ı' => ascii_to_morse(b'i'),

        '“' | '”' | '«' | '»' => ascii_to_morse(b'"'), // English & French quotes (1.1.3)
        '×' => ascii_to_morse(b'x'),                   // Multiplication sign (1.1.3)
        '‰' => "----- -..-. ----- -----",              // Mapped to "0/00" (3.3.1)
        '′' => ascii_to_morse(b'\''),                  // Minute (3.5.1), mapped to "'"
        '″' => ".----. .----.",                        // Second (3.5.1), mapped to "''"

        // non-Latin extensions (from https://en.wikipedia.org/wiki/Morse_code#Letters,_numbers,_punctuation,_prosigns_for_Morse_code_and_non-Latin_variants)
        // Uppercase | Lowercase
        'À' | 'à' => ".--.-",
        'Ä' | 'ä' => ".-.-",
        'Å' | 'å' => ".--.-",
        'Ą' | 'ą' => ".-.-",
        'Æ' | 'æ' => ".-.-",
        'Ć' | 'ć' => "-.-..",
        'Ĉ' | 'ĉ' => "-.-..",
        'Ç' | 'ç' => "-.-..",
        // 'CH' | 'ch' => "----",
        'Đ' | 'đ' => "..-..",
        'Ð' | 'ð' => "..-.",
        'É' | 'é' => "..-..",
        'È' | 'è' => ".-..-",
        'Ę' | 'ę' => "..-..",
        'Ĝ' | 'ĝ' => "--.-.",
        'Ĥ' | 'ĥ' => "----",
        'Ĵ' | 'ĵ' => ".---.",
        'Ł' | 'ł' => ".-..-",
        'Ń' | 'ń' => "--.--",
        'Ñ' | 'ñ' => "--.--",
        'Ó' | 'ó' => "---.",
        'Ö' | 'ö' => "---.",
        'Ø' | 'ø' => "---.",
        'Ś' | 'ś' => "...-...",
        'Ŝ' | 'ŝ' => "...-.",
        'Š' | 'š' => "----",
        'Þ' | 'þ' => ".--..",
        'Ü' | 'ü' => "..--",
        'Ŭ' | 'ŭ' => "..--",
        'Ź' | 'ź' => "--..-.",
        'Ż' | 'ż' => "--..-.",

        // other characters without a reference
        // Uppercase | Lowercase
        /* "SS" */
        'ß' => "...--..",
        'Á' | 'á' => ".--.-",
        'Œ' | 'œ' => "---.",
        'Ì' | 'ì' => ".---.",

        // mapping of other Latin characters with diacritics to standard characters
        // Uppercase | Lowercase
        // A
        'Â' | 'â' => ".-",
        'Ã' | 'ã' => ".-",
        'Ā' | 'ā' => ".-",
        'Ă' | 'ă' => ".-",
        // C
        'Ċ' | 'ċ' => "-.-.",
        'Č' | 'č' => "-.-.",
        // D
        'Ď' | 'ď' => "-..",
        // E
        'Ê' | 'ê' => ".",
        'Ë' | 'ë' => ".",
        'Ē' | 'ē' => ".",
        'Ĕ' | 'ĕ' => ".",
        'Ė' | 'ė' => ".",
        'Ě' | 'ě' => ".",
        // G
        'Ğ' | 'ğ' => "--.",
        'Ġ' | 'ġ' => "--.",
        'Ģ' | 'ģ' => "--.",
        // H
        'Ħ' | 'ħ' => "....",
        // I
        'Í' | 'í' => "..",
        'Î' | 'î' => "..",
        'Ï' | 'ï' => "..",
        'Ĩ' | 'ĩ' => "..",
        'Ī' | 'ī' => "..",
        'Ĭ' | 'ĭ' => "..",
        'Į' | 'į' => "..",
        // IJ
        'Ĳ' | 'ĳ' => ".. .---",
        // K
        'Ķ' | 'ķ' => "-.-",
        /* NA */ 'ĸ' => "-.-",
        // L
        'Ĺ' | 'ĺ' => ".-..",
        'Ļ' | 'ļ' => ".-..",
        'Ľ' | 'ľ' => ".-..",
        'Ŀ' | 'ŀ' => ".-..",
        // N
        'Ņ' | 'ņ' => "-.",
        'Ň' | 'ň' => "-.",
        /* "ʼN" */ 'ŉ' => "-.",
        'Ŋ' | 'ŋ' => "-.",
        // O
        'Ò' | 'ò' => "---",
        'Ô' | 'ô' => "---",
        'Õ' | 'õ' => "---",
        'Ō' | 'ō' => "---",
        'Ŏ' | 'ŏ' => "---",
        'Ő' | 'ő' => "---",
        // R
        'Ŕ' | 'ŕ' => ".-.",
        'Ŗ' | 'ŗ' => ".-.",
        'Ř' | 'ř' => ".-.",
        // S
        'Ş' | 'ş' => "...",
        // T
        'Ţ' | 'ţ' => "-",
        'Ť' | 'ť' => "-",
        'Ŧ' | 'ŧ' => "-",
        // U
        'Ù' | 'ù' => "..-",
        'Ú' | 'ú' => "..-",
        'Û' | 'û' => "..-",
        'Ũ' | 'ũ' => "..-",
        'Ū' | 'ū' => "..-",
        'Ů' | 'ů' => "..-",
        'Ű' | 'ű' => "..-",
        'Ų' | 'ų' => "..-",
        // W
        'Ŵ' | 'ŵ' => ".--",
        // Y
        'Ý' | 'ý' => "-.--",
        'Ŷ' | 'ŷ' => "-.--",
        'Ÿ' | 'ÿ' => "-.--",
        // Z
        'Ž' | 'ž' => "--..",

        // Greek Morse code
        // Wikipedia: The Greek Morse code alphabet is very similar to the
        //            Latin alphabet. It uses one extra letter for Greek
        //            letter Χ and no longer uses the codes for Latin
        //            letters "J", "U" and "V".
        // https://en.wikipedia.org/wiki/Morse_code_for_non-Latin_alphabets#Greek
        // Uppercase | Lowercase | Lowercase in word-final position
        'Α' | 'α' => ".-",
        'Β' | 'β' => "-...",
        'Γ' | 'γ' => "--.",
        'Δ' | 'δ' => "-..",
        'Ε' | 'ε' => ".",
        'Ζ' | 'ζ' => "--..",
        'Η' | 'η' => "....",
        'Θ' | 'θ' => "-.-.",
        'Ι' | 'ι' => "..",
        'Κ' | 'κ' => "-.-",
        'Λ' | 'λ' => ".-..",
        'Μ' | 'μ' => "--",
        'Ν' | 'ν' => "-.",
        'Ξ' | 'ξ' => "-..-",
        'Ο' | 'ο' => "---",
        'Π' | 'π' => ".--.",
        'Ρ' | 'ρ' => ".-.",
        'Σ' | 'σ' | 'ς' => "...",
        'Τ' | 'τ' => "-",
        'Υ' | 'υ' => "-.--",
        'Φ' | 'φ' => "..-.",
        'Χ' | 'χ' => "----",
        'Ψ' | 'ψ' => "--.-",

        // Russian Morse code for Cyrillic
        // https://en.wikipedia.org/wiki/Russian_Morse_code (1857)
        // Полное собрание законов Российской Империи. Собрание Второе
        // These are listed in the order of the Wikipedia page (alphabetical
        // order of the corresponding latin script character)
        // Uppercase | Lowercase
        'А' | 'а' => ".-",   // a
        'Б' | 'б' => "-...", // be
        'В' | 'в' => ".--",  // ve
        'Г' | 'г' => "--.",  // ghe
        'Д' | 'д' => "-..",  // de
        'Е' | 'е' => ".",    // ie
        'Ж' | 'ж' => "...-", // zhe
        'З' | 'з' => "--..", // ze
        'И' | 'и' => "..",   // i
        'Й' | 'й' => ".---", // short i
        'К' | 'к' => "-.-",  // ka
        'Л' | 'л' => ".-..", // el
        'М' | 'м' => "--",   // em
        'Н' | 'н' => "-.",   // en
        'О' | 'о' => "---",  // o
        'П' | 'п' => ".--.", // pe
        'Р' | 'р' => ".-.",  // er
        'С' | 'с' => "...",  // es
        'Т' | 'т' => "-",
        'У' | 'у' => "..-",   // u
        'Ф' | 'ф' => "..-.",  // ef
        'Х' | 'х' => "....",  // ha
        'Ц' | 'ц' => "-.-.",  // tse
        'Ч' | 'ч' => "---.",  // che
        'Ш' | 'ш' => "----",  // sha
        'Щ' | 'щ' => "--.-",  // shcha
        'Ъ' | 'ъ' => "-..-",  // hard sign
        'Ы' | 'ы' => "-.--",  // yeru
        'Ь' | 'ь' => "-..-",  // soft sign
        'Ѣ' | 'ѣ' => "..-..", // yat, in Wikipedia article and in Russian law document
        'Э' | 'э' => "..-..", // e, in Wikipedia article only
        'Ю' | 'ю' => "..--",  // yu
        'Я' | 'я' => ".-.-",  // ya

        // mapping of other Cyrillic characters to standard ones
        'Ѐ' | 'ѐ' => ".",    // ie with grave
        'Ё' | 'ё' => ".",    // io
        'Є' | 'є' => ".",    // ukrainian  ie
        'І' | 'і' => "..",   // byelorussian-ukrainian i
        'Ї' | 'ї' => "..",   // yi
        'Ј' | 'ј' => ".---", // je
        'Ћ' | 'ћ' => "-.-.", // tshe
        'Ѝ' | 'ѝ' => "..",   // i with grave
        'Ў' | 'ў' => "..-",  // short u

        // phonetic decomposition of other Cyrillic characters
        'Ђ' | 'ђ' => "-.. .---",  // dje
        'Ѓ' | 'ѓ' => "--. .---",  // gje
        'Ѕ' | 'ѕ' => "-.. --..",  // dze
        'Љ' | 'љ' => ".-.. .---", // lje
        'Њ' | 'њ' => "-. .---",   // nje
        'Ќ' | 'ќ' => "-.- .---",  // kje
        'Џ' | 'џ' => "-.. --..",  // dzhe

        // Wabun code for Japanese, tnx JE1TRV
        // https://en.wikipedia.org/wiki/Wabun_code
        // https://www.rfcafe.com/references/qst/japanese-morse-telegraph-code-sep-1942-qst.htm (1942)
        // https://web.archive.org/web/20220129114408/https://elaws.e-gov.go.jp/data/325M50080000017_20200622_502M60000008061/pict/S25F30901000017-001.pdf (1945?)
        // 1. Kanas without any diacritics (dakuten or handakuten)
        // Katakana    Hiragana
        'イ' | 'い' => ".-",    // i
        'ロ' | 'ろ' => ".-.-",  // ro
        'ハ' | 'は' => "-...",  // ha
        'ニ' | 'に' => "-.-.",  // ni
        'ホ' | 'ほ' => "-..",   // ho
        'ヘ' | 'へ' => ".",     // he
        'ト' | 'と' => "..-..", // to
        'チ' | 'ち' => "..-.",  // ti
        'リ' | 'り' => "--.",   // ri
        'ヌ' | 'ぬ' => "....",  // nu
        'ル' | 'る' => "-.--.", // ru
        'ヲ' | 'を' => ".---",  // wo
        'ワ' | 'わ' => "-.-",   // wa
        'カ' | 'か' => ".-..",  // ka
        'ヨ' | 'よ' => "--",    // yo
        'ョ' | 'ょ' => "--",    // yo
        'タ' | 'た' => "-.",    // ta
        'レ' | 'れ' => "---",   // re
        'ソ' | 'そ' => "---.",  // so
        'ツ' | 'つ' => ".--.",  // tu
        'ッ' | 'っ' => ".--.",  // tu
        'ネ' | 'ね' => "--.-",  // ne
        'ナ' | 'な' => ".-.",   // na
        'ラ' | 'ら' => "...",   // ra
        'ム' | 'む' => "-",     // mu
        'ウ' | 'う' => "..-",   // u
        'ヰ' | 'ゐ' => ".-..-", // yi
        'ノ' | 'の' => "..--",  // no
        'オ' | 'お' => ".-...", // o
        'ク' | 'く' => "...-",  // ku
        'ヤ' | 'や' => ".--",   // ya
        'ャ' | 'ゃ' => ".--",   // ya
        'マ' | 'ま' => "-..-",  // ma
        'ケ' | 'け' => "-.--",  // ke
        'フ' | 'ふ' => "--..",  // fu
        'コ' | 'こ' => "----",  // ko
        'エ' | 'え' => "-.---", // e
        'テ' | 'て' => ".-.--", // te
        'ア' | 'あ' => "--.--", // a
        'サ' | 'さ' => "-.-.-", // sa
        'キ' | 'き' => "-.-..", // ki
        'ユ' | 'ゆ' => "-..--", // yu
        'ュ' | 'ゅ' => "-..--", // yu
        'メ' | 'め' => "-...-", // me
        'ミ' | 'み' => "..-.-", // mi
        'シ' | 'し' => "--.-.", // si
        'ヱ' | 'ゑ' => ".--..", // ye
        'ヒ' | 'ひ' => "--..-", // hi
        'モ' | 'も' => "-..-.", // mo
        'セ' | 'せ' => ".---.", // se
        'ス' | 'す' => "---.-", // su
        'ン' | 'ん' => ".-.-.", // n
        // 2. Kanas with dakuten
        '゛' => "..", // Dakuten modifier
        // Katakanas        Hiraganas
        'ガ' | 'が' => ".-.. ..",  // ga
        'ギ' | 'ぎ' => "-.-.. ..", // gi
        'グ' | 'ぐ' => "...- ..",  // gu
        'ゲ' | 'げ' => "-.-- ..",  // ge
        'ゴ' | 'ご' => "---- ..",  // go
        'ザ' | 'ざ' => "-.-.- ..", // za
        'ジ' | 'じ' => "--.-. ..", // zi
        'ズ' | 'ず' => "---.- ..", // zu
        'ゼ' | 'ぜ' => ".---. ..", // ze
        'ゾ' | 'ぞ' => "---. ..",  // zo
        'ダ' | 'だ' => "-. ..",    // da
        'ヂ' | 'ぢ' => "..-. ..",  // di
        'ヅ' | 'づ' => ".--. ..",  // du
        'デ' | 'で' => ".-.-- ..", // de
        'ド' | 'ど' => "..-.. ..", // do
        'バ' | 'ば' => "-... ..",  // ba
        'ビ' | 'び' => "--..- ..", // bi
        'ブ' | 'ぶ' => "--.. ..",  // bu
        'ベ' | 'べ' => ". ..",     // be
        'ボ' | 'ぼ' => "-.. ..",   // bo
        // 3. Kanas with handakuten
        '゜' => "..--.", // Handakuten modifier
        // Katakanas        Hiraganas
        'パ' | 'ぱ' => "-... ..--.",  // pa
        'ピ' | 'ぴ' => "--..- ..--.", // pi
        'プ' | 'ぷ' => "--.. ..--.",  // pu
        'ペ' | 'ぺ' => ". ..--.",     // pe
        'ポ' | 'ぽ' => "-.. ..--.",   // po
        // 4. Other characters in the Wabun code
        '－' => ".--.-",  // -
        'ー' => ".--.-",  // -
        '（' => "-.--.-", // (
        '）' => ".-..-.", // )
        '、' => ".-.-.-", // .
        '」' => ".-.-..", // \n

        // SKATS for Korean
        // The ARRL handbook for the radio amateur, 19-3 (1985)
        // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        'ㄱ' => ".-..",    // kiyeok
        'ㄴ' => "..-.",    // nieun
        'ㄷ' => "-...",    // tikeut
        'ㄹ' => "...-",    // rieul
        'ㅁ' => "--",      // mieum
        'ㅂ' => ".--",     // pieup
        'ㅅ' => "--.",     // sios
        'ㅇ' => "-.-",     // ieung
        'ㅈ' => ".--.",    // cieuc
        'ㅊ' => "-.-.",    // chieuch
        'ㅋ' => "-..-",    // khieukh
        'ㅌ' => "--..",    // thieuth
        'ㅍ' => "---",     // phieuph
        'ㅎ' => ".---",    // hieuh
        'ㅏ' => ".",       // a
        'ㅐ' => "--.-",    // ae
        'ㅑ' => "..",      // ya
        'ㅒ' => ".. ..-",  // yae
        'ㅓ' => "-",       // eo
        'ㅔ' => "-.--",    // e
        'ㅕ' => "...",     // yeo
        'ㅖ' => "... ..-", // ye
        'ㅗ' => ".-",      // o
        'ㅛ' => "-.",      // yo
        'ㅜ' => "....",    // u
        'ㅠ' => ".-.",     // yu
        'ㅡ' => "-..",     // eu
        'ㅣ' => "..-",     // i

        // Hebrew
        // The ARRL handbook for the radio amateur, 19-3 (1985)
        // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        'א' => ".-",   // alef
        'ב' => "-...", // bet
        //'בּ' => "-...", // dotted bet
        'ג' => "--.", // gimel
        //'גּ' => "--.",   // dotted gimel
        'ד' => "-..", // dalet
        //'דּ' => "-..",   // dotted dalet
        'ה' => "---",  // he
        'ו' => ".",    // vav
        'ז' => "--..", // zayin
        'ח' => "....", // chet
        'ט' => "..-",  // tet
        'י' => "..",   // yod
        'ך' => "-.-",  // final kaf
        //'ךּ' => "-.-",   // dotted final kaf
        'כ' => "-.-", // kaf
        //'כּ' => "-.-",   // dotted kaf
        'ל' => ".-..", // lamed
        'ם' => "--",   // final mem
        'מ' => "--",   // mem
        'ן' => "-.",   // final nun
        'נ' => "-.",   // nun
        'ס' => "-.-.", // samekh
        'ע' => ".---", // ayin
        'ף' => ".--.", // final pe
        //'ףּ' => ".--.", // final pe
        'פ' => ".--.", // pe
        //'פּ' => ".--.", // dotted pe
        'ץ' => ".--",  // final tsadi
        'צ' => ".--",  // tsadi
        'ק' => "--.-", // qof
        'ר' => ".-.",  // resh
        'ש' => "...",  // dotless shin
        //'שׁ' => "...",   // right-dotted shin
        //'שׂ' => "...",   // left-dotted shin
        'ת' => "-", // dotless tav
        //'תּ' => "-",     // dotted tav

        // Arabic
        // The ARRL handbook for the radio amateur, 19-3 (1985)
        // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        // Unicode points were copied from “Isolated form”, and names from “Letter name” in
        // https://en.wikipedia.org/wiki/Arabic_alphabet#Table_of_basic_letters
        // TODO: add contextual forms
        'ا' => ".-",    // ʾalif
        'ب' => "-...",  // bāʾ/bah
        'ت' => "-",     // tāʾ/tah
        'ث' => "-.-.",  // thāʾ/thah
        'ج' => ".---",  // jīm
        'ح' => "....",  // ḥāʾ/ḥah
        'خ' => "---",   // khāʾ/khah
        'د' => "-..",   // dāl/dāʾ/dah
        'ذ' => "--..",  // dhāl/dhāʾ/dhah
        'ر' => ".-.",   // rāʾ/rah
        'ز' => "---.",  // zāy/zayn/zāʾ/zah
        'س' => "...",   // sīn
        'ش' => "----",  // shīn
        'ص' => "-..-",  // ṣād
        'ض' => "...-",  // ḍād/ḍāʾ/ḍah
        'ط' => "..-",   // ṭāʾ/ṭah
        'ظ' => "-.--",  // ẓāʾ/ẓah
        'ع' => ".-.-",  // ʿayn
        'غ' => "--.",   // ghayn
        'ف' => "..-.",  // fāʾ/fah
        'ق' => "--.-",  // qāf
        'ڪ' => "-.-",   // kāf/kāʾ/kah
        'ك' => "-.-",   // kāf/kāʾ/kah
        'ل' => ".-..",  // lām
        'م' => "--",    // mīm
        'ن' => "-.",    // nūn
        'ه' => "..-..", // hāʾ/hah
        'و' => ".--",   // wāw
        'ے' => "..",    // yāʾ/yah
        'ي' => "..",    // yāʾ/yah
        //'لا' => ".-...-", // lām-alif (ligature)
        // other characters without a reference
        'ء' => ".", // hamzah

        _ => "",
    }
}
