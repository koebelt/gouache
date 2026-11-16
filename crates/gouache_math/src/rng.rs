//! Integer hashing and a PCG32 generator. Platform independent by construction.

/// 32-bit integer hash (lowbias32 by Chris Wellons, public domain).
#[inline]
pub fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// 64-bit integer hash (splitmix64 finalizer).
#[inline]
pub fn hash_u64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Hash two integers (e.g. a lattice cell) into one.
#[inline]
pub fn hash2(a: i32, b: i32) -> u32 {
    hash_u32((a as u32) ^ hash_u32((b as u32).wrapping_add(0x9e37_79b9)))
}

/// Hash three integers into one.
#[inline]
pub fn hash3(a: i32, b: i32, c: i32) -> u32 {
    hash_u32((a as u32) ^ hash2(b, c).wrapping_add(0x85eb_ca6b))
}

/// PCG32 (XSH RR) random number generator.
#[derive(Clone, Debug)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    /// Seed a generator; `stream` selects an independent sequence.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut r = Self { state: 0, inc: (stream << 1) | 1 };
        r.next_u32();
        r.state = r.state.wrapping_add(seed);
        r.next_u32();
        r
    }

    /// Next 32 random bits.
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform f32 in [0, 1).
    #[inline]
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }

    /// Uniform f32 in [lo, hi).
    #[inline]
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// Symmetric jitter in [-a, a).
    #[inline]
    pub fn jitter(&mut self, a: f32) -> f32 {
        self.range(-a, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcg_reference_sequence_is_stable() {
        // Golden values guard cross-platform determinism.
        let mut r = Pcg32::new(42, 54);
        let v: Vec<u32> = (0..4).map(|_| r.next_u32()).collect();
        let mut r2 = Pcg32::new(42, 54);
        let v2: Vec<u32> = (0..4).map(|_| r2.next_u32()).collect();
        assert_eq!(v, v2);
        // Published PCG32 demo output for seed 42 / stream 54.
        assert_eq!(v, vec![0xa15c_02b7, 0x7b47_f409, 0xba1d_3330, 0x83d2_f293]);
    }

    #[test]
    fn f32_in_unit_range() {
        let mut r = Pcg32::new(1, 1);
        for _ in 0..10_000 {
            let x = r.f32();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn hashes_differ() {
        assert_ne!(hash2(1, 2), hash2(2, 1));
        assert_ne!(hash3(1, 2, 3), hash3(3, 2, 1));
        assert_ne!(hash_u64(1), hash_u64(2));
    }
}
