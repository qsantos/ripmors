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

pub const fn from_unicode(c: char) -> (&'static [u8], usize) {
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
        'Ð' | 'ð' => "..--.",
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

/// Encoding of a non-ASCII character, as stored in the lookup tables below.
///
/// The first `N - 1` bytes hold the Morse code (followed by the letter space), padded with zeroes;
/// the last byte holds its length. This lets the encoder copy the whole entry with a single store
/// of `N` bytes and advance by the length; the extra bytes written past the length are overwritten
/// by the next character or truncated.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Entry<const N: usize>(pub [u8; N]);

impl<const N: usize> Entry<N> {
    const EMPTY: Self = Self([0; N]);

    const fn for_code_point(cp: u32) -> Self {
        let Some(c) = char::from_u32(cp) else {
            return Self::EMPTY;
        };
        let (elements, len) = from_unicode(c);
        assert!(len < N, "encoding does not fit in a table entry");
        let mut entry = [0; N];
        let mut i = 0;
        while i < len {
            entry[i] = elements[i];
            i += 1;
        }
        entry[N - 1] = len as u8;
        Self(entry)
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.0[N - 1] as usize
    }
}

/// Table entry for characters encoded with two bytes in UTF-8, whose Morse code is at most 10 bytes
/// long (e.g. `Љ`).
pub type Entry2 = Entry<16>;
/// Table entry for characters encoded with three bytes in UTF-8, whose Morse code is at most 24
/// bytes long (`‰`).
pub type Entry3 = Entry<32>;

/// Aligns tables to cache lines, so that entries are naturally aligned.
#[repr(C, align(64))]
pub struct Aligned<T>(pub T);

/// Encodings of the characters encoded with two bytes in UTF-8 (U+0080 to U+07FF), indexed directly
/// by code point. Most non-ASCII scripts with a Morse code (Latin extensions, Greek, Cyrillic,
/// Hebrew, Arabic) live in this range. Entries below U+0080 are unused (and empty).
static UTF8_2_TABLE: Aligned<[Entry2; 0x800]> = {
    let mut table = [Entry2::EMPTY; 0x800];
    let mut cp = 0x80;
    while cp < 0x800 {
        table[cp] = Entry2::for_code_point(cp as u32);
        cp += 1;
    }
    Aligned(table)
};

/// Number of code points per page of [`UTF8_3_PAGES`]; it matches the payload of a UTF-8
/// continuation byte, so the offset in the page is just the low 6 bits of the last byte.
const PAGE_SIZE: usize = 64;
/// Number of pages for U+0000 to U+FFFF (only the pages from U+0800, encoded with three bytes in
/// UTF-8, are used).
const PAGE_COUNT: usize = 0x10000 / PAGE_SIZE;

/// For each page, whether it contains at least one encodable character (only pages from U+0800
/// are considered).
const NON_EMPTY_PAGES: [bool; PAGE_COUNT] = {
    let mut pages = [false; PAGE_COUNT];
    let mut cp = 0x800;
    while cp < 0x10000 {
        if Entry3::for_code_point(cp as u32).len() != 0 {
            pages[cp / PAGE_SIZE] = true;
        }
        cp += 1;
    }
    pages
};

/// Number of distinct pages in [`UTF8_3_PAGES`] (the first one is the shared empty page).
const DISTINCT_PAGES: usize = {
    let mut count = 1;
    let mut page = 0;
    while page < PAGE_COUNT {
        if NON_EMPTY_PAGES[page] {
            count += 1;
        }
        page += 1;
    }
    count
};

/// First level of the lookup table for characters encoded with three bytes in UTF-8: index in
/// [`UTF8_3_PAGES`] of the page containing a code point, indexed by `code_point / 64`.
static UTF8_3_PAGE_INDEX: [u8; PAGE_COUNT] = {
    assert!(DISTINCT_PAGES <= 256);
    let mut index = [0; PAGE_COUNT];
    let mut next = 1;
    let mut page = 0;
    while page < PAGE_COUNT {
        if NON_EMPTY_PAGES[page] {
            index[page] = next as u8;
            next += 1;
        }
        page += 1;
    }
    index
};

/// Second level of the lookup table for characters encoded with three bytes in UTF-8: pages of
/// [`PAGE_SIZE`] entries, indexed by `code_point % 64`. The first page is empty.
static UTF8_3_PAGES: Aligned<[[Entry3; PAGE_SIZE]; DISTINCT_PAGES]> = {
    let mut pages = [[Entry3::EMPTY; PAGE_SIZE]; DISTINCT_PAGES];
    let mut next = 1;
    let mut page = 0;
    while page < PAGE_COUNT {
        if NON_EMPTY_PAGES[page] {
            let mut offset = 0;
            while offset < PAGE_SIZE {
                pages[next][offset] = Entry3::for_code_point((page * PAGE_SIZE + offset) as u32);
                offset += 1;
            }
            next += 1;
        }
        page += 1;
    }
    Aligned(pages)
};

// NOTE: we avoid masking the marker bits of the UTF-8 bytes in the functions below; instead, we
// subtract them as a constant, which the compiler folds into the address computation.

/// Look up the encoding of a character from its two-byte UTF-8 sequence `b0 b1`, without decoding
/// it.
///
/// # Safety
///
/// `b0 b1` must be a valid two-byte UTF-8 sequence.
#[inline(always)]
pub unsafe fn lookup_utf8_2(b0: u8, b1: u8) -> &'static Entry2 {
    // `(b0 << 6) + b1 == 0x3080 + code_point`, with `0x80 <= code_point < 0x800`
    let cp = ((b0 as usize) << 6) + b1 as usize - 0x3080;
    // SAFETY: the caller guarantees that this is a valid two-byte sequence, so `cp < 0x800`
    unsafe { UTF8_2_TABLE.0.get_unchecked(cp) }
}

