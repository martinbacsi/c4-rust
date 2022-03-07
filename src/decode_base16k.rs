use std::{
    fs::File,
    io::{BufReader, Read, Write},
    mem,
};

use crate::nn_len;
use crate::nn_string::nn_str;

pub fn encode_b16k(binary_file: &str) -> (usize, Vec<u16>) {
    let f = File::open(binary_file);
    let mut reader = BufReader::new(f.unwrap());
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer);

    let mut enc: Vec<u16> = Vec::new();
    enc.reserve(buffer.len() * 6 / 5);

    let mut code: u16 = 0 as u16;
    for (i, b) in buffer.iter().enumerate() {
        let byte_value = *b as u16;
        match i % 7 {
            0 => {
                code = byte_value << 6;
            }
            1 => {
                code |= byte_value >> 2;
                code += 0x5000;
                enc.push(code);
                code = (byte_value & 3) << 12;
            }
            2 => {
                code |= byte_value << 4;
            }
            3 => {
                code |= byte_value >> 4;
                code += 0x5000;
                enc.push(code);
                code = (byte_value & 0xf) << 10;
            }
            4 => {
                code |= byte_value << 2;
            }
            5 => {
                code |= byte_value >> 6;
                code += 0x5000;
                enc.push(code);
                code = (byte_value & 0x3f) << 8;
            }
            6 => {
                code |= byte_value;
                code += 0x5000;
                enc.push(code);
                code = 0;
            }
            _ => {}
        }
    }
    if (buffer.len() % 7 > 0) {
        code += 0x5000;
        enc.push(code);
    }

    (buffer.len(), enc)
}

pub fn decode_b16k() -> Vec<u8> {
    let mut length = nn_len / 2;
    //let mut length = a.len();
    let mut i = 0;
    let mut code: usize = 0;
    let mut byte_value = 0u8;
    let mut pos = 0;
    let chars = nn_str.encode_utf16().collect::<Vec<u16>>();
    let mut out: Vec<u8> = Vec::new();
    out.reserve(length);
    while length > 0 {
        length -= 1;
        if ((1 << i) & 0x2b) != 0 {
            code = chars[pos] as usize - 0x5000;
            pos += 1;
        }

        match i % 7 {
            0 => {
                byte_value = (code >> 6) as u8;
                out.push(byte_value);
                byte_value = ((code & 0x3f) << 2) as u8;
            }
            1 => {
                byte_value |= (code >> 12) as u8;
                out.push(byte_value);
            }
            2 => {
                byte_value = ((code >> 4) & 0xff) as u8;
                out.push(byte_value);
                byte_value = ((code & 0xf) << 4) as u8;
            }
            3 => {
                byte_value |= (code >> 10) as u8;
                out.push(byte_value);
            }
            4 => {
                byte_value = ((code >> 2) & 0xff) as u8;
                out.push(byte_value);
                byte_value = ((code & 3) << 6) as u8;
            }
            5 => {
                byte_value |= (code >> 8) as u8;
                out.push(byte_value);
            }
            6 => {
                byte_value = (code & 0xff) as u8;
                out.push(byte_value);
            }
            _ => {}
        }
        i = (i + 1) % 7;
    }

    out
}

pub fn f16_to_f32(i: u16) -> f32 {
    if i & 0x7FFFu16 == 0 {
        return unsafe { mem::transmute((i as u32) << 16) };
    }
    let half_sign = (i & 0x8000u16) as u32;
    let half_exp = (i & 0x7C00u16) as u32;
    let half_man = (i & 0x03FFu16) as u32;
    if half_exp == 0x7C00u32 {
        if half_man == 0 {
            return unsafe { mem::transmute((half_sign << 16) | 0x7F80_0000u32) };
        } else {
            return unsafe {
                mem::transmute((half_sign << 16) | 0x7FC0_0000u32 | (half_man << 13))
            };
        }
    }
    let sign = half_sign << 16;
    let unbiased_exp = ((half_exp as i32) >> 10) - 15;
    if half_exp == 0 {
        let e = (half_man as u16).leading_zeros() - 6;
        let exp = (127 - 15 - e) << 23;
        let man = (half_man << (14 + e)) & 0x7F_FF_FFu32;
        return unsafe { mem::transmute(sign | exp | man) };
    }
    let exp = ((unbiased_exp + 127) as u32) << 23;
    let man = (half_man & 0x03FFu32) << 13;
    unsafe { mem::transmute(sign | exp | man) }
}
