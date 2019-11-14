#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::mem;

mod fallback;
mod shuffle_mask;

pub fn fallback_decode(src: &[u8], dst: &mut Vec<u8>) {
    fallback::decode(src, dst);
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! print_m128i {
    ($msg:expr, $x:expr) => {{
        let x: [u8; 16] = mem::transmute($x);

        println!("{:015}{:03} {:03} {:03} {:03} | {:03} {:03} {:03} {:03} | {:03} {:03} {:03} {:03} | {:03} {:03} {:03} {:03}", $msg,
            x[0].to_string(),
            x[1].to_string(),
            x[2].to_string(),
            x[3].to_string(),

            x[4].to_string(),
            x[5].to_string(),
            x[6].to_string(),
            x[7].to_string(),

            x[8].to_string(),
            x[9].to_string(),
            x[10].to_string(),
            x[11].to_string(),

            x[12].to_string(),
            x[13].to_string(),
            x[14].to_string(),
            x[15].to_string(),
        );
    }};
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! print_m128i {
    ($msg:expr, $x:expr) => {{
        // do nothing in release mode
        ()
    }};
}

#[target_feature(enable = "avx")]
pub unsafe fn url_decode(src: &[u8], dst: &mut Vec<u8>) {
    let mut src = src;
    // let mut dst_offset = 0;

    // Load chunks of 16 bytes of data at a time.
    while src.len() >= 16 {
        // Load data from unaligned address.
        // TODO: is this notably slower than loading from an aligned address?
        // TODO: is _mm_lddqu_si128 better?
        let chunk: __m128i = _mm_loadu_si128(src.as_ptr() as *const __m128i);
        print_m128i!("chunk", chunk);

        let search: __m128i = _mm_set1_epi8(b'%' as i8);

        let found = _mm_cmpeq_epi8(chunk, search);
        let found = _mm_and_si128(found, _mm_xor_si128(found, _mm_srli_si128(found, 1)));
        let found = _mm_and_si128(found, _mm_xor_si128(found, _mm_srli_si128(found, 2)));
        print_m128i!("found", found);

        // Find the next 2 bytes

        let mask1 = _mm_slli_si128(found, 1);
        print_m128i!("mask1", mask1);
        let first1 = _mm_and_si128(chunk, mask1);
        print_m128i!("first1", first1);

        // TODO: should this be found + 2 or mask1 + 1
        let mask2 = _mm_slli_si128(found, 2);
        print_m128i!("mask2", mask2);
        let second1 = _mm_and_si128(chunk, mask2);
        print_m128i!("second1", second1);

        // Decode hex
        let first_and_second = _mm_or_si128(first1, second1);

        // First hex digit

        // number
        let byte_zero = _mm_set1_epi8(b'0' as i8);
        let digit_mask1 = _mm_cmplt_epi8(first_and_second, _mm_set1_epi8(b':' as i8)); // : is character after 9
        let digit_mask2 = _mm_cmpgt_epi8(first_and_second, _mm_set1_epi8(b'/' as i8)); // / is character before 0
        let digit_mask = _mm_and_si128(digit_mask1, digit_mask2);
        let first_part1 = _mm_and_si128(digit_mask, _mm_sub_epi8(first_and_second, byte_zero));
        let valid_mask = digit_mask;
        print_m128i!("digit_mask1", digit_mask);
        print_m128i!("first1-1", first_part1);

        // uppercase
        let byte_upper = _mm_set1_epi8(b'A' as i8 - 10);
        let digit_mask1 = _mm_cmplt_epi8(first_and_second, _mm_set1_epi8(b'G' as i8)); // G is character after F
        let digit_mask2 = _mm_cmpgt_epi8(first_and_second, _mm_set1_epi8(b'@' as i8)); // @ is character before A
        let digit_mask = _mm_and_si128(digit_mask1, digit_mask2);
        let first_part2 = _mm_and_si128(digit_mask, _mm_sub_epi8(first_and_second, byte_upper));
        let valid_mask = _mm_or_si128(valid_mask, digit_mask);
        print_m128i!("digit_mask2", digit_mask);
        print_m128i!("first1-2", first_part2);

        // lowercase
        let byte_lower = _mm_set1_epi8(b'a' as i8 - 10);
        let digit_mask1 = _mm_cmplt_epi8(first_and_second, _mm_set1_epi8(b'g' as i8)); // g is character after f
        let digit_mask2 = _mm_cmpgt_epi8(first_and_second, _mm_set1_epi8(b'`' as i8)); // ` is character before a
        let digit_mask = _mm_and_si128(digit_mask1, digit_mask2);
        let first_part3 = _mm_and_si128(digit_mask, _mm_sub_epi8(first_and_second, byte_lower));
        let valid_mask = _mm_or_si128(valid_mask, digit_mask);
        print_m128i!("digit_mask3", digit_mask);
        print_m128i!("first1-3", first_part3);

        // both digits must be valid
        let valid_mask = _mm_and_si128(valid_mask, _mm_slli_si128(valid_mask, 1));
        let valid_mask = _mm_or_si128(valid_mask, _mm_srli_si128(valid_mask, 1));
        let valid_mask = _mm_or_si128(valid_mask, _mm_srli_si128(valid_mask, 1));
        print_m128i!("valid_mask", valid_mask);
        let found = _mm_and_si128(valid_mask, found);
        print_m128i!("found2", found);

        // merge first hex digit transforms
        let first_and_second = _mm_or_si128(_mm_or_si128(first_part1, first_part2), first_part3);
        let first_and_second = _mm_and_si128(valid_mask, first_and_second);

        // merge first hex digit

        // Note: I really want a `<< 4` for epi8 but it doesn't exist :(
        let first1 = _mm_slli_epi16(_mm_and_si128(mask1, first_and_second), 4);
        let first1 = _mm_and_si128(first1, mask1);
        print_m128i!("first1-merged", first1);

        // Second hex digit

        let second1 = _mm_srli_si128(_mm_and_si128(first_and_second, mask2), 1);

        // merge
        let hex = _mm_or_si128(first1, second1);
        let hex = _mm_srli_si128(hex, 1);
        let hex = _mm_and_si128(hex, found);
        print_m128i!("hex", hex);

        // Squash together
        // let ignore_mask = _mm_or_si128(mask1, mask2);
        let hex = _mm_blendv_epi8(chunk, hex, found);
        print_m128i!("chunk2", chunk);
        print_m128i!("found2", found);
        print_m128i!("hex2", hex);


        // print_m128i!("ignore_mask", ignore_mask);

        // The following works but only for "simple cases" like %20%20 but not %20a%20
        // let shift_mask = ignore_mask;
        // let shift_mask = _mm_and_si128(shift_mask, _mm_set1_epi8(2));
        // let shift_mask_add_2 = _mm_slli_si128(shift_mask, 2);
        // let shift_mask = _mm_add_epi8(shift_mask, _mm_add_epi8(shift_mask_add_2, shift_mask_add_2));
        // let shift_mask = _mm_add_epi8(shift_mask, _mm_slli_si128(shift_mask, 4));

        // The following works for more complex cases like %41a%42b
        //  except that it requires another _mm_add_epi8 + _mm_srli_si128 for every % symbol
        //  and it requires a loop to produce first_mask.
        //  We might as well produce the final mask in the loop.
        // let shift_mask = _mm_xor_si128(ignore_mask, _mm_set1_epi8(255u8 as i8));
        // let shift_mask = _mm_and_si128(shift_mask, _mm_set1_epi8(2));
        // print_m128i!("shift_mask", shift_mask);

        // let shuffle_map = _mm_add_epi8(shift_mask, _mm_srli_si128(shift_mask, 2));
        // print_m128i!("shuffle_map1", shuffle_map);

        // let shuffle_map = _mm_add_epi8(shuffle_map, _mm_srli_si128(shift_mask, 4));
        // print_m128i!("shuffle_map2", shuffle_map);

        // let mut first_mask: [u8; 16] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
        // for i in 0..16 {
        //     first_mask[i] = 0;
        //     if src[i] == b'%' {
        //         break;
        //     }
        // }
        // let first_mask = mem::transmute(first_mask);

        // print_m128i!("first_mask", first_mask);
        // let shuffle_map = _mm_and_si128(first_mask, shuffle_map);
        // print_m128i!("shuffle_map3", shuffle_map);

        // let shuffle_map = _mm_add_epi8(plain_shuffle_map, shuffle_map);
        // print_m128i!("shuffle_map4", shuffle_map);

        // // popcnt
        // let found_hi: __m128i = _mm_unpackhi_epi64(found, found);
        // let set_bits = _popcnt64(_mm_cvtsi128_si64(found)) + _popcnt64(_mm_cvtsi128_si64(found_hi));
        // let num_junk = ((set_bits / 8) * 2) as usize;
        // println!("num_junk: {}", num_junk);
        // let end = 16 - num_junk;

        // If trying to reproduce the loop with SIMD, considered:
        //   * _mm_cmpistrz - for zeroing out the until the first valid %.

        // Produce the shuffle map with a loop.
        // This loop costs about 45% of this function time!!!

        // let mut shuffle_map: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        // let found_arr: [u8; 16] = mem::transmute(found);
        // let mut num_junk = 0usize;
        // let mut out_i = 0;
        // for in_i in 0..16 {
        //     shuffle_map[out_i] = num_junk as u8;
        //     if found_arr[in_i] > 0 {
        //         num_junk += 2;
        //         if num_junk > 2 {
        //             out_i -= 2;
        //         }
        //     }
        //     out_i += 1;
        // }

        // let mut shift_mask = _mm_set1_epi8(255u8 as i8);
        // let mut percent_offset = 0b1;
        // Sample 16 bytes to 16 bits for ease of use
        let found_mask = _mm_movemask_epi8(found) as u32;
        // let two = _mm_set1_epi8(2);

        // Instead of all this fiddling, could we use a lookup table keyed with 2^16 keys
        //  which has all of the possible movemask combinations?
        // This value could be inflated with avx512's _mm_mask_blend_epi8.
        // Without using avx512, shuffle could be used.

        // Another idea is to swap the order of found_mask using _bswap64 and then
        //  we can access some bit operations like find index of lowest set bit
        //  and clear lowest set bit.

        // This loop is about as expensive as the more boring loop (~45% of the function time).
        // A noticable difference is that the compiler doesn't unroll the original
        // but it does unroll this loop.
        // let mut offset_map = _mm_set1_epi8(0);
        let num_percent = _popcnt32(found_mask as i32) as usize;
        let num_junk = 2 * num_percent;

        let inflate_shuffle_mask = shuffle_mask::shuffle_mask.get_unchecked(found_mask as usize);
        let inflate_shuffle_mask = mem::transmute(*inflate_shuffle_mask);

        // Calculate number of bits to process next time.
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
        let shuffle_map = _mm_add_epi8(plain_shuffle_map, inflate_shuffle_mask);
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

fn build_table() {
    // Note: the last n bytes of the mask are useless depending on how many valid
    //  percent symbols were found.
    // Note: the first bit is always 0 for valid masks.

    let max: u16 = 65535;
    for i in 0..=max {
        println!("{:?},", unsafe { build_mask(i) });
    }
}

unsafe fn build_mask(found_mask: u16) -> [u8; 16] {
    // let mut shuffle_map: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // let mut num_junk = 0usize;
    // let mut out_i = 0;
    // let mut found_mask = 1<<15;
    // for _ in 0..16 {
    //     shuffle_map[out_i] = num_junk as u8;
    //     found_mask = found_mask >> 1;
    //     if found & found_mask > 0 {
    //         if found & (found_mask >> 1) > 0 || found & (found_mask >> 2) > 0 {
    //             // this is invalid
    //             return [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //         }

    //         num_junk += 2;
    //         if num_junk > 2 {
    //             out_i -= 2;
    //         }
    //     }
    //     out_i += 1;
    // }
    // shuffle_map

    let mut shift_mask = _mm_set1_epi8(255u8 as i8);
    let mut offset_map = _mm_set1_epi8(0);
    let mut percent_offset = 0b1;
    // Sample 16 bytes to 16 bits for ease of use
    let two = _mm_set1_epi8(2);
    let mut num_percent = 0;
    for _ in 0..16 {
        shift_mask = _mm_slli_si128(shift_mask, 1);
        if percent_offset & found_mask > 0 {
            let mut this_offset = _mm_and_si128(two, shift_mask);
            for _ in 0..num_percent {
                this_offset = _mm_srli_si128(this_offset, 2);
            }
            offset_map = _mm_add_epi8(this_offset, offset_map);
            num_percent += 1;
        }
        percent_offset = percent_offset << 1;
    }
    mem::transmute(offset_map)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::url_decode;

    // #[test]
    // fn build_table() {
    //     super::build_table();
    //     assert!(false);
    // }

    #[test]
    fn url_decode_space() {
        let v = &[0x25, 0x32, 0x30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!(" \0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_A() {
        let v = &[0x25, 0x34, 0x31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!("A\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_AB() {
        let v = &[0x25, 0x34, 0x31, 0x25, 0x34, 0x32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
        assert_eq!("AB\0\0\0\0\0\0\0\0\0\0".as_bytes(), &result[..])
    }

    #[test]
    fn url_decode_AaBb() {
        let v = &[
            0x25, 0x34, 0x31, // %41
            0x61, // a
            0x25, 0x34, 0x32, // %42
            0x62, // b
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut result = Vec::new();

        unsafe { url_decode(v, &mut result) };
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

        unsafe { url_decode(v, &mut result) };
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

        unsafe { url_decode(v, &mut result) };
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

        unsafe { url_decode(v, &mut result) };
        assert_eq!("KaLb12345678".as_bytes(), &result[..])
    }

    #[test]
    fn test_decode_invalid_chars() {
        let mut result = Vec::new();

        let v = b"%%12345678901234";
        result.clear();
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
    fn test_random_junk() {
        let mut result = Vec::new();

        let v = b"\xCF%%sA\x00`A%5%%6%6\xEF";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%6\xEF", &result[..]);
    }

    #[test]
    fn test_end_percent() {
        let mut result = Vec::new();

        let v = b"\xCF%%sA\x00`A%5%%6%6%";
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%6%", &result[..]);

        let v = b"\xCF%%sA\x00`A%5%%6%%6";
        result.clear();
        unsafe { url_decode(v, &mut result) };
        assert_eq!(b"\xCF%%sA\x00`A%5%%6%%6", &result[..]);
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
}
