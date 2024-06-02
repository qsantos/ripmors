// International Morse code, as per ITU-R M.1677-1

pub fn ascii_to_morse(c: char) -> &'static str {
    match c {
        // NOTE: look for `=> "",` for missing characters
        '\t' => "\t",
        '\n' => "\n",
        '\r' => "\r",
        ' ' => "/",

        '!' => "..--.",  // non standard
        '"' => ".-..-.", // Straight quotes (1.1.3)
        '#' => "",
        '$' => "...-..-",           // non standard
        '%' => "----- -..-. -----", // Mapped to "0/0" (3.3.1)
        '&' => ". ...",             // non standard: mapped to "es"

        '\'' => ".----.", // Apostrophe (1.1.3)
        '(' => "-.--.",   // Left-hand bracket (parenthesis) (1.1.3)
        ')' => "-.--.-",  // Right-hand bracket (parenthesis) (1.1.3)
        '*' => "-..-",    // Multiplication sign (same as letter X) (1.1.3)
        '+' => ".-.-.",   // Cross or addition sign (1.1.3)
        ',' => "--..--",  // Comma (1.1.3)
        '-' => "-....-",  // Hyphen (1.1.3)
        '.' => ".-.-.-",  // Full stop (period) (1.1.3)
        '/' => "-..-.",   // Fraction bar or division sign (1.1.3)

        // 1.1.2. Figures (Hindu-Arab digits)
        '0' => "-----",
        '1' => ".----",
        '2' => "..---",
        '3' => "...--",
        '4' => "....-",
        '5' => ".....",
        '6' => "-....",
        '7' => "--...",
        '8' => "---..",
        '9' => "----.",

        ':' => "---...", // Colon r division sign (1.1.3)
        ';' => "-.-.-.", // non standard
        '<' => "-.--.",  // non standard: mapped to (
        '=' => "-...-",  // Double hyphen (1.1.3)
        '>' => "-.--.-", // non standard: mapped to )
        '?' => "..--..", // Question mark (1.1.3)
        '@' => ".--.-.", // Commercial at (1.1.3)

        // 1.1.1. Letters (Latin script)
        // Uppercase
        'A' => ".-",
        'B' => "-...",
        'C' => "-.-.",
        'D' => "-..",
        'E' => ".",
        'F' => "..-.",
        'G' => "--.",
        'H' => "....",
        'I' => "..",
        'J' => ".---",
        'K' => "-.-",
        'L' => ".-..",
        'M' => "--",
        'N' => "-.",
        'O' => "---",
        'P' => ".--.",
        'Q' => "--.-",
        'R' => ".-.",
        'S' => "...",
        'T' => "-",
        'U' => "..-",
        'V' => "...-",
        'W' => ".--",
        'X' => "-..-",
        'Y' => "-.--",
        'Z' => "--..",

        '[' => "-.--.",  // non standard: mapped to (
        '\\' => "-..-.", // non standard: mapped to /
        ']' => "-.--.-", // non standard: mapped to )
        '^' => "",
        '_' => "..--.-",  // non standard
        '`' => ".-----.", // non standard

        // 1.1.1. Letters (Latin script)
        // Lowercase
        'a' => ".-",
        'b' => "-...",
        'c' => "-.-.",
        'd' => "-..",
        'e' => ".",
        'f' => "..-.",
        'g' => "--.",
        'h' => "....",
        'i' => "..",
        'j' => ".---",
        'k' => "-.-",
        'l' => ".-..",
        'm' => "--",
        'n' => "-.",
        'o' => "---",
        'p' => ".--.",
        'q' => "--.-",
        'r' => ".-.",
        's' => "...",
        't' => "-",
        'u' => "..-",
        'v' => "...-",
        'w' => ".--",
        'x' => "-..-",
        'y' => "-.--",
        'z' => "--..",

        '{' => "-.--.",  // non standard: mapped to (
        '|' => "-..-.",  // non standard: mapped to /
        '}' => "-.--.-", // non standard: mapped to )
        '~' => "",

        _ => "", // 33 control characters
    }
}

