use criterion::{black_box, criterion_group, criterion_main, Criterion};
use resplix_cdc::{FastCDC, Config, Chunker};

fn bench_cdc(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024]; // 1MB of zeros
    let config = Config::default();

    c.bench_function("fastcdc_1mb_zeros", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&data), config);
            while let Some(chunk) = cdc.next_chunk() {
                black_box(chunk);
            }
        })
    });
}

criterion_group!(benches, bench_cdc);
criterion_main!(benches);
