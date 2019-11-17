#[macro_export]
#[cfg(any(test, feature = "debug_simd"))]
macro_rules! print_m128i {
    ($msg:expr, $x:expr) => {{
        let x: [u8; 16] = std::mem::transmute($x);
        crate::debug::print_slice($msg, &x);
    }};
}

#[macro_export]
#[cfg(not(any(test, feature = "debug_simd")))]
macro_rules! print_m128i {
    ($msg:expr, $x:expr) => {{
        // do nothing in release mode
        ()
    }};
}

#[macro_export]
#[cfg(any(test, feature = "debug_simd"))]
macro_rules! print_m256i {
    ($msg:expr, $x:expr) => {{
        let x: [u8; 32] = std::mem::transmute($x);
        crate::debug::print_slice($msg, &x);
    }};
}

#[macro_export]
#[cfg(not(any(test, feature = "debug_simd")))]
macro_rules! print_m256i {
    ($msg:expr, $x:expr) => {{
        // do nothing in release mode
        ()
    }};
}

#[cfg(any(test, feature = "debug_simd"))]
#[cfg(all(any(target_feature = "avx2", target_feature = "sse4.1"), target_feature = "popcnt"))]
pub (crate) fn print_slice(msg: &str, slice: &[u8]) {
    let mut out = Vec::new();
    let mut i = 0;

    while i < slice.len() {
        out.push(format!("{:03} {:03} {:03} {:03}",
            slice[i+0].to_string(),
            slice[i+1].to_string(),
            slice[i+2].to_string(),
            slice[i+3].to_string(),
        ));
        i += 4
    }

    println!("{:015}{}", msg, out.join(" | "))
}