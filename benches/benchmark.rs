use criterion::{black_box, criterion_group, criterion_main, Criterion};

use url_decode_simd::{url_decode, fallback_decode};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simd", |b| b.iter(|| {
        let input = &[
            0x25, 0x34, 0x31, // %41
            0x61, // a
            0x25, 0x34, 0x32, // %42
            0x62, // b
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38
        ];
        let mut output = Vec::with_capacity(16);
        unsafe { url_decode(black_box(input), &mut output) }
        output
    }));
    c.bench_function("fallback", |b| b.iter(|| {
        let input = &[
            0x25, 0x34, 0x31, // %41
            0x61, // a
            0x25, 0x34, 0x32, // %42
            0x62, // b
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38
        ];
        let mut output = Vec::with_capacity(16);
        fallback_decode(black_box(input), &mut output);
        output
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);