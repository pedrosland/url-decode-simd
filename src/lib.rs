#[macro_use]
mod debug;
mod shuffle_mask;

#[cfg(feature = "benchmark")]
pub mod fallback;
#[cfg(feature = "benchmark")]
pub mod sse41;

#[cfg(not(feature = "benchmark"))]
mod fallback;
#[cfg(not(feature = "benchmark"))]
mod sse41;

#[inline]
pub fn url_decode(src: &[u8], dst: &mut Vec<u8>) {
    #[target_feature(enable = "sse4.1,popcnt")]
    unsafe { sse41::url_decode(src, dst) };

    #[cfg(not(target_feature = "sse4.1,popcnt"))]
    fallback::url_decode(src, dst);
}