/// Look up the encoding of a character from its three-byte UTF-8 sequence `b0 b1 b2`, without
/// decoding it.
///
/// # Safety
///
/// `b0 b1 b2` must be a valid three-byte UTF-8 sequence.
#[inline(always)]
pub unsafe fn lookup_utf8_3(b0: u8, b1: u8, b2: u8) -> &'static Entry3 {
    // `(b0 << 6) + b1 == 0x3880 + code_point / 64`, with `code_point < 0x10000`
    let page = ((b0 as usize) << 6) + b1 as usize - 0x3880;
    // SAFETY: the caller guarantees that this is a valid three-byte sequence, so `page < 1024`,
    // and `UTF8_3_PAGE_INDEX` only contains valid indices in `UTF8_3_PAGES`; `b2` is a
    // continuation byte, so `0 <= b2 - 0x80 < 64`
    unsafe {
        let page = *UTF8_3_PAGE_INDEX.get_unchecked(page) as usize;
        UTF8_3_PAGES
            .0
            .get_unchecked(page)
            .get_unchecked(b2 as usize - 0x80)
    }
}

#[test]
#[cfg_attr(miri, ignore)] // checks every code point: too slow, and has no unsafe code
fn test_lookup_tables_match_from_unicode() {
    for c in (0x80..=0x10ffff).filter_map(char::from_u32) {
        let (bytes, len) = from_unicode(c);
        let expected = &bytes[..len];
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf).as_bytes();
        let actual = match *s {
            [b0, b1] => {
                // SAFETY: `s` is a valid two-byte UTF-8 sequence
                let entry = unsafe { lookup_utf8_2(b0, b1) };
                assert!(entry.0[entry.len()..15].iter().all(|&b| b == 0));
                &entry.0[..entry.len()]
            }
            [b0, b1, b2] => {
                // SAFETY: `s` is a valid three-byte UTF-8 sequence
                let entry = unsafe { lookup_utf8_3(b0, b1, b2) };
                assert!(entry.0[entry.len()..31].iter().all(|&b| b == 0));
                &entry.0[..entry.len()]
            }
            // characters outside the BMP have no Morse code
            _ => &[],
        };
        assert_eq!(actual, expected, "{c:?} (U+{:04X})", c as u32);
    }
    assert!(UTF8_3_PAGES.0[0].iter().all(|e| *e == Entry3::EMPTY));
}
