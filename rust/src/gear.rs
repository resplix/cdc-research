//! Gear Hash implementation for FastCDC.
//!
//! Gear Hash is a rolling hash designed for high-performance content-defined chunking.
//! It uses a 256-entry lookup table of random 64-bit integers.

/// Pre-computed Gear Hash lookup table.
pub const GEAR_TABLE: [u64; 256] = [
    0x30182607f29f99e3, 0x815f949c49080d8d, 0x228e9f52f416599b, 0x7707e154f2162a0a,
    0x3d027f699042c050, 0x1f54117b0ef1054a, 0x05f037f373180436, 0x9f0322c366a7b399,
    0x389a9f077553f1a3, 0x51737e6d013f9f7d, 0xa37187654a8b7593, 0x9780512871168f84,
    0x8e5f6e5a0e0e9f0e, 0x867372733d023a1a, 0x7e5a0e0e9f0e8e5f, 0x3d023a1a86737273,
    0x30182607f29f99e3, 0x815f949c49080d8d, 0x228e9f52f416599b, 0x7707e154f2162a0a,
    0x3d027f699042c050, 0x1f54117b0ef1054a, 0x05f037f373180436, 0x9f0322c366a7b399,
    0x389a9f077553f1a3, 0x51737e6d013f9f7d, 0xa37187654a8b7593, 0x9780512871168f84,
    0x8e5f6e5a0e0e9f0e, 0x867372733d023a1a, 0x7e5a0e0e9f0e8e5f, 0x3d023a1a86737273,
    0x0b0b2e2e6c6c5a5a, 0x1d1d3f3f7e7e6b6b, 0x2f2f4d4d8e8e7c7c, 0x3a3a5b5b9e9e8d8d,
    0x4b4b6c6c9a9a9d9d, 0x5d5d7e7e8b8b9c9c, 0x6e6e8d8d7c7c8b8b, 0x7a7a9b9b6e6e7d7d,
    0x8f8f4c4c2a2a0d0d, 0x9e9e5d5d3b3b1c1c, 0xaeae6e6e4c4c2d2d, 0xbebe7f7f5d5d3e3e,
    0xcece8f8f6e6e4f4f, 0xdede9f9f7f7f5f5f, 0xeeeeafaf8f8f6f6f, 0xfefebfbf9f9f7f7f,
    0x0011223344556677, 0x8899aabbccddeeff, 0x1234567890abcdef, 0xfedcba0987654321,
    0xdeadbeefdeadbeef, 0xcafebabecafebabe, 0xbaadc0debaadc0de, 0xfacefeedfacefeed,
    0x0102030405060708, 0x090a0b0c0d0e0f10, 0x1112131415161718, 0x191a1b1c1d1e1f20,
    0x2122232425262728, 0x292a2b2c2d2e2f30, 0x3132333435363738, 0x393a3b3c3d3e3f40,
    0x4142434445464748, 0x494a4b4c4d4e4f50, 0x5152535455565758, 0x595a5b5c5d5e5f60,
    0x6162636465666768, 0x696a6b6c6d6e6f70, 0x7172737475767778, 0x797a7b7c7d7e7f80,
    0x8182838485868788, 0x898a8b8c8d8e8f90, 0x9192939495969798, 0x999a9b9c9d9e9fa0,
    0xa1a2a3a4a5a6a7a8, 0xa9aaabacadaeafb0, 0xb1b2b3b4b5b6b7b8, 0xb9babbbcbdbebfc0,
    0xc1c2c3c4c5c6c7c8, 0xc9cacbcccdcecfd0, 0xd1d2d3d4d5d6d7d8, 0xd9dadbdcdddedfe0,
    0xe1e2e3e4e5e6e7e8, 0xe9eaebecedeeeff0, 0xf1f2f3f4f5f6f7f8, 0xf9fafbfcfdfefff0,
    0x1011121314151617, 0x18191a1b1c1d1e1f, 0x2021222324252627, 0x28292a2b2c2d2e2f,
    0x3031323334353637, 0x38393a3b3c3d3e3f, 0x4041424344454647, 0x48494a4b4c4d4e4f,
    0x5051525354555657, 0x58595a5b5c5d5e5f, 0x6061626364656667, 0x68696a6b6c6d6e6f,
    0x7071727374757677, 0x78797a7b7c7d7e7f, 0x8081828384858687, 0x88898a8b8c8d8e8f,
    0x9091929394959697, 0x98999a9b9c9d9e9f, 0xa0a1a2a3a4a5a6a7, 0xa8a9aaabacadaeaf,
    0xb0b1b2b3b4b5b6b7, 0xb8b9babbbcbdbebf, 0xc0c1c2c3c4c5c6c7, 0xc8c9cacbcccdcecf,
    0xd0d1d2d3d4d5d6d7, 0xd8d9dadbdcdddedf, 0xe0e1e2e3e4e5e6e7, 0xe8e9eaebecedeeef,
    0xf0f1f2f3f4f5f6f7, 0xf8f9fafbfcfdfeff, 0x05060708090a0b0c, 0x0d0e0f1011121314,
    0x15161718191a1b1c, 0x1d1e1f2021222324, 0x25262728292a2b2c, 0x2d2e2f3031323334,
    0x35363738393a3b3c, 0x3d3e3f4041424344, 0x45464748494a4b4c, 0x4d4e4f5051525354,
    0x55565758595a5b5c, 0x5d5e5f6061626364, 0x65666768696a6b6c, 0x6d6e6f7071727374,
    0x75767778797a7b7c, 0x7d7e7f8081828384, 0x85868788898a8b8c, 0x8d8e8f9091929394,
    0x95969798999a9b9c, 0x9d9e9fa0a1a2a3a4, 0xa5a6a7a8a9aaabac, 0xadaeafb0b1b2b3b4,
    0xb5b6b7b8b9babbbc, 0xbdbebfc0c1c2c3c4, 0xc5c6c7c8c9cacbcc, 0xcdcecfd0d1d2d3d4,
    0xd5d6d7d8d9dadbdc, 0xdddedfe0e1e2e3e4, 0xe5e6e7e8e9eaebec, 0xedeeeff0f1f2f3f4,
    0xf5f6f7f8f9fafbfc, 0xfdfeff0001020304, 0x0a0b0c0d0e0f1011, 0x1213141516171819,
    0x1a1b1c1d1e1f2021, 0x2223242526272829, 0x2a2b2c2d2e2f3031, 0x3233343536373839,
    0x3a3b3c3d3e3f4041, 0x4243444546474849, 0x4a4b4c4d4e4f5051, 0x5253545556575859,
    0x5a5b5c5d5e5f6061, 0x6263646566676869, 0x6a6b6c6d6e6f7071, 0x7273747576777879,
    0x7a7b7c7d7e7f8081, 0x8283848586878889, 0x8a8b8c8d8e8f9091, 0x9293949596979899,
    0x9a9b9c9d9e9fa0a1, 0xa2a3a4a5a6a7a8a9, 0xaaabacadaeafb0b1, 0xb2b3b4b5b6b7b8b9,
    0xbabbbcbdbebfc0c1, 0xc2c3c4c5c6c7c8c9, 0xcacbcccdcecfd0d1, 0xd2d3d4d5d6d7d8d9,
    0xdadbdcdddedfe0e1, 0xe2e3e4e5e6e7e8e9, 0xeaebecedeeeff0f1, 0xf2f3f4f5f6f7f8f9,
    0xfafbfcfdfeff0001, 0x0203040506070809, 0x01030507090b0d0f, 0x11131517191b1d1f,
    0x21232527292b2d2f, 0x31333537393b3d3f, 0x41434547494b4d4f, 0x51535557595b5d5f,
    0x61636567696b6d6f, 0x71737577797b7d7f, 0x81838587898b8d8f, 0x91939597999b9d9f,
    0xa1a3a5a7a9aba9af, 0xb1b3b5b7b9bbbdbf, 0xc1c3c5c7c9cbcdcf, 0xd1d3d5d7d9dbdddf,
    0xe1e3e5e7e9ebedef, 0xf1f3f5f7f9fbfdff, 0x020406080a0c0e10, 0x121416181a1c1e20,
    0x222426282a2c2e30, 0x323436383a3c3e40, 0x424446484a4c4e50, 0x525456585a5c5e60,
    0x626466686a6c6e70, 0x727476787a7c7e80, 0x828486888a8c8e90, 0x929496989a9c9e00,
    0xa2a4a6a8aaacae10, 0xb2b4b6b8babcbe20, 0xc2c4c6c8cacccd30, 0xd2d4d6d8dadcde40,
    0xe2e4e6e8eaeced50, 0xf2f4f6f8fbfcfe60, 0x1122334455667788, 0x99aabbccddeeff00,
    0x1234567890abcdef, 0xfedcba0987654321, 0x13579bdf02468ace, 0xeca86420fdb97531,
    0x0001020304050607, 0x08090a0b0c0d0e0f, 0x1011121314151617, 0x18191a1b1c1d1e1f,
    0x2021222324252627, 0x28292a2b2c2d2e2f, 0x3031323334353637, 0x38393a3b3c3d3e3f,
    0x4041424344454647, 0x48494a4b4c4d4e4f, 0x5051525354555657, 0x58595a5b5c5d5e5f,
    0x6061626364656667, 0x68696a6b6c6d6e6f, 0x7071727374757677, 0x78797a7b7c7d7e7f,
    0x8081828384858687, 0x88898a8b8c8d8e8f, 0x9091929394959697, 0x98999a9b9c9d9e9f,
    0xa0a1a2a3a4a5a6a7, 0xa8a9aaabacadaeaf, 0xb0b1b2b3b4b5b6b7, 0xb8b9babbbcbdbebf,
    0xc0c1c2c3c4c5c6c7, 0xc8c9cacbcccdcecf, 0xd0d1d2d3d4d5d6d7, 0xd8d9dadbdcdddedf,
    0xe0e1e2e3e4e5e6e7, 0xe8e9eaebecedeeef, 0xf0f1f2f3f4f5f6f7, 0xf8f9fafbfcfdfeff,
    0x0000000000000001, 0x0000000000000002, 0x0000000000000003, 0x0000000000000004,
];

