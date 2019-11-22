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

/// Decode a URL-encoded value into the given Vector.
///
/// If compiled with support for SSE4.1 and POPCNT extensions, and the input is
/// at least 16 bytes, it will use an optimised implementation.
///
/// # Examples
///
/// ```
/// use url_decode_simd::url_decode;
///
/// let input = b"Hello%20world%21";
/// let mut output = Vec::new();
///
/// url_decode(input, &mut output);
/// assert_eq!(b"Hello world!", &output[..]);
/// ```
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
