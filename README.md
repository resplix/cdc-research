# Resplix CDC Research Cluster

> **Data is an Asset. Movement is a Liability.**

This repository contains high-performance implementations of Content-Defined Chunking (CDC) algorithms, specifically focusing on the **FastCDC** (2016) paper. Engineered for industrial-scale data deduplication and verifiable movement within the [Resplix](https://resplix.com) ecosystem.

---

## /// Architecture Manifestos

### The Gear Hash Advantage
Unlike traditional Rabin Fingerprinting, FastCDC utilizes the **Gear Hash** algorithm. Gear Hash is designed for SIMD acceleration, allowing us to process multiple window offsets in parallel. This eliminates the CPU bottleneck often associated with CDC, pushing throughput towards hardware limits.

### FastCDC (2016) Key Innovations
*   **Gear Hash**: A simplified rolling hash that utilizes a pre-computed lookup table to map byte values to random 64-bit integers.
*   **Normalized Chunk Distribution**: Solving the "chunk size variance" problem by using a dual-threshold mask to keep chunk sizes within a predictable, optimal range.
*   **Cut-point Skipping**: Accelerating the scanning process by skipping a minimum distance after each chunk boundary.

---

## /// Performance Targets

| Metric | Target | Status |
| :--- | :--- | :--- |
| **Hash Throughput** | 10GB/s+ per core | Researching |
| **Dedupe Efficiency** | > 90% (Standard Datasets) | Validating |
| **Indexing Latency** | < 0.002ms | Engineering Beta |
| **CPU Overhead** | < 1.2% Under Load | Benchmarking |

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

For a detailed exploration of the mathematics behind our implementation, see the [Resplix Architectural Manifestos](https://resplix.com/docx).

*   **SIMD Acceleration**: Leveraging `AVX-512` and `NEON` to saturate 10GbE links.
*   **Atomic Resumability**: Cryptographic proof of data parity, ensuring zero restarts from zero.
*   **Xor-Filter Indexing**: Sub-microsecond shard discovery with zero false-positives for static data sets.

---

## /// License

Proprietary. © 2026 Resplix Infrastructure. Sharding the future of data mobility.
