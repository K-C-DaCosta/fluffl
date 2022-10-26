#![allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    InvalidBase64Digit,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        let err_msg = match self {
            Self::InvalidBase64Digit => "invalid base64 digit detected",
        };
        String::from(err_msg)
    }
}

pub fn encode(data: &[u8]) -> String {
    let mut output = String::new();
    let num_bits = data.len() as u128 * 8;
    let mut accum = 0;
    for i in 0..num_bits {
        let bit = (data[(i / 8) as usize] >> (8 - (i % 8) - 1)) & 1;
        accum |= bit << (6 - (i % 6) - 1);
        if i % 6 == 5 {
            output.push(map_binary_to_b64_digit(accum).unwrap());
            accum = 0;
        }
    }
    if accum != 0 {
        output.push(map_binary_to_b64_digit(accum).unwrap());
    }

    //apply padding
    while output.len() % 4 != 0 {
        output.push('=');
    }

    output
}

#[allow(clippy::same_item_push)]
pub fn decode<S: AsRef<str>>(raw_text: S) -> Result<Vec<u8>, Error> {
    let raw_text = raw_text.as_ref();
    let mut byte_buffer = vec![0; 64];
    let mut remaining_size = byte_buffer.len() * 8;
    let mut bit_cursor = 0;

    let iter = raw_text.chars().filter(|&a| !a.is_whitespace() && a != '=');

    for c in iter {
        //every bit pattern has 6 bits worth of content
        let bit_pattern = match map_b64_digit_to_binary(c) {
            Some(x) => x,
            None => return Err(Error::InvalidBase64Digit),
        };
        //loop through every kth bit, where 0<=k<=5
        for k in 0..6 {
            let bit = (bit_pattern >> (6 - k - 1)) & 1;
            if bit_cursor >= remaining_size {
                //allocate 8 more bytes (64 bits)
                remaining_size += 64;
                for _ in 0..8 {
                    byte_buffer.push(0);
                }
            }
            //sets the bit
            let mut chunk = byte_buffer[bit_cursor / 8];
            chunk |= bit << (8 - bit_cursor % 8 - 1);
            byte_buffer[bit_cursor / 8] = chunk;
            bit_cursor += 1;
        }
    }
    Ok(byte_buffer)
}

fn map_b64_digit_to_binary(c: char) -> Option<u8> {
    let c = c as u8;
    let c = match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        b'=' => 0,
        _ => 0,
    };
    Some(c)
}

fn map_binary_to_b64_digit(b: u8) -> Option<char> {
    let b = b as u8;
    let b = match b {
        0..=25 => b + b'A',
        26..=51 => (b - 26) + b'a',
        52..=61 => (b - 52) + b'0',
        62 => b'+',
        63 => b'/',
        _ => return None,
    };
    Some(b as char)
}

#[test]
fn encode_sanity_test() {
    let tests = [
        ["pleasure.", "cGxlYXN1cmUu"],
        ["leasure.", "bGVhc3VyZS4="],
        ["easure.", "ZWFzdXJlLg=="],
        ["asure.", "YXN1cmUu"],
        ["sure.", "c3VyZS4="],
    ];
    for &[input, output] in tests.iter() {
        assert_eq!(encode(input.as_bytes()), output);
    }
}
