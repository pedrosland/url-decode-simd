//! This is a slightly modified version of the
//! [url crate](https://crates.io/crates/url)'s decode implementation.

use std::slice;
use std::borrow::Cow;

#[inline]
fn percent_decode(input: &[u8]) -> PercentDecode {
    PercentDecode {
        bytes: input.iter(),
    }
}

/// Replace b'+' with b' '
fn replace_plus(input: &[u8]) -> Cow<[u8]> {
    match input.iter().position(|&b| b == b'+') {
        None => Cow::Borrowed(input),
        Some(first_position) => {
            let mut replaced = input.to_owned();
            replaced[first_position] = b' ';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b'+' {
                    *byte = b' ';
                }
            }
            Cow::Owned(replaced)
        }
    }
}

pub (crate) fn decode(src: &[u8], dst: &mut Vec<u8>) {
    let src = replace_plus(src);
    match percent_decode(&src).if_any() {
        Some(vec) => *dst = vec,
        None => *dst = src.to_vec(),
    };
}

pub (crate) fn decode_extend(src: &[u8], dst: &mut Vec<u8>) {
    let src = replace_plus(src);
    match percent_decode(&src).if_any() {
        Some(vec) => dst.extend_from_slice(&vec),
        None => dst.extend_from_slice(&src),
    };
}

/// The return type of [`percent_decode`].
#[derive(Clone, Debug)]
struct PercentDecode<'a> {
    bytes: slice::Iter<'a, u8>,
}

fn after_percent_sign(iter: &mut slice::Iter<u8>) -> Option<u8> {
    let mut cloned_iter = iter.clone();
    let h = char::from(*cloned_iter.next()?).to_digit(16)?;
    let l = char::from(*cloned_iter.next()?).to_digit(16)?;
    *iter = cloned_iter;
    Some(h as u8 * 0x10 + l as u8)
}

impl<'a> Iterator for PercentDecode<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.bytes.next().map(|&byte| {
            if byte == b'%' {
                after_percent_sign(&mut self.bytes).unwrap_or(byte)
            } else {
                byte
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let bytes = self.bytes.len();
        (bytes / 3, Some(bytes))
    }
}

impl<'a> From<PercentDecode<'a>> for Cow<'a, [u8]> {
    fn from(iter: PercentDecode<'a>) -> Self {
        match iter.if_any() {
            Some(vec) => Cow::Owned(vec),
            None => Cow::Borrowed(iter.bytes.as_slice()),
        }
    }
}

impl<'a> PercentDecode<'a> {
    /// If the percent-decoding is different from the input, return it as a new bytes vector.
    fn if_any(&self) -> Option<Vec<u8>> {
        let mut bytes_iter = self.bytes.clone();
        while bytes_iter.any(|&b| b == b'%') {
            if let Some(decoded_byte) = after_percent_sign(&mut bytes_iter) {
                let initial_bytes = self.bytes.as_slice();
                let unchanged_bytes_len = initial_bytes.len() - bytes_iter.len() - 3;
                let mut decoded = initial_bytes[..unchanged_bytes_len].to_owned();
                decoded.push(decoded_byte);
                decoded.extend(PercentDecode { bytes: bytes_iter });
                return Some(decoded);
            }
        }
        // Nothing to decode
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::decode;

    #[test]
    fn url_decode_space() {
        let v = &[0x25, 0x32, 0x30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!(" \0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_A() {
        let v = &[0x25, 0x34, 0x31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!("A\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_AB() {
        let v = &[0x25, 0x34, 0x31, 0x25, 0x34, 0x32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!("AB\0\0\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_AaBb() {
        let v = &[
            0x25, 0x34, 0x31, // %41
            0x61, // a
            0x25, 0x34, 0x32, // %42
            0x62, // b
            0, 0, 0, 0, 0, 0, 0, 0
        ];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!("AaBb\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_AaBb_numbers() {
        let v = &[
            0x25, 0x34, 0x31, // %41
            0x61, // a
            0x25, 0x34, 0x32, // %42
            0x62, // b
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38
        ];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!("AaBb12345678".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_upper_hex_KaLb_numbers() {
        let v = &[
            0x25, 0x34, 0x42, // %4B
            0x61, // a
            0x25, 0x34, 0x43, // %4C
            0x62, // b
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38
        ];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!("KaLb12345678".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_lower_hex_KaLb_numbers() {
        let v = &[
            0x25, 0x34, 0x62, // %4b
            0x61, // a
            0x25, 0x34, 0x63, // %4c
            0x62, // b
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38
        ];
        let mut result = Vec::new();

        decode(v, &mut result);
        assert_eq!("KaLb12345678".as_bytes(), &result[..])
    }

    #[test]
    fn test_decode_invalid_chars() {
        let mut result = Vec::new();

        let v = b"%%12345678901234";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"%\x12345678901234", &result[..]);

        let v = b"%1%2345678901234";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"%1\x2345678901234", &result[..]);

        let v = b"%%%1234567890123";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"%%\x1234567890123", &result[..]);

        let v = b"%-12345678901234";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"%-12345678901234", &result[..]);

        let v = b"%1-2345678901234";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"%1-2345678901234", &result[..]);
    }

    #[test]
    fn test_random_junk() {
        let mut result = Vec::new();

        let v = b"\xCF%%sA\x00`A%5%%6%6\xEF";
        decode(v, &mut result);
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%6\xEF", &result[..]);
    }

    #[test]
    fn test_end_percent() {
        let mut result = Vec::new();

        let v = b"\xCF%%sA\x00`A%5%%6%6%";
        decode(v, &mut result);
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%6%", &result[..]);

        let v = b"\xCF%%sA\x00`A%5%%6%%6";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%%6", &result[..]);
    }

    #[test]
    fn test_split_percent() {
        let mut result = Vec::new();

        // last char of block is %
        let v = b"aaaaaaaaaaaaaaa%aaaaaaaaaaaaaaaa";
        decode(v, &mut result);
        assert_eq!(b"aaaaaaaaaaaaaaa\xAAaaaaaaaaaaaaaa", &result[..]);

        // 2nd last char of block is %
        let v = b"aaaaaaaaaaaaaa%aaaaaaaaaaaaaaaaa";
        result.clear();
        decode(v, &mut result);
        assert_eq!(b"aaaaaaaaaaaaaa\xAAaaaaaaaaaaaaaaa", &result[..]);
    }

    #[test]
    fn test_out_of_ascii_hex() {
        let mut result = Vec::new();

        let v = b"%AAaaaaaaaaaaaaa";
        decode(v, &mut result);
        assert_eq!(b"\xAAaaaaaaaaaaaaa", &result[..]);
    }
}