pub fn standard_to_morse(c: char) -> &'static str {
    if c.is_ascii() {
        return ascii_to_morse(c);
    }
    match c {
        // Dot-less i (see https://en.wikipedia.org/wiki/Dotless_I)
        'ı' => ascii_to_morse('i'),

        '“' | '”' | '«' | '»' => ascii_to_morse('"'), // English & French quotes (1.1.3)
        '×' => ascii_to_morse('x'),                   // Multiplication sign (1.1.3)
        '‰' => "----- -..-. ----- -----",             // Mapped to "0/00" (3.3.1)
        '′' => ascii_to_morse('\''),                  // Minute (3.5.1), mapped to "'"
        '″' => ".----. .----.",                       // Second (3.5.1), mapped to "''"

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
        'Ι' | 'ι' | 'Ί' | 'ί' => "..",
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

        _ => "?",
    }
}

pub fn morse_to_standard(s: &[u8]) -> char {
    match s {
        // Space
        b"/" => ' ',

        // NOTE: Mappings are sorted like a complete binary tree in array representation. In other
        // words, they are sorted by length, then in lexicographic order. For lengths up to 5, all
        // possible combinations of Morse symbols are listed.

        // One element
        b"." => 'E',
        b"-" => 'T',

        // Two elements
        b".." => 'I',
        b".-" => 'A',
        b"-." => 'N',
        b"--" => 'M',

        // Three elements
        b"..." => 'S',
        b"..-" => 'U',
        b".-." => 'R',
        b".--" => 'W',
        b"-.." => 'D',
        b"-.-" => 'K',
        b"--." => 'G',
        b"---" => 'O',

        // Four elements
        b"...." => 'H',
        b"...-" => 'V',
        b"..-." => 'F',
        // ..--
        b".-.." => 'L',
        // .-.-
        b".--." => 'P',
        b".---" => 'J',
        b"-..." => 'B',
        b"-..-" => 'X',
        b"-.-." => 'C',
        b"-.--" => 'Y',
        b"--.." => 'Z',
        b"--.-" => 'Q',
        // ---.
        // ----

        // Five elements
        b"....." => '5',
        b"....-" => '4',
        // ...-.
        b"...--" => '3',
        // ..-..
        // ..-.-
        b"..--." => '!',
        b"..---" => '2',
        // .-...
        // .-..-
        b".-.-." => '+',
        // .-.--
        // .--..
        // .--.-
        // .---.
        b".----" => '1',
        b"-...." => '6',
        b"-...-" => '=',
        b"-..-." => '/',
        // -..--
        // -.-..
        // -.-.-
        b"-.--." => '(',
        // -.---
        b"--..." => '7',
        // --..-
        // --.-.
        // --.--
        b"---.." => '8',
        // ---.-
        b"----." => '9',
        b"-----" => '0',

        // Six elements (only mapped)
        b"..--.." => '?',
        b"..--.-" => '_',
        b".-..-." => '"',
        b".-.-.-" => '.',
        b".--.-." => '@',
        b".----." => '\'',
        b".-----." => '`',
        b"-....-" => '-',
        b"-.-.-." => ';',
        b"-.--.-" => ')',
        b"--..--" => ',',
        b"---..." => ':',

        // Seven elements (only mapped)
        b"...-..-" => '$',

        _ => '\0',
    }
}

