#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::mem;

use crate::fallback;
use crate::shuffle_mask;

use shuffle_mask::SHUFFLE_MASK;

/// This is an SSE4.1 + POPCNT implementation of URL decode.
///
/// It requires SSE4.1 for `_mm_blendv_epi8` and `_mm_test_all_zeros`.
///
/// No validation of UTF-8 data is performed so if a string is desired,
/// it should be sanitised with eg [String::from_utf8_lossy]
/// (https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy)
#[target_feature(enable = "sse4.1,popcnt")]
pub unsafe fn url_decode(src: &[u8], dst: &mut Vec<u8>) {
    let mut src = src;

    let byte_plus = _mm_set1_epi8(b'+' as i8);
    let byte_space = _mm_set1_epi8(b' ' as i8);
    let byte_percent = _mm_set1_epi8(b'%' as i8);

    // Load chunks of 16 bytes of data at a time.
    while src.len() >= 16 {
        // Load data from unaligned address.
        // TODO: is this notably slower than loading from an aligned address?
        let chunk = _mm_loadu_si128(src.as_ptr() as *const __m128i);
        print_m128i!("chunk", chunk);

        // Replace plus (+) with space
        let found = _mm_cmpeq_epi8(chunk, byte_plus);
        print_m128i!("found+", found);
        let chunk = _mm_blendv_epi8(chunk, byte_space, found);
        print_m128i!("chunk+", chunk);

        // Locate percent symbol
        let found = _mm_cmpeq_epi8(chunk, byte_percent);
        let found = _mm_and_si128(found, _mm_xor_si128(found, _mm_srli_si128(found, 1)));
        let found = _mm_and_si128(found, _mm_xor_si128(found, _mm_srli_si128(found, 2)));
        print_m128i!("found", found);

        if _mm_test_all_zeros(found, found) > 0 {
            let x: [u8; 16] = mem::transmute(chunk);
            dst.extend_from_slice(&x[..16]);
            src = &src[16..];
            continue;
        }

        // Find the next 2 bytes

        let mask1 = _mm_slli_si128(found, 1);
        print_m128i!("mask1", mask1);
        let first1 = _mm_and_si128(chunk, mask1);
        print_m128i!("first1", first1);

        // Using `found` allows us to not depend on mask1
        let mask2 = _mm_slli_si128(found, 2);
        print_m128i!("mask2", mask2);
        let second1 = _mm_and_si128(chunk, mask2);
        print_m128i!("second1", second1);

        // Decode hex

        let first_and_second = _mm_or_si128(first1, second1);

        // Number hex
        let byte_zero = _mm_set1_epi8(b'0' as i8);
        let digit_mask1 = _mm_cmplt_epi8(first_and_second, _mm_set1_epi8(b':' as i8)); // : is character after 9
        let digit_mask2 = _mm_cmpgt_epi8(first_and_second, _mm_set1_epi8(b'/' as i8)); // / is character before 0
        let digit_mask = _mm_and_si128(digit_mask1, digit_mask2);
        let first_part1 = _mm_and_si128(digit_mask, _mm_sub_epi8(first_and_second, byte_zero));
        let valid_mask = digit_mask;
        print_m128i!("digit_mask1", digit_mask);
        print_m128i!("first1-1", first_part1);

        // Uppercase hex
        let byte_upper = _mm_set1_epi8(b'A' as i8 - 10);
        let digit_mask1 = _mm_cmplt_epi8(first_and_second, _mm_set1_epi8(b'G' as i8)); // G is character after F
        let digit_mask2 = _mm_cmpgt_epi8(first_and_second, _mm_set1_epi8(b'@' as i8)); // @ is character before A
        let digit_mask = _mm_and_si128(digit_mask1, digit_mask2);
        let first_part2 = _mm_and_si128(digit_mask, _mm_sub_epi8(first_and_second, byte_upper));
        let valid_mask = _mm_or_si128(valid_mask, digit_mask);
        print_m128i!("digit_mask2", digit_mask);
        print_m128i!("first1-2", first_part2);

        // Lowercase hex
        let byte_lower = _mm_set1_epi8(b'a' as i8 - 10);
        let digit_mask1 = _mm_cmplt_epi8(first_and_second, _mm_set1_epi8(b'g' as i8)); // g is character after f
        let digit_mask2 = _mm_cmpgt_epi8(first_and_second, _mm_set1_epi8(b'`' as i8)); // ` is character before a
        let digit_mask = _mm_and_si128(digit_mask1, digit_mask2);
        let first_part3 = _mm_and_si128(digit_mask, _mm_sub_epi8(first_and_second, byte_lower));
        let valid_mask = _mm_or_si128(valid_mask, digit_mask);
        print_m128i!("digit_mask3", digit_mask);
        print_m128i!("first1-3", first_part3);

        // Check that both digits are valid
        let valid_mask = _mm_and_si128(valid_mask, _mm_slli_si128(valid_mask, 1));
        let valid_mask = _mm_or_si128(valid_mask, _mm_srli_si128(valid_mask, 1));
        let valid_mask = _mm_or_si128(valid_mask, _mm_srli_si128(valid_mask, 1));
        print_m128i!("valid_mask", valid_mask);
        let found = _mm_and_si128(valid_mask, found);
        print_m128i!("found2", found);

        // Merge first hex digit transforms
        let first_and_second = _mm_or_si128(_mm_or_si128(first_part1, first_part2), first_part3);
        let first_and_second = _mm_and_si128(valid_mask, first_and_second);

        // Note: I really want a `<< 4` for epi8 but it doesn't exist :(
        // This is ok because valid first digits have a spare byte on each side.
        let first1 = _mm_slli_epi16(_mm_and_si128(mask1, first_and_second), 4);
        let first1 = _mm_and_si128(first1, mask1);
        print_m128i!("first1-merged", first1);

        // Second hex digit
        let second1 = _mm_srli_si128(_mm_and_si128(first_and_second, mask2), 1);

        // Merge hex digits into place and position where the percent was
        let hex = _mm_or_si128(first1, second1);
        let hex = _mm_srli_si128(hex, 1);
        let hex = _mm_and_si128(hex, found);
        print_m128i!("hex", hex);

        // Squash hex and original data together with mask
        let hex = _mm_blendv_epi8(chunk, hex, found);
        print_m128i!("chunk2", chunk);
        print_m128i!("found2", found);
        print_m128i!("hex2", hex);
        // Reduce 16 bytes to 16 bits for ease of use
        let found_mask = _mm_movemask_epi8(found) as u32;

        // Count number of valid percent symbols. These are represented as a 1 in found_mask.
        let num_percent = _popcnt32(found_mask as i32) as usize;
        let num_junk = 2 * num_percent;

        // Shave off the right two bits as they are always 0 or irelevant
        let found_mask = found_mask & 0b0011111111111111;

        // Instead of a map, we could swap the order of found_mask using _bswap64 and then
        //  we can access some bit operations like find index of lowest set bit
        //  and clear lowest set bit.

        // Another possibility is the map could only contain a 1 when there is an increment.
        // This increment can be used to derive the shuffle_mask eg 00010 -> 2,2,2,2,0.
        let shuffle_mask = SHUFFLE_MASK.get_unchecked(found_mask as usize);
        let shuffle_mask = mem::transmute(*shuffle_mask);
        print_m128i!("shuffle_mask", shuffle_mask);

        // Calculate number of bits to re-process next time.
        // This is because we end in % or %X and can't decode bytes that aren't in the chunk.

        let mut shift_next = 0;
        if src[14] == b'%' {
            shift_next += 2;
        } else if src[15] == b'%' {
            shift_next += 1;
        }

        let src_end: usize = 16 - shift_next;
        let dst_end: usize = src_end - num_junk;

        // Shuffle the output
        let plain_shuffle_map = _mm_set_epi8(15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0);
        let shuffle_map = _mm_add_epi8(plain_shuffle_map, shuffle_mask);
        print_m128i!("shuffle_map", shuffle_map);

        let hex = _mm_shuffle_epi8(hex, shuffle_map);

        // Copy to dst
        let x: [u8; 16] = mem::transmute(hex);
        dst.extend_from_slice(&x[..dst_end]);

        // Advance
        src = &src[src_end..];
    }

    if src.len() > 0 {
        fallback::decode_extend(src, dst);
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::url_decode;

    #[test]
    fn url_decode_space() {
        let v = b"%20\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b" \0\0\0\0\0\0\0\0\0\0\0\0\0", &result[..])
    }

    #[test]
    fn url_decode_A() {
        let v = b"%41\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"A\0\0\0\0\0\0\0\0\0\0\0\0\0", &result[..])
    }

    #[test]
    fn url_decode_AB() {
        let v = b"%41%42\0\0\0\0\0\0\0\0\0\0";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"AB\0\0\0\0\0\0\0\0\0\0", &result[..])
    }

    #[test]
    fn url_decode_AaBb() {
        let v = b"%41a%42b\0\0\0\0\0\0\0\0\0";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"AaBb\0\0\0\0\0\0\0\0\0", &result[..])
    }

    #[test]
    fn url_decode_AaBb_numbers() {
        let v = b"%41a%42b12345678";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"AaBb12345678", &result[..])
    }

    #[test]
    fn url_decode_upper_hex_KaLb_numbers() {
        let v = b"%4Ba%4Cb12345678";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"KaLb12345678", &result[..])
    }

    #[test]
    fn url_decode_lower_hex_KaLb_numbers() {
        let v = b"%4ba%4cb12345678";
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"KaLb12345678", &result[..])
    }

    #[test]
    fn test_decode_invalid_chars() {
        let mut result = Vec::new();

        let v = b"%%12345678901234";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"%\x12345678901234", &result[..]);

        let v = b"%1%2345678901234";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"%1\x2345678901234", &result[..]);

        let v = b"%%%1234567890123";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"%%\x1234567890123", &result[..]);

        let v = b"%-12345678901234";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"%-12345678901234", &result[..]);

        let v = b"%1-2345678901234";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"%1-2345678901234", &result[..]);
    }

    #[test]
    fn test_end_percent() {
        let mut result = Vec::new();

        // last char of block is %
        let v = b"aaaaaaaaaaaaaaa%";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"aaaaaaaaaaaaaaa%", &result[..]);

        // 2nd last char of block is %
        let v = b"aaaaaaaaaaaaaa%a";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"aaaaaaaaaaaaaa%a", &result[..]);
    }

    #[test]
    fn test_split_percent() {
        let mut result = Vec::new();

        // last char of block is %
        let v = b"aaaaaaaaaaaaaaa%aaaaaaaaaaaaaaaa";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"aaaaaaaaaaaaaaa\xAAaaaaaaaaaaaaaa", &result[..]);

        // 2nd last char of block is %
        let v = b"aaaaaaaaaaaaaa%aaaaaaaaaaaaaaaaa";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"aaaaaaaaaaaaaa\xAAaaaaaaaaaaaaaaa", &result[..]);
    }

    #[test]
    fn test_out_of_ascii_hex() {
        let mut result = Vec::new();

        let v = b"%AAaaaaaaaaaaaaa";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"\xAAaaaaaaaaaaaaa", &result[..]);
    }

    #[test]
    fn test_replace_plus() {
        let mut result = Vec::new();

        let v = b"a+a+a+a+a+a+a+a+";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"a a a a a a a a ", &result[..]);
    }

    #[test]
    fn test_random_junk() {
        let mut result = Vec::new();

        let v = b"\xCF%%sA\x00`A%5%%6%6\xEF";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%6\xEF", &result[..]);
    }
}
