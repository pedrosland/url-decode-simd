# url-decode-simd

SIMD accelerated URL decoding.

It converts a string such as `Hello+brave%20world%21` to `Hello brave world!`.

Right now there is SIMD support for SSE4.1 instructions. In the future there may
be AVX2 and AVX-512 implementations. There is also a fallback in standard Rust in case
the binary is not compiled with support for SSE4.1.

## Stability

The API and features are not stable.

Opportunities for improvement include:
 * runtime CPU detection
 * allow tests to run even if current system does not support all CPU instructions (eg qemu)
 * AVX2 support
 * AVX-512 support
 * instructions for ARM

## Benchmarks

```
RUSTFLAGS="-C target-cpu=native" cargo bench --features=benchmark
```

## License

Either your choice of MIT or Apache 2 license.
