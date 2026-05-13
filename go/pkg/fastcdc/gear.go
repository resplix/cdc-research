package fastcdc

// GearTable is a high-entropy lookup table generated using SplitMix64.
// It ensures every byte value maps to a unique, 64-bit random integer
// to maximize rolling hash distribution and minimize collision cascades.
// SplitMix64 finalizer — produces excellent bit distribution from sequential inputs.
func splitmix64(seed uint64) uint64 {
	x := seed + 0x9e3779b97f4a7c15
	x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9
	x = (x ^ (x >> 27)) * 0x94d049bb133111eb
	return x ^ (x >> 31)
}

// GenerateGearTable generates the 256-entry gear table.
// In Go, we can't do this at compile time easily, so we use init() or sync.Once.
func GenerateGearTable() [256]uint64 {
	var table [256]uint64
	for i := 0; i < 256; i++ {
		// Seed offset: fractional bits of √2 — ensures no degenerate values
		table[i] = splitmix64(uint64(i) + 0x6a09e667f3bcc908)
	}
	return table
}

// Pre-computed Gear Hash lookup table.
// 256 entries × 8 bytes = 2KB — fits entirely in L1 cache (32KB typical).
//
// Properties:
// - All 256 entries are unique (guaranteed by SplitMix64 bijection)
// - ~32 bits set per entry (excellent avalanche)
// - Zero near-zero or near-max degenerate entries
var GearTable [256]uint64 = GenerateGearTable()


// UpdateHash updates the rolling Gear Hash with a new byte.
func UpdateHash(hash uint64, b byte) uint64 {
	return (hash << 1) + GearTable[b]
}

// FindCutpoint searches for a cut-point in the given data within [start, max).
func FindCutpoint(data []byte, start, max int, mask uint64) (int, uint64) {
	var hash uint64
	for i := start; i < max; i++ {
		hash = (hash << 1) + GearTable[data[i]]
		if (hash & mask) == 0 {
			return i + 1, hash
		}
	}
	return max, hash
}
