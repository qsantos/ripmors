use crate::encode_ascii_mapping::ASCII_TO_BYTES;

macro_rules! from_unicode {
    ($c:expr, $($letter:pat => $elements:literal),+ $(,)? ) => {
        match $c {
            $(
                $letter => {
                    let (elements, len) = match $elements.len() {
                        1 => (concat!($elements, " \0\0\0\0\0\0").as_bytes(), 2),
                        2 => (concat!($elements, " \0\0\0\0\0").as_bytes(), 3),
                        3 => (concat!($elements, " \0\0\0\0").as_bytes(), 4),
                        4 => (concat!($elements, " \0\0\0").as_bytes(), 5),
                        5 => (concat!($elements, " \0\0").as_bytes(), 6),
                        6 => (concat!($elements, " \0").as_bytes(), 7),
                        _ => (concat!($elements, " ").as_bytes(), $elements.len() + 1),
                    };
                    (elements, len)
                }
            )+
            _ => (b"\0\0\0\0\0\0\0\0", 0)
        }
    };
}

pub fn from_unicode(c: char) -> (&'static [u8], usize) {
    if c.is_ascii() {
        return ASCII_TO_BYTES[c as usize];
    }
    from_unicode! {
        c,
        // non-English Latin extensions (from https://en.wikipedia.org/wiki/Morse_code#Letters,_numbers,_punctuation,_prosigns_for_Morse_code_and_non-Latin_variants)
        // The ARRL handbook for the Radio Amateur (1985), 19-2 https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n415/mode/2up
        // The ARRL handbook for the Radio Amateur (1985), 19-20 https://archive.org/details/arrlhandbookforr0000unse_w7j4/page/n433/mode/2up
        // Uppercase | Lowercase
        'Á' | 'á' => ".--.-",
        'À' | 'à' => ".--.-",
        'Ä' | 'ä' => ".-.-",
        'Å' | 'å' => ".--.-",
        'Ą' | 'ą' => ".-.-",
        'Æ' | 'æ' => ".-.-",
        'Ć' | 'ć' => "-.-..",
        'Ĉ' | 'ĉ' => "-.-..",
        'Ç' | 'ç' => "-.-..",
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
        'Ñ' | 'ñ' => "--.--", // NOTE: typo in the ARRL handbook incorrectly shows it encoded as --..--
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
        // not in Wikipedia article nor in ARRL handbook
        'ß' => "...--..",
        'Œ' | 'œ' => "---.",
        'Ì' | 'ì' => ".---.",

        // mappings to standard characters
        '“' | '”' | '«' | '»' => ".-..-.", // English & French quotes (1.1.3), mapped to "
        '×' => "-..-",                     // Multiplication sign (1.1.3), mapped to x
        '‰' => "----- -..-. ----- -----",  // Per mille (3.3.1), mapped to 0/00
        '′' => ".----.",                   // Minute (3.5.1), mapped to '
        '″' => ".----. .----.",            // Second (3.5.1), mapped to ''
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
        'ı' => "..",
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
        '－' => ".--.-",
        'ー' => ".--.-",
        '（' => "-.--.-",
        '）' => ".-..-.",
        '、' => ".-.-.-",
        '。' => ".-.-..",

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
        //'בּ' => "-...", // dotted bet (multiple Unicode codepoints)
        'ג' => "--.", // gimel
        //'גּ' => "--.",   // dotted gimel (multiple Unicode codepoints)
        'ד' => "-..", // dalet
        //'דּ' => "-..",   // dotted dalet (multiple Unicode codepoints)
        'ה' => "---",  // he
        'ו' => ".",    // vav
        'ז' => "--..", // zayin
        'ח' => "....", // chet
        'ט' => "..-",  // tet
        'י' => "..",   // yod
        'ך' => "-.-",  // final kaf
        //'ךּ' => "-.-",   // dotted final kaf (multiple Unicode codepoints)
        'כ' => "-.-", // kaf
        //'כּ' => "-.-",   // dotted kaf (multiple Unicode codepoints)
        'ל' => ".-..", // lamed
        'ם' => "--",   // final mem
        'מ' => "--",   // mem
        'ן' => "-.",   // final nun
        'נ' => "-.",   // nun
        'ס' => "-.-.", // samekh
        'ע' => ".---", // ayin
        'ף' => ".--.", // final pe
        //'ףּ' => ".--.", // final pe (multiple Unicode codepoints)
        'פ' => ".--.", // pe
        //'פּ' => ".--.", // dotted pe (multiple Unicode codepoints)
        'ץ' => ".--",  // final tsadi
        'צ' => ".--",  // tsadi
        'ק' => "--.-", // qof
        'ר' => ".-.",  // resh
        'ש' => "...",  // dotless shin
        //'שׁ' => "...",   // right-dotted shin (multiple Unicode codepoints)
        //'שׂ' => "...",   // left-dotted shin (multiple Unicode codepoints)
        'ת' => "-", // dotless tav
        //'תּ' => "-",     // dotted tav (multiple Unicode codepoints)

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
        //'لا' => ".-...-", // lām-alif (ligature) (multiple Unicode codepoints)
        // other characters without a reference
        'ء' => ".", // hamzah
    }
}
