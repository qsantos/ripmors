use std::io::{BufWriter, Read, Write};

use crate::encode_ascii_mapping::ASCII_TO_QWORD;

pub fn ascii_encode_to_writer<W: Write>(
    writer: &mut W,
    s: &[u8],
    need_separator: &mut bool,
    buf: &mut [u8; 1 << 15],
) -> Result<(), std::io::Error> {
    let mut cur = 0;
    if *need_separator {
        buf[cur] = b' ';
        cur += 1;
    }
    for chunk in s.chunks(1 << 10) {
        for c in chunk {
            let (bytes, len) = ASCII_TO_QWORD[*c as usize];
            if len == 0 {
            } else if len <= 8 {
                if (*c == b'\t' || *c == b'\n' || *c == b'\r') && cur > 0 && buf[cur - 1] == b' ' {
                    cur -= 1;
                }
                unsafe {
                    let dst = buf.as_mut_ptr().add(cur) as *mut u64;
                    dst.write_unaligned(bytes);
                }
            } else {
                // handle only ASCII character encoded as more than 7 elements + space
                assert_eq!(*c, b'%');
                buf[cur..cur + 18].copy_from_slice(b"----- -..-. ----- ");
            }
            cur += len;
        }
        // flush buffer
        if cur >= buf.len() - (1 << 10) * 18 {
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

pub fn encode_string_ascii(s: &str) -> String {
    let mut writer = BufWriter::new(Vec::new());
    let mut buf = [0u8; 1 << 15];
    ascii_encode_to_writer(&mut writer, s.as_bytes(), &mut false, &mut buf).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

pub fn encode_stream_ascii<R: Read, W: Write>(i: &mut R, o: &mut W) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut need_separator = false;
    let mut buf = [0u8; 1 << 15];
    loop {
        let n = i.read(&mut input_buf).unwrap();
        if n == 0 {
            break;
        }
        ascii_encode_to_writer(o, &input_buf[..n], &mut need_separator, &mut buf).unwrap();
    }
}

#[test]
fn test_ascii_encode_simple() {
    assert_eq!(encode_string_ascii("PARIS"), ".--. .- .-. .. ...");
    assert_eq!(
        encode_string_ascii("Hello, World!"),
        ".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."
    );
    assert_eq!(
        encode_string_ascii("one line\nand  another\tline"),
        "--- -. . / .-.. .. -. .\n.- -. -.. / / .- -. --- - .... . .-.\t.-.. .. -. ."
    );
}

// short enough to run with Miri
#[test]
fn test_ascii_encode_random_short() {
    use rand::{distributions::Standard, Rng};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1024)
        .map(|c| c as char)
        .collect();
    encode_string_ascii(&data);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_ascii_encode_random_large() {
    use rand::{distributions::Standard, Rng};
    let data: String = rand::thread_rng()
        .sample_iter::<u8, _>(Standard)
        .take(1048576)
        .map(|c| c as char)
        .collect();
    encode_string_ascii(&data);
}