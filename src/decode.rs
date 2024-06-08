use std::io::{BufWriter, Read, Write};

#[inline(always)] // prefer inline to avoid reloading constants in registers
unsafe fn morse_to_binary(bytes: *const u8, len: usize) -> u8 {
    // Interpret next 8 bytes as u64
    let a = unsafe { (bytes as *const u64).read_unaligned() };
    // Only keep the LSB of each byte
    let b = 0x0101010101010101;
    let a = a & b;
    // Pack the bits together
    let a = a.wrapping_mul(0x102040810204080) >> 56;
    // Truncate to len lowest significant bits
    let a = a & !(0xff << len);
    // Add a leading one to distinguish e.g. - from .-, ..-, ...- and ....-
    // NOTE: we use "b" instead of "1" to avoid having to load another immediate in a register.
    // Since the lowest byte of b is 0x01 and we only keep the last byte, it works.
    let a = a | (b << len);
    // Done
    a as u8
}

fn morse_to_binary_safe(bytes: &[u8], len: usize) -> u8 {
    if len + 8 <= bytes.len() {
        return unsafe { morse_to_binary(bytes.as_ptr(), len) };
    }
    let mut ret = 1;
    for byte in bytes[..len].iter().rev() {
        ret *= 2;
        ret |= byte & 1;
    }
    ret
}

pub fn decode_buffer<W: Write>(
    writer: &mut W,
    s: &[u8],
    char_decode: fn(u8) -> char,
    buf: &mut [char; 1 << 15],
) -> Result<usize, std::io::Error> {
    let mut cur = 0;
    let mut chunk_start = 0;
    let last_seven_bytes = s.len().saturating_sub(7);
    for i in 0..last_seven_bytes {
        let c = s[i];
        if c <= b' ' {
            let binary = unsafe { morse_to_binary(s.as_ptr().add(chunk_start), i - chunk_start) };
            let decoded = char_decode(binary);
            if decoded != '\0' {
                buf[cur] = decoded;
                cur += 1;
            }
            chunk_start = i + 1;
            if c != b' ' {
                buf[cur] = c as char;
                cur += 1;
            }
        } else if c == b'/' {
            buf[cur] = ' ';
            cur += 1;
            chunk_start = i + 1;
        }
        // flush buffer
        // NOTE: we may write up to two character per iteration
        if cur > buf.len() - 2 {
            let decoded: String = buf[..cur].iter().collect();
            writer.write_all(decoded.as_bytes())?;
            cur = 0;
        }
    }
    for i in last_seven_bytes..s.len() {
        let c = s[i];
        if c <= b' ' {
            let binary = morse_to_binary_safe(&s[chunk_start..], i - chunk_start);
            let decoded = char_decode(binary);
            if decoded != '\0' {
                buf[cur] = decoded;
                cur += 1;
            }
            chunk_start = i + 1;
            if c != b' ' {
                buf[cur] = c as char;
                cur += 1;
            }
        } else if c == b'/' {
            buf[cur] = ' ';
            cur += 1;
            chunk_start = i + 1;
        }
    }
    // flush buffer
    if cur > 0 {
        let decoded: String = buf[..cur].iter().collect();
        writer.write_all(decoded.as_bytes())?;
    }
    Ok(chunk_start)
}

pub fn decode_buffer_end<W: Write>(
    writer: &mut W,
    s: &[u8],
    char_decode: fn(u8) -> char,
) -> Result<(), std::io::Error> {
    let mut buf = ['\0'; 1 << 15];
    let chunk_start = decode_buffer(writer, s, char_decode, &mut buf)?;
    let binary = morse_to_binary_safe(&s[chunk_start..], s.len() - chunk_start);
    let decoded = char_decode(binary);
    if decoded != '\0' {
        writer.write_all(decoded.to_string().as_bytes())?;
    }
    Ok(())
}

pub fn decode_string(s: &[u8], char_decode: fn(u8) -> char) -> String {
    let mut writer = BufWriter::new(Vec::new());
    decode_buffer_end(&mut writer, s, char_decode).unwrap();
    let vec = writer.into_inner().unwrap();
    String::from_utf8(vec).unwrap()
}

pub fn decode_stream<R: Read, W: Write>(i: &mut R, o: &mut W, char_decode: fn(u8) -> char) {
    let mut input_buf = vec![0u8; 1 << 15];
    let mut bytes_available = 0;
    let mut buf = ['\0'; 1 << 15];
    loop {
        let bytes_read = i.read(&mut input_buf[bytes_available..]).unwrap();
        if bytes_read == 0 {
            break;
        }
        bytes_available += bytes_read;

        let bytes_used =
            decode_buffer(o, &input_buf[..bytes_available], char_decode, &mut buf).unwrap();

        input_buf.copy_within(bytes_used..bytes_available, 0);
        bytes_available -= bytes_used;
    }

    if bytes_available != 0 {
        decode_buffer_end(o, &input_buf[..bytes_available], char_decode).unwrap();
    }
}

#[test]
fn test_standard_decode() {
    use crate::decode_mapping::to_standard;
    let f = |s| decode_string(s, to_standard);
    assert_eq!(f(b".--. .- .-. .. ..."), "PARIS");
    assert_eq!(
        f(b".... . .-.. .-.. --- --..-- / .-- --- .-. .-.. -.. ..--."),
        "HELLO, WORLD!",
    );
}
