//! # Build Shuffle Mask
//!
//! Used to generate the shuffle_mask lookup table in shuffle_mask.rs.
//!
//! Note that this particular algorithm can produce masks that are technically
//! not correct but it doesn't matter because of how they are used.
//!
//! An input found_mask of 0b1000 could see an output like [0, 2, 2, 0].
//! The last digit doesn't matter as there was an encoded character found
//! and the last digit is unused in url decode's output.
//!
//! Example usage:
//! ```
//! cargo run --example build-shuffle-mask > /tmp/shuffle_mask.rs
//! mv /tmp/shuffle_mask.rs src/shuffle_mask.rs
//! ```

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::mem;

fn main() {
    // Note: the last 2+ bytes of the mask are useless depending on how many valid
    // percent symbols were found.
    // Note: the first byte is always 0 for valid masks.
    // With these in mind, the size of the mask in the table could be reduced to
    // 13 bytes.

    let max: u16 = 16383; // 14 bits

    println!(
        "pub (crate) const SHUFFLE_MASK: [[u8; 16]; {}] = [",
        max + 1
    );

    for i in 0..=max {
        let mask = unsafe { build_mask(i) };
        println!("    {:?},", mask);
    }

    println!("];");
}

#[target_feature(enable = "sse2")]
unsafe fn build_mask(found_mask: u16) -> [u8; 16] {
    let mut shift_mask = _mm_set1_epi8(255u8 as i8);
    let mut offset_map = _mm_set1_epi8(0);
    let mut percent_offset = 0b1;
    let mut num_percent = 0;
    let two = _mm_set1_epi8(2);

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