pub fn morse_to_greek(c: &[u8]) -> char {
    match c {
        // Greek Morse code
        // Wikipedia: The Greek Morse code alphabet is very similar to the
        //            Latin alphabet. It uses one extra letter for Greek
        //            letter Χ and no longer uses the codes for Latin
        //            letters "J ", "U" and "V".
        // https://en.wikipedia.org/wiki/Morse_code_for_non-Latin_alphabets#Greek
        b".-" => 'Α',
        b"-..." => 'Β',
        b"--." => 'Γ',
        b"-.." => 'Δ',
        b"." => 'Ε',
        b"--.." => 'Ζ',
        b"...." => 'Η',
        b"-.-." => 'Θ',
        b".." => 'Ι',
        b"-.-" => 'Κ',
        b".-.." => 'Λ',
        b"--" => 'Μ',
        b"-." => 'Ν',
        b"-..-" => 'Ξ',
        b"---" => 'Ο',
        b".--." => 'Π',
        b".-." => 'Ρ',
        b"..." => 'Σ',
        b"-" => 'Τ',
        b"-.--" => 'Υ',
        b"..-." => 'Φ',
        b"----" => 'Χ',
        b"--.-" => 'Ψ',
        _ => '\0',
    }
}

pub fn morse_to_russian(c: &[u8]) -> char {
    match c {
        // Russian Morse code for Cyrillic
        // https://en.wikipedia.org/wiki/Russian_Morse_code (1857)
        // Полное собрание законов Российской Империи. Собрание Второе
        // These are listed in the order of the Wikipedia page (alphabetical
        // order of the corresponding latin script character)
        b".-" => 'А',   // a
        b"-..." => 'Б', // be
        b".--" => 'В',  // ve
        b"--." => 'Г',  // ghe
        b"-.." => 'Д',  // de
        b"." => 'Е',    // ie
        b"...-" => 'Ж', // zhe
        b"--.." => 'З', // ze
        b".." => 'И',   // i
        b".---" => 'Й', // short i
        b"-.-" => 'К',  // ka
        b".-.." => 'Л', // el
        b"--" => 'М',   // em
        b"-." => 'Н',   // en
        b"---" => 'О',  // o
        b".--." => 'П', // pe
        b".-." => 'Р',  // er
        b"..." => 'С',  // es
        b"-" => 'Т',
        b"..-" => 'У',   // u
        b"..-." => 'Ф',  // ef
        b"...." => 'Х',  // ha
        b"-.-." => 'Ц',  // tse
        b"---." => 'Ч',  // che
        b"----" => 'Ш',  // sha
        b"--.-" => 'Щ',  // shcha
        b"-..-" => 'Ъ',  // hard sign
        b"-.--" => 'Ы',  // yeru
        b"..-.." => 'Ѣ', // yat  in Wikipedia article and in Russian law document
        b"..--" => 'Ю',  // yu
        b".-.-" => 'Я',  // ya
        _ => '\0',
    }
}

