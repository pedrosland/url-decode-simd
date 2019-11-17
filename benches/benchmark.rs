use criterion::{BenchmarkId, Throughput, black_box, criterion_group, criterion_main, Criterion};

use url_decode_simd::{fallback};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("URL Decode");

    for i in [1, 10, 1310720].iter() {
        let section: &[u8] = &[
            0x25, 0x34, 0x31, // %41
            0x61, // a
            0x25, 0x34, 0x32, // %42
            0x62, // b
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38
        ];
        let input: Vec<u8> = (0..*i).map(|_| section.to_vec()).flatten().collect();

        if i > &500_000 {
            group.sample_size(50);
        }

        group.throughput(Throughput::Bytes(input.len() as u64));

        #[cfg(all(target_feature = "sse4.1", target_feature = "popcnt"))]
        group.bench_with_input(BenchmarkId::new("mixed SSE4.1", i), i,
            |b, _i| b.iter(|| {
                let mut output = Vec::with_capacity(input.len());
                unsafe { url_decode_simd::sse41::url_decode(black_box(input.as_slice()), &mut output) }
                output
            })
        );

        #[cfg(not(all(target_feature = "sse4.1", target_feature = "popcnt")))]
        println!("--- Skipping SSE4.1 (no CPU support compiled in)");

        group.bench_with_input(BenchmarkId::new("mixed fallback", i), i,
            |b, _i| b.iter(|| {
                let mut output = Vec::with_capacity(input.len());
                fallback::url_decode(black_box(input.as_slice()), &mut output);
                output
            })
        );
    }

    group.sample_size(100);
    for i in [1, 10, 1310720].iter() {
        let section: &[u8] = b"1234567890123456";
        let input: Vec<u8> = (0..*i).map(|_| section.to_vec()).flatten().collect();

        if i > &500_000 {
            group.sample_size(50);
        }

        group.throughput(Throughput::Bytes(input.len() as u64));

        #[cfg(all(target_feature = "sse4.1", target_feature = "popcnt"))]
        group.bench_with_input(BenchmarkId::new("no-op SSE4.1", i), i,
            |b, _i| b.iter(|| {
                let mut output = Vec::with_capacity(input.len());
                unsafe { url_decode_simd::sse41::url_decode(black_box(input.as_slice()), &mut output) }
                output
            })
        );

        #[cfg(not(all(target_feature = "sse4.1", target_feature = "popcnt")))]
        println!("--- Skipping SSE4.1 (no CPU support compiled in)");

        group.bench_with_input(BenchmarkId::new("no-op fallback", i), i,
            |b, _i| b.iter(|| {
                let mut output = Vec::with_capacity(input.len());
                fallback::url_decode(black_box(input.as_slice()), &mut output);
                output
            })
        );
    }
}

pub fn small_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Small URL Decode");
    let inputs: Vec<&[u8]> = vec![
        b"123456789012345",
        b"1",
        b"%%%%%%%%%%%%%%%",
    ];

    for (i, input) in inputs.iter().enumerate() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("fallback", i), &i,
            |b, _i| b.iter(|| {
                let mut output = Vec::with_capacity(input.len());
                fallback::url_decode(black_box(input), &mut output);
                output
            })
        );
    }
}

criterion_group!(benches, criterion_benchmark, small_benchmark);
criterion_main!(benches);
