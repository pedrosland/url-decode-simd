#[macro_export]
#[cfg(any(test, feature = "debug_simd"))]
macro_rules! print_m128i {
    ($msg:expr, $x:expr) => {{
        let x: [u8; 16] = std::mem::transmute($x);

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
#[cfg(not(any(test, feature = "debug_simd")))]
macro_rules! print_m128i {
    ($msg:expr, $x:expr) => {{
        // do nothing in release mode
        ()
    }};
}