pub fn morse_to_japanese(c: &[u8]) -> char {
    match c {
        // Wabun code for Japanese, tnx JE1TRV
        // https://en.wikipedia.org/wiki/Wabun_code
        // https://www.rfcafe.com/references/qst/japanese-morse-telegraph-code-sep-1942-qst.htm (1942)
        // https://web.archive.org/web/20220129114408/https://elaws.e-gov.go.jp/data/325M50080000017_20200622_502M60000008061/pict/S25F30901000017-001.pdf (1945?)
        // 1. Kanas without any diacritics (dakuten or handakuten)
        b".-" => 'イ',    // i
        b".-.-" => 'ロ',  // ro
        b"-..." => 'ハ',  // ha
        b"-.-." => 'ニ',  // ni
        b"-.." => 'ホ',   // ho
        b"." => 'ヘ',     // he
        b"..-.." => 'ト', // to
        b"..-." => 'チ',  // ti
        b"--." => 'リ',   // ri
        b"...." => 'ヌ',  // nu
        b"-.--." => 'ル', // ru
        b".---" => 'ヲ',  // wo
        b"-.-" => 'ワ',   // wa
        b".-.." => 'カ',  // ka
        b"--" => 'ヨ',    // yo
        b"-." => 'タ',    // ta
        b"---" => 'レ',   // re
        b"---." => 'ソ',  // so
        b".--." => 'ツ',  // tu
        b"--.-" => 'ネ',  // ne
        b".-." => 'ナ',   // na
        b"..." => 'ラ',   // ra
        b"-" => 'ム',     // mu
        b"..-" => 'ウ',   // u
        b".-..-" => 'ヰ', // yi
        b"..--" => 'ノ',  // no
        b".-..." => 'オ', // o
        b"...-" => 'ク',  // ku
        b".--" => 'ヤ',   // ya
        b"-..-" => 'マ',  // ma
        b"-.--" => 'ケ',  // ke
        b"--.." => 'フ',  // fu
        b"----" => 'コ',  // ko
        b"-.---" => 'エ', // e
        b".-.--" => 'テ', // te
        b"--.--" => 'ア', // a
        b"-.-.-" => 'サ', // sa
        b"-.-.." => 'キ', // ki
        b"-..--" => 'ユ', // yu
        b"-...-" => 'メ', // me
        b"..-.-" => 'ミ', // mi
        b"--.-." => 'シ', // si
        b".--.." => 'ヱ', // ye
        b"--..-" => 'ヒ', // hi
        b"-..-." => 'モ', // mo
        b".---." => 'セ', // se
        b"---.-" => 'ス', // su
        b".-.-." => 'ン', // n
        // 2. kanas with dakuten
        b".." => '゛',       // dakuten modifier
        b".-.. .." => 'ガ',  // ga
        b"-.-.. .." => 'ギ', // gi
        b"...- .." => 'グ',  // gu
        b"-.-- .." => 'ゲ',  // ge
        b"---- .." => 'ゴ',  // go
        b"-.-.- .." => 'ザ', // za
        b"--.-. .." => 'ジ', // zi
        b"---.- .." => 'ズ', // zu
        b".---. .." => 'ゼ', // ze
        b"---. .." => 'ゾ',  // zo
        b"-. .." => 'ダ',    // da
        b"..-. .." => 'ヂ',  // di
        b".--. .." => 'ヅ',  // du
        b".-.-- .." => 'デ', // de
        b"..-.. .." => 'ド', // do
        b"-... .." => 'バ',  // ba
        b"--..- .." => 'ビ', // bi
        b"--.. .." => 'ブ',  // bu
        b". .." => 'ベ',     // be
        b"-.. .." => 'ボ',   // bo
        // 3. kanas with handakuten
        b"..--." => '゜',       // handakuten modifier
        b"-... ..--." => 'パ',  // pa
        b"--..- ..--." => 'ピ', // pi
        b"--.. ..--." => 'プ',  // pu
        b". ..--." => 'ペ',     // pe
        b"-.. ..--." => 'ポ',   // po
        b".--.-" => 'ー',       // -
        b"-.--.-" => '（',      // (
        b".-..-." => '）',      // )
        b".-.-.-" => '、',      // .
        b".-.-.." => '」',      // \n
        _ => '\0',
    }
}

pub fn morse_to_korean(c: &[u8]) -> char {
    match c {
        // SKATS for Korean
        // The ARRL handbook for the radio amateur, 19-3 (1985)
        // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        b".-.." => 'ㄱ',    // kiyeok
        b"..-." => 'ㄴ',    // nieun
        b"-..." => 'ㄷ',    // tikeut
        b"...-" => 'ㄹ',    // rieul
        b"--" => 'ㅁ',      // mieum
        b".--" => 'ㅂ',     // pieup
        b"--." => 'ㅅ',     // sios
        b"-.-" => 'ㅇ',     // ieung
        b".--." => 'ㅈ',    // cieuc
        b"-.-." => 'ㅊ',    // chieuch
        b"-..-" => 'ㅋ',    // khieukh
        b"--.." => 'ㅌ',    // thieuth
        b"---" => 'ㅍ',     // phieuph
        b".---" => 'ㅎ',    // hieuh
        b"." => 'ㅏ',       // a
        b"--.-" => 'ㅐ',    // ae
        b".." => 'ㅑ',      // ya
        b".. ..-" => 'ㅒ',  // yae
        b"-" => 'ㅓ',       // eo
        b"-.--" => 'ㅔ',    // e
        b"..." => 'ㅕ',     // yeo
        b"... ..-" => 'ㅖ', // ye
        b".-" => 'ㅗ',      // o
        b"-." => 'ㅛ',      // yo
        b"...." => 'ㅜ',    // u
        b".-." => 'ㅠ',     // yu
        b"-.." => 'ㅡ',     // eu
        b"..-" => 'ㅣ',     // i
        _ => '\0',
    }
}

