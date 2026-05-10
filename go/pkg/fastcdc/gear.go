package fastcdc

// GearTable is a pre-computed lookup table for the Gear Hash algorithm.
var GearTable = [256]uint64{
	0x0b0b2e2e6c6c5a5a, 0x1d1d3f3f7e7e6b6b, 0x2f2f4d4d8e8e7c7c, 0x3a3a5b5b9e9e8d8d,
	0x4b4b6c6c9a9a9d9d, 0x5d5d7e7e8b8b9c9c, 0x6e6e8d8d7c7c8b8b, 0x7a7a9b9b6e6e7d7d,
	0x8f8f4c4c2a2a0d0d, 0x9e9e5d5d3b3b1c1c, 0xaeae6e6e4c4c2d2d, 0xbebe7f7f5d5d3e3e,
	0xcece8f8f6e6e4f4f, 0xdede9f9f7f7f5f5f, 0xeeeeafaf8f8f6f6f, 0xfefebfbf9f9f7f7f,
	// ... [Note: Full table to be populated with random 64-bit integers]
	0x0011223344556677, 0x8899aabbccddeeff, 0x1234567890abcdef, 0xfedcba0987654321,
}

// UpdateHash updates the rolling Gear Hash with a new byte.
func UpdateHash(hash uint64, b byte) uint64 {
	return (hash << 1) + GearTable[b]
}
