use std::io::{BufWriter, Read, Write};

use crate::encode_ascii::ascii_encode_to_writer;
use crate::encode_ascii_mapping::ASCII_TO_MORSE2;
use crate::encode_unicode_mapping::unicode_to_morse;

pub fn unicode_encode_to_writer<W: Write>(
    writer: &mut W,
    s: &str,
    need_separator: &mut bool,
    buf: &mut [u8; 1 << 15],
) -> Result<(), std::io::Error> {
    if s.is_ascii() {
        return ascii_encode_to_writer(writer, s.as_bytes(), need_separator, buf);
    }
    let mut cur = 0;
    if *need_separator {
        buf[cur] = b' ';
        cur += 1;
    }
    for c in s.chars() {
        if c.is_ascii() {
            let (bytes, len) = ASCII_TO_MORSE2[c as usize];
            if len == 0 {
            } else if len <= 8 {
                if (c == '\t' || c == '\n' || c == '\r') && cur > 0 && buf[cur - 1] == b' ' {
                    cur -= 1;
                }
                unsafe {
                    let dst = buf.as_mut_ptr().add(cur) as *mut u64;
                    dst.write_unaligned(bytes);
                }
            } else {
                // handle only ASCII character encoded as more than 7 elements + space
                assert_eq!(c, '%');
                buf[cur..cur + 18].copy_from_slice(b"----- -..-. ----- ");
            }
            cur += len;
        } else {
            let (bytes, len) = unicode_to_morse(c);
            if len == 0 {
            } else if len <= 8 {
                let buf8 = unsafe { buf.get_unchecked_mut(cur..cur + 8) };
                let bytes8 = unsafe { &*(bytes.as_ptr() as *const [u8; 8]) };
                buf8.copy_from_slice(bytes8);
            } else {
                buf[cur..cur + len].copy_from_slice(bytes);
            }
            cur += len;
        }
        // flush buffer
        if cur >= buf.len() - 25 {
            if buf[cur - 1] == b' ' {
                cur -= 1;
                writer.write_all(&buf[..cur])?;
                buf[0] = b' ';
                cur = 1;
            } else {
                writer.write_all(&buf[..cur])?;
                cur = 0;
            }
        }
    }
    // flush buffer
    if cur != 0 {
        if buf[cur - 1] == b' ' {
            cur -= 1;
            *need_separator = true;
        } else {
            *need_separator = false;
        }
        writer.write_all(&buf[..cur])?;
    }
    Ok(())
}

pub fn unicode_encode_to_string(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    let mut buf = [0u8; 1 << 15];
    unicode_encode_to_writer(&mut writer, s, &mut false, &mut buf).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

pub fn encode_stream_unicode<R: Read, W: Write>(i: &mut R, o: &mut W) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut need_separator = false;
    let mut buf = [0u8; 1 << 15];
    loop {
        let n = i.read(&mut input_buf[bytes_available..]).unwrap();
        if n == 0 {
            break;
        }
        bytes_available += n;
        let s = match std::str::from_utf8(&input_buf[..bytes_available]) {
            Ok(s) => s,
            Err(e) => {
                let bytes_decoded = e.valid_up_to();
                unsafe { std::str::from_utf8_unchecked(&input_buf[..bytes_decoded]) }
            }
        };
        unicode_encode_to_writer(o, s, &mut need_separator, &mut buf).unwrap();
        let bytes_decoded = s.as_bytes().len();
        input_buf.copy_within(bytes_decoded..bytes_available, 0);
        bytes_available -= bytes_decoded;
    }
}

#[test]
fn test_unicode_encode() {
    assert_eq!(
        unicode_encode_to_string("télégraphie"),
        "- ..-.. .-.. ..-.. --. .-. .- .--. .... .. ."
    );
    assert_eq!(
        unicode_encode_to_string("でんしん"),
        ".-.-- .. .-.-. --.-. .-.-."
    );
    assert_eq!(unicode_encode_to_string("تلغراف"), "- .-.. --. .-. .- ..-.");
    assert_eq!(
        unicode_encode_to_string("телеграфия"),
        "- . .-.. . --. .-. .- ..-. .. .-.-"
    );
    assert_eq!(
        unicode_encode_to_string("τηλεγραφία"),
        "- .... .-.. . --. .-. .- ..-. .. .-"
    );
    assert_eq!(
        unicode_encode_to_string("one line\nand  another\tline"),
        "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
    );
}

// short enough to run with Miri
#[test]
fn test_unicode_encode_random_short() {
    use rand::{distributions::Standard, Rng};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1024)
        .map(|c| c as char)
        .collect();
    unicode_encode_to_string(&data);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_unicode_encode_random_large() {
    use rand::{distributions::Standard, Rng};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1048576)
        .map(|c| c as char)
        .collect();
    unicode_encode_to_string(&data);
}
