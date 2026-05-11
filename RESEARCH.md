# Resplix CDC Research: Investigative Findings

## Observation: The "Skip-Zone Blind Spot"

During **Experiment 3 (Block Reordering)**, we observed a 0% deduplication ratio despite swapping large, identical halves of a file.

### Technical Analysis
FastCDC uses a `min_size` skipping optimization to accelerate processing. The scanner jumps `min_size` bytes after each cut-point before it begins hashing. 

When data is reordered (e.g., swapped at a midpoint), the new alignment of the scanner can cause a valid cut-point (an "anchor") to fall within the `min_size` skip zone.

1.  **Original File**: Anchor found at position $P$.
2.  **Reordered File**: Scanner starts at position $S$. If $P - S < min\_size$, the scanner skips over $P$.
3.  **Synchronization Loss**: By skipping $P$, the scanner continues into the next block with an unaligned hash state, eventually creating a new, non-matching cut-point $P'$. This "cascades" until the scanner happens to land in a position where the next anchor is outside the skip zone.

### Empirical Proof
In our 100KB test with 8KB `min_size`, the skip zone covers ~50% of the average chunk size. In small files, a single missed anchor can prevent synchronization for the remainder of the dataset.

### Mitigation
*   **Lower `min_size`**: Increases CPU load but improves synchronization recovery.
*   **Normalized Chunking**: Helps, but cannot overcome the hard skip zone.

---

## Gear Hash: Windowed Property
The Gear Hash used in Resplix (`(h << 1) + T[b]`) is effectively a **64-byte sliding window**. After 64 bytes of identical input, the hash state is guaranteed to be synchronized regardless of the state before those 64 bytes. This is why CDC is resilient to byte-shifts, provided the anchor is not skipped.