pub fn morse_to_hebrew(c: &[u8]) -> char {
    match c {
        // Hebrew
        // The ARRL handbook for the radio amateur, 19-3 (1985)
        // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        b".-" => 'א',   // alef
        b"-..." => 'ב', // bet
        b"--." => 'ג',  // gimel
        b"-.." => 'ד',  // dalet
        b"---" => 'ה',  // he
        b"." => 'ו',    // vav
        b"--.." => 'ז', // zayin
        b"...." => 'ח', // chet
        b"..-" => 'ט',  // tet
        b".." => 'י',   // yod
        b"-.-" => 'כ',  // kaf
        b".-.." => 'ל', // lamed
        b"--" => 'מ',   // mem
        b"-." => 'נ',   // nun
        b"-.-." => 'ס', // samekh
        b".---" => 'ע', // ayin
        b".--." => 'פ', // pe
        b".--" => 'צ',  // tsadi
        b"--.-" => 'ק', // qof
        b".-." => 'ר',  // resh
        b"..." => 'ש',  // dotless shin
        b"-" => 'ת',    // dotless tav
        _ => '\0',
    }
}

pub fn morse_to_arabic(c: &[u8]) -> char {
    match c {
        // Arabic
        // The ARRL handbook for the radio amateur, 19-3 (1985)
        // https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        // Unicode points were copied from “Isolated form”, and names from “Letter name” in
        // https://en.wikipedia.org/wiki/Arabic_alphabet#Table_of_basic_letters
        // TODO: add contextual forms
        b".-" => 'ا',    // ʾalif
        b"-..." => 'ب',  // bāʾ/bah
        b"-" => 'ت',     // tāʾ/tah
        b"-.-." => 'ث',  // thāʾ/thah
        b".---" => 'ج',  // jīm
        b"...." => 'ح',  // ḥāʾ/ḥah
        b"---" => 'خ',   // khāʾ/khah
        b"-.." => 'د',   // dāl/dāʾ/dah
        b"--.." => 'ذ',  // dhāl/dhāʾ/dhah
        b".-." => 'ر',   // rāʾ/rah
        b"---." => 'ز',  // zāy/zayn/zāʾ/zah
        b"..." => 'س',   // sīn
        b"----" => 'ش',  // shīn
        b"-..-" => 'ص',  // ṣād
        b"...-" => 'ض',  // ḍād/ḍāʾ/ḍah
        b"..-" => 'ط',   // ṭāʾ/ṭah
        b"-.--" => 'ظ',  // ẓāʾ/ẓah
        b".-.-" => 'ع',  // ʿayn
        b"--." => 'غ',   // ghayn
        b"..-." => 'ف',  // fāʾ/fah
        b"--.-" => 'ق',  // qāf
        b"-.-" => 'ڪ',   // kāf/kāʾ/kah
        b".-.." => 'ل',  // lām
        b"--" => 'م',    // mīm
        b"-." => 'ن',    // nūn
        b"..-.." => 'ه', // hāʾ/hah
        b".--" => 'و',   // wāw
        b".." => 'ے',    // yāʾ/yah
        //".-...-" => 'لا', // lām-alif (ligature)
        // other characters without a reference
        b"." => 'ء', // hamzah
        _ => '\0',
    }
}
