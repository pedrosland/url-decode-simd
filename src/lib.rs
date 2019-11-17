#[macro_use]
mod debug;
#[cfg(all(any(target_feature = "sse4.1"), target_feature = "popcnt"))]
mod shuffle_mask;

#[cfg(feature = "benchmark")]
pub mod fallback;
#[cfg(feature = "benchmark")]
#[cfg(all(target_feature = "sse4.1", target_feature = "popcnt"))]
pub mod sse41;

#[cfg(not(feature = "benchmark"))]
mod fallback;
#[cfg(not(feature = "benchmark"))]
#[cfg(all(target_feature = "sse4.1", target_feature = "popcnt"))]
mod sse41;

#[cfg(not(feature = "benchmark"))]
pub use fallback::url_decode as fallback_decode;

#[inline]
#[allow(unreachable_code)]
pub fn url_decode(src: &[u8], dst: &mut Vec<u8>) {
    #[cfg(all(target_feature = "sse4.1", target_feature = "popcnt"))]
    return unsafe { sse41::url_decode(src, dst) };

    fallback::url_decode(src, dst);
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::url_decode;

    #[test]
    fn smoke_test() {
        // Should be as wide as the widest simd implementation.
        let v = b"%20\0\0\0\0\0\0\0\0\0\0\0\0\0%20\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let mut result = Vec::new();

        url_decode(v, &mut result);
        assert_eq!(b" \0\0\0\0\0\0\0\0\0\0\0\0\0 \0\0\0\0\0\0\0\0\0\0\0\0\0", &result[..])
    }
}