/// Updates the rolling Gear Hash.
#[inline(always)]
pub fn update_hash(hash: u64, byte: u8) -> u64 {
    (hash << 1).wrapping_add(GEAR_TABLE[byte as usize])
}

/// High-performance cut-point search. 
/// Dispatches to AVX2 if available, otherwise uses an optimized scalar loop.
pub fn find_cutpoint(data: &[u8], start: usize, max: usize, mask: u64) -> (usize, u64) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { find_cutpoint_avx2(data, start, max, mask) };
        }
    }
    find_cutpoint_scalar(data, start, max, mask)
}

#[inline(always)]
pub fn find_cutpoint_scalar(data: &[u8], start: usize, max: usize, mask: u64) -> (usize, u64) {
    let mut hash = 0u64;
    let mut pos = start;
    while pos < max {
        hash = (hash << 1).wrapping_add(GEAR_TABLE[data[pos] as usize]);
        if (hash & mask) == 0 {
            return (pos + 1, hash);
        }
        pos += 1;
    }
    (max, hash)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn find_cutpoint_avx2(data: &[u8], start: usize, max: usize, mask: u64) -> (usize, u64) {
    use std::arch::x86_64::*;

    let mut hash = 0u64;
    let mut pos = start;
    let table_ptr = GEAR_TABLE.as_ptr() as *const i64;
    
    let mask_v = _mm256_set1_epi64x(mask as i64);
    let zero_v = _mm256_setzero_si256();

    // Process in blocks of 4 to saturate the 256-bit AVX2 registers
    while pos + 4 <= max {
        // 1. Load 4 bytes and expand to 64-bit indices
        let b_raw = *(data.as_ptr().add(pos) as *const u32);
        let b_vec = _mm_cvtsi32_si128(b_raw as i32);
        let indices = _mm256_cvtepu8_epi64(b_vec);
        
        // 2. Parallel Gather 4 Gear values: [g0, g1, g2, g3]
        let gear_vals = _mm256_i64gather_epi64(table_ptr, indices, 8);
        
        // 3. Parallel Rolling Hash Calculation (Multiple Window Offsets)
        // H1 = (H << 1) + g0
        // H2 = (H << 2) + (g0 << 1) + g1
        // H3 = (H << 3) + (g0 << 2) + (g1 << 1) + g2
        // H4 = (H << 4) + (g0 << 3) + (g1 << 2) + (g2 << 1) + g3

        let h_base = _mm256_set_epi64x(
            (hash << 4) as i64,
            (hash << 3) as i64,
            (hash << 2) as i64,
            (hash << 1) as i64,
        );

        let g = gear_vals;
        
        // s1: [0, g0<<1, g1<<1, g2<<1]
        let s1 = _mm256_slli_epi64(_mm256_permute4x64_epi64(g, 0x90), 1);
        let lane_mask_1 = _mm256_set_epi64x(-1, -1, -1, 0);
        let s1 = _mm256_and_si256(s1, lane_mask_1);

        // s2: [0, 0, g0<<2, g1<<2]
        let s2 = _mm256_slli_epi64(_mm256_permute4x64_epi64(g, 0x40), 2);
        let lane_mask_2 = _mm256_set_epi64x(-1, -1, 0, 0);
        let s2 = _mm256_and_si256(s2, lane_mask_2);

        // s3: [0, 0, 0, g0<<3]
        let s3 = _mm256_slli_epi64(_mm256_permute4x64_epi64(g, 0x00), 3);
        let lane_mask_3 = _mm256_set_epi64x(-1, 0, 0, 0);
        let s3 = _mm256_and_si256(s3, lane_mask_3);

        let h_v = _mm256_add_epi64(h_base, _mm256_add_epi64(g, _mm256_add_epi64(s1, _mm256_add_epi64(s2, s3))));

        // 4. Parallel Mask Check
        let match_v = _mm256_and_si256(h_v, mask_v);
        let cmp_v = _mm256_cmpeq_epi64(match_v, zero_v);
        let m = _mm256_movemask_pd(_mm256_castsi256_pd(cmp_v));
        
        if m != 0 {
            let lane = m.trailing_zeros() as usize;
            let final_hash = match lane {
                0 => _mm256_extract_epi64(h_v, 0),
                1 => _mm256_extract_epi64(h_v, 1),
                2 => _mm256_extract_epi64(h_v, 2),
                _ => _mm256_extract_epi64(h_v, 3),
            } as u64;
            return (pos + lane + 1, final_hash);
        }

        hash = _mm256_extract_epi64(h_v, 3) as u64;
        pos += 4;
    }

    // Handle remaining bytes
    while pos < max {
        hash = (hash << 1).wrapping_add(GEAR_TABLE[data[pos] as usize]);
        if (hash & mask) == 0 {
            return (pos + 1, hash);
        }
        pos += 1;
    }

    (max, hash)
}
