use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use resplix_cdc::{
    chunk_file_mmap, chunk_file_read_to_vec, Config, ContentHashMode, FastCDC, Chunker, gear,
};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
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

fn ensure_bench_file(path: &Path, size: usize) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    let mut rng = 0xC0FFEE_u64;

    // Write in 1MiB blocks to avoid allocating the full file in memory.
    let mut remaining = size;
    let mut block = vec![0u8; 1024 * 1024];
    while remaining > 0 {
        for b in block.iter_mut() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *b = (rng >> 33) as u8;
        }
        let to_write = remaining.min(block.len());
        file.write_all(&block[..to_write])?;
        remaining -= to_write;
    }

    Ok(())
}

fn bench_file_path() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| std::env::temp_dir().join("resplix-cdc").join("criterion-bench.bin"))
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

    // ── x86_64: AVX2 via vpgatherqq ─────────────────────────────────────────
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            group.bench_function("AVX2", |b| {
                b.iter(|| unsafe { gear::find_cutpoint_avx2(black_box(&data), 0, data.len(), mask) })
            });
        }
    }

    // ── AArch64: NEON via vshl_n_u64 + vadd_u64 (4-byte unrolled) ───────────
    // NEON has no gather — win comes from loop unrolling + bounds-check elimination
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            group.bench_function("NEON", |b| {
                b.iter(|| unsafe { gear::find_cutpoint_neon(black_box(&data), 0, data.len(), mask) })
            });
        }
    }

    // Dispatch always runs — shows overhead of runtime feature detection
    group.bench_function("Dispatch", |b| {
        b.iter(|| gear::find_cutpoint(black_box(&data), 0, data.len(), mask))
    });

    group.finish();
}

fn bench_file_pipeline(c: &mut Criterion) {
    const FILE_SIZE: usize = 32 * 1024 * 1024; // 32MiB (CI-friendly, still non-trivial)

    let path = bench_file_path().clone();
    ensure_bench_file(&path, FILE_SIZE).expect("failed to create benchmark file");

    let mut group = c.benchmark_group("File_Pipeline");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Bytes(FILE_SIZE as u64));

    let config_blake3 = Config::default();
    let config_cdc_only = Config {
        content_hash_mode: ContentHashMode::None,
        ..Config::default()
    };

    group.bench_function("ReadToVec_CDCOnly", |b| {
        b.iter(|| {
            let count = chunk_file_read_to_vec(path.to_string_lossy().as_ref(), config_cdc_only)
                .expect("read_to_vec failed");
            black_box(count)
        })
    });

    group.bench_function("Mmap_CDCOnly", |b| {
        b.iter(|| {
            let count =
                chunk_file_mmap(path.to_string_lossy().as_ref(), config_cdc_only).expect("mmap failed");
            black_box(count)
        })
    });

    group.bench_function("ReadToVec_Blake3", |b| {
        b.iter(|| {
            let count = chunk_file_read_to_vec(path.to_string_lossy().as_ref(), config_blake3)
                .expect("read_to_vec failed");
            black_box(count)
        })
    });

    group.bench_function("Mmap_Blake3", |b| {
        b.iter(|| {
            let count =
                chunk_file_mmap(path.to_string_lossy().as_ref(), config_blake3).expect("mmap failed");
            black_box(count)
        })
    });

    group.finish();
}

fn bench_cdc_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("CDC_Pipeline");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    let config_blake3 = Config::default();
    let config_cdc_only = Config {
        content_hash_mode: ContentHashMode::None,
        ..Config::default()
    };

    // Test multiple sizes to reveal cache effects (64KB → L1, 256KB → L2, 1MB+ → L3/RAM)
    for &size in &[64 * 1024, 256 * 1024, 1024 * 1024, 4 * 1024 * 1024] {
        let data = make_random_data(size, 0xCAFEBABE);
        let label = format!("{}KB", size / 1024);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("FastCDC_CDCOnly", &label), &data, |b, d| {
            b.iter(|| {
                let mut cdc = FastCDC::new(black_box(d), config_cdc_only);
                let mut count = 0u64;
                while let Some(chunk) = cdc.next_chunk() {
                    black_box(&chunk);
                    count += 1;
                }
                count
            })
        });

        group.bench_with_input(BenchmarkId::new("FastCDC_Blake3", &label), &data, |b, d| {
            b.iter(|| {
                let mut cdc = FastCDC::new(black_box(d), config_blake3);
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

    let config_blake3 = Config::default();
    let config_cdc_only = Config {
        content_hash_mode: ContentHashMode::None,
        ..Config::default()
    };
    let size = 1024 * 1024;// 1MB data
    group.throughput(Throughput::Bytes(size as u64));

    let zeros = vec![0u8; size];
    let random = make_random_data(size, 0x12345678);

    // Zeros: all gear lookups hit the same cache line — best-case cache scenario
    group.bench_function("Zeros_1MB_CDCOnly", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&zeros), config_cdc_only);
            while let Some(c) = cdc.next_chunk() { black_box(&c); }
        })
    });

    // Random: uniform distribution across all 256 entries — realistic workload
    group.bench_function("Random_1MB_CDCOnly", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&random), config_cdc_only);
            while let Some(c) = cdc.next_chunk() { black_box(&c); }
        })
    });

    group.bench_function("Zeros_1MB_Blake3", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&zeros), config_blake3);
            while let Some(c) = cdc.next_chunk() { black_box(&c); }
        })
    });

    group.bench_function("Random_1MB_Blake3", |b| {
        b.iter(|| {
            let mut cdc = FastCDC::new(black_box(&random), config_blake3);
            while let Some(c) = cdc.next_chunk() { black_box(&c); }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_gear_cutpoint,
    bench_file_pipeline,
    bench_cdc_pipeline,
    bench_zeros_vs_random
);
criterion_main!(benches);
