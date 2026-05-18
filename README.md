# Resplix CDC Research Cluster

> **Data is an Asset. Movement is a Liability.**

This repository contains high-performance implementations of Content-Defined Chunking (CDC) algorithms, specifically focusing on the **FastCDC** (2016) paper: [FastCDC: a Fast and Efficient Content-Defined Chunking Approach for Data Deduplication](https://www.usenix.org/conference/atc16/technical-sessions/presentation/xia). Engineered for industrial-scale data deduplication and verifiable movement within the [Resplix](https://resplix.com) ecosystem.

---

## /// Architecture Manifestos

### The Gear Hash Advantage
Unlike traditional Rabin Fingerprinting, FastCDC utilizes the **Gear Hash** algorithm. Gear Hash is designed for SIMD acceleration, allowing us to process multiple window offsets in parallel. This eliminates the CPU bottleneck often associated with CDC, pushing throughput towards hardware limits.

### FastCDC (2016) Key Innovations
*   **Gear Hash**: A simplified rolling hash that utilizes a pre-computed lookup table to map byte values to random 64-bit integers.
*   **Normalized Chunk Distribution**: Solving the "chunk size variance" problem by using a dual-threshold mask to keep chunk sizes within a predictable, optimal range.
*   **Cut-point Skipping**: Accelerating the scanning process by skipping a minimum distance after each chunk boundary.
*   **Content-Addressing**: Leveraging **BLAKE3** for cryptographically secure chunk identification, enabling verifiable data movement.
*   **Streaming Support**: Implementations for `io.Reader` (Go) and `Read` (Rust) allowing for zero-copy, memory-efficient chunking of multi-terabyte streams.

---

## /// Performance Targets

| Metric | Target | Status |
| :--- | :--- | :--- |
| **Hash Throughput** | 10GB/s+ per core | Optimizing (Core Logic Done) |
| **Dedupe Efficiency** | > 90% (Standard Datasets) | Validating |
| **Indexing Latency** | < 0.002ms | Engineering Beta |
| **CPU Overhead** | < 1.2% Under Load | Benchmarking |

---

## /// Cross-Platform CI Benchmarks

We continuously verify our FastCDC SIMD implementations on GitHub Actions runners. Below are the median throughputs for raw Gear Hashing and the full FastCDC chunking pipeline (CDC cut-point search + per-chunk BLAKE3, default ~16KB avg chunks) across different architectures:

| Architecture | CPU / Instance | SIMD Target | Raw Hash (GiB/s) | Pipeline (MiB/s) |
| :--- | :--- | :--- | :--- | :--- |
| **Linux ARM64** | Ampere Altra (Neoverse N1) | `NEON` | **28.18** | ~963 |
| **macOS ARM64** | Apple M1 | `NEON` | **12.30** | ~844 |
| **Linux x64** | AMD EPYC 7763 (Zen 3) | `Scalar`* | **10.97** | ~1125 |
| **Windows x64** | AMD EPYC 7763 (Zen 3) | `Scalar`* | **10.91** | ~1105 |

> **Note on x86_64:** On AMD Zen 3 architectures, the scalar implementation heavily outperforms AVX2. This is a known micro-architectural quirk where Zen 3 implements the `vpgatherqq` (AVX2 gather) instruction using dozens of micro-ops, whereas our aggressively unrolled scalar loop is perfectly pipelined. On Intel architectures (Skylake/Ice Lake), AVX2 gather is fully hardware-accelerated.
> 
> **Note on ARM64:** Our NEON implementation uses a 4-byte unrolled loop with bounds-check elimination and `vshl/vadd` intrinsics. This yields a massive **+58% throughput increase** over the scalar path on Neoverse N1 processors!

---

## /// Project Structure

```text
.
├── go/             # Go (Golang) implementation with SIMD/Assembler optimizations
└── rust/           # Rust implementation utilizing AVX-512 / NEON intrinsics
```

## /// Getting Started

### Rust Implementation
```bash
cd rust
cargo bench
```

### Go Implementation
```bash
cd go
go test -v ./...
```

---

## /// Technical Deep-Dive

For a detailed exploration of the mathematics behind our implementation, see the [Resplix Architectural Manifestos](https://resplix.com/docs).

*   **SIMD Acceleration**: Leveraging `AVX-512` and `NEON` to saturate 10GbE links.
*   **Atomic Resumability**: Cryptographic proof of data parity, ensuring zero restarts from zero.
*   **Xor-Filter Indexing**: Sub-microsecond shard discovery with zero false-positives for static data sets.

# The 64-Bit Gear Table

## Why 64 Bits?

The Gear Table consists of 256 entries, each 64 bits wide. This is not accidental—it's a deliberate optimization across three dimensions: hardware architecture, collision resistance, and cache efficiency.

### 1. Native CPU Word Size

Modern CPUs are 64-bit architectures. All general-purpose registers (RAX, RBX, RCX, etc.) operate natively on 64-bit values. Using 64-bit integers means:

- XOR, SHIFT, AND operations execute in 1 cycle
- No masking or splitting across multiple registers
- Full utilization of the ALU (Arithmetic Logic Unit)

```asm
; Single instruction for the entire rolling hash update
xor rax, qword ptr [gear_table + rdx*8]
```

### 2\. Cache Line Alignment

CPU cache lines are 64 bytes wide. Since each table entry is 8 bytes:

 ```text

Cache line (64 bytes) = 8 × (64-bit gear entries)

This means:

-   Loading one entry pulls 7 neighboring entries into cache for free
    
-   The entire 2KB table occupies exactly 32 cache lines
    
-   Sequential byte access maintains near-100% cache hit rates
```

### 3\. Collision Resistance (The Birthday Paradox)

| Bit Width | Possible Values | 50% Collision Probability After |
| --- | --- | --- |
| 32-bit | 4.29 billion | ~77,000 chunks |
| 64-bit | 18.4 quintillion | ~5 billion chunks |
| 128-bit | 3.4×10³⁸ | Impractical to compute |

For a system processing petabytes of data, 64-bit provides mathematical certainty that:

-   Two different data patterns will not produce the same rolling hash
    
-   Chunk boundaries are deterministic and reproducible
    
-   No false deduplication matches
    

### 4\. Memory Footprint vs. Coverage

| Table Size | Memory | Input Coverage | L1 Fit? | L2 Fit? |
| --- | --- | --- | --- | --- |
| 256×8 bytes | 2KB | 256 values (1 byte) | ✅ | ✅ |
| 256×4 bytes | 1KB | 256 values | ✅ | ✅ |
| 256×16 bytes | 4KB | 256 values | ⚠️ | ✅ |
| 65536×8 bytes | 512KB | 65536 values (2 bytes) | ❌ | ⚠️ |

The 64-bit choice maximizes entropy per byte while maintaining optimal cache residency.

### 5\. Practical Throughput Impact

```text

With 64-bit gear table:
- L1 hit: 1-3 cycles
- L2 hit: 10-15 cycles
- L3 hit: 30-50 cycles
- Average lookup: <10ns on modern hardware

→ Sustained throughput: 10-15GB/s per core

## Why Not Smaller? (32-bit)

A 32-bit gear table would:

-   Collide after ~77,000 chunks (unacceptable for production)
    
-   Require masking operations to stay within 32 bits
    
-   Waste ALU capability (CPU still processes 64 bits)
    

## Why Not Larger? (128-bit)

A 128-bit gear table would:

-   Require SIMD instructions (AVX) for efficient operation
    
-   Not fit in L1 cache (4KB vs 32KB total)
    
-   Provide diminishing returns (2⁶⁴ is already astronomical)
    

## The Bottom Line

> [!IMPORTANT]
> **64-bit × 256 entries = 2KB** is the mathematical "sweet spot" where hardware constraints (cache lines, registers, ALU) align perfectly with statistical guarantees. Memory footprint remains trivial (0.006% of a typical 32KB L1 cache) while providing collision resistance that exceeds any realistic workload.

This is why FastCDC, and every production CDC system, uses 64-bit gear tables. It's not arbitrary—it's optimal.

---

## © 2026 Resplix Infrastructure. Sharding the future of data mobility.
