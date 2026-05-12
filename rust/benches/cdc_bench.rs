use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use resplix_cdc::{Config, gear};
use std::time::Duration;

fn bench_gear_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("GearHash_Comparison");
    
    // Increase warm-up and measurement time for more stability
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));
    
    let data = vec![0u8; 1024 * 1024]; // 1MB test block
    let mask = 0x0003590703530000u64; // Example mask

    // 1. Benchmark Scalar
    group.bench_function("Scalar", |b| {
        b.iter(|| gear::find_cutpoint_scalar(black_box(&data), 0, data.len(), mask))
    });

    // 2. Benchmark AVX2 (only if supported)
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            group.bench_function("AVX2", |b| {
                b.iter(|| unsafe { gear::find_cutpoint_avx2(black_box(&data), 0, data.len(), mask) })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_gear_comparison);
criterion_main!(benches);
