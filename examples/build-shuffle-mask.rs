//! # Build Shuffle Mask
//!
//! Used to generate the shuffle_mask lookup table in shuffle_mask.rs.
//!
//! Example usage:
//! ```
//! cargo run --example build-shuffle-mask > /tmp/shuffle_mask.rs
//! mv /tmp/shuffle_mask.rs src/shuffle_mask.rs
//! ```

fn main() {
    // Quick check that this works.
    let result = build_mask_128(0b10001);
    let expected = [0, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
    assert_eq!(result, expected);

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
        let mask = build_mask_128(i as u16);
        println!("    {:?},", mask);
    }

    println!("];\n");
}

fn build_mask_128(found_mask: u16) -> [u8; 16] {
    let mut shuffle_map: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut num_junk = 0usize;
    let mut out_i = 0;
    let mut mask = 1;
    while out_i < 16 {
        shuffle_map[out_i] = num_junk as u8;
        if found_mask & mask > 0 {
            num_junk += 2;
            // If out_i < 2 this is certainly invalid.
            // There will be other invalid masks not covered by this.
            if num_junk > 2 && out_i >= 2 {
                out_i -= 2;
            }
        }
        mask = mask << 1;
        out_i += 1;
    }
    shuffle_map
}
