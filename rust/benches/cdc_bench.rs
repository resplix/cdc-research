use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use resplix_cdc::{Config, FastCDC, Chunker, gear};
use std::time::Duration;

/// Generate deterministic "random" data using LCG
fn make_random_data(size: usize, seed: u64) -> Vec<u8> {
    let mut data = vec![0u8; size];
    let mut rng = seed;
    for byte in data.iter_mut() {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *byte = (rng >> 33) as u8;
    }
    data
}

fn bench_gear_cutpoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("GearHash_Cutpoint");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));

    let data = make_random_data(1024 * 1024, 0xDEADBEEF); // 1MB random
    let mask = 0x0003590703530000u64;
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("Scalar", |b| {
        b.iter(|| gear::find_cutpoint_scalar(black_box(&data), 0, data.len(), mask))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            group.bench_function("AVX2", |b| {
                b.iter(|| unsafe { gear::find_cutpoint_avx2(black_box(&data), 0, data.len(), mask) })
            });
        }
    }

    group.bench_function("Dispatch", |b| {
        b.iter(|| gear::find_cutpoint(black_box(&data), 0, data.len(), mask))
    });

    group.finish();
}

fn bench_cdc_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("CDC_Pipeline");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    let config = Config::default();

    // Test multiple sizes
    for &size in &[64 * 1024, 256 * 1024, 1024 * 1024, 4 * 1024 * 1024] {
        let data = make_random_data(size, 0xCAFEBABE);
        let label = format!("{}KB", size / 1024);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("FastCDC", &label), &data, |b, d| {
            b.iter(|| {
                let mut cdc = FastCDC::new(black_box(d), config);
                let mut count = 0u64;
                while let Some(chunk) = cdc.next_chunk() {
                    black_box(&chunk);
                    count += 1;
                }
                count
            })
        });
    }

    group.finish();
}

fn bench_zeros_vs_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("Data_Entropy");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    let config = Config::default();
    let size = 1024 * 1024;
    group.throughput(Throughput::Bytes(size as u64));

    let zeros = vec![0u8; size];
    let random = make_random_data(size, 0x12345678);

    group.bench_function("Zeros_1MB", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&zeros), config);
            while let Some(c) = cdc.next_chunk() { black_box(&c); }
        })
    });

    group.bench_function("Random_1MB", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&random), config);
            while let Some(c) = cdc.next_chunk() { black_box(&c); }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_gear_cutpoint, bench_cdc_pipeline, bench_zeros_vs_random);
criterion_main!(benches);
