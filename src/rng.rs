//! A small deterministic generator, ours so the same seed always makes the same
//! map: splitmix64 to scramble the seed, xorshift64* to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // xorshift must never sit at zero.
        Self(z | 1)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `[0, 1)`.
    pub(crate) fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `low..=high`.
    pub(crate) fn range(&mut self, low: u32, high: u32) -> u32 {
        let span = u64::from(high.max(low) - low) + 1;
        low + (self.next_u64() % span) as u32
    }

    pub(crate) fn chance(&mut self, probability: f64) -> bool {
        self.unit() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence_and_ranges_hold() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let mut c = Rng::new(8);
        let first: Vec<u64> = (0..8).map(|_| a.next_u64()).collect();
        let again: Vec<u64> = (0..8).map(|_| b.next_u64()).collect();
        let other: Vec<u64> = (0..8).map(|_| c.next_u64()).collect();
        assert_eq!(first, again);
        assert_ne!(first, other);
        for _ in 0..1000 {
            let value = a.range(6, 12);
            assert!((6..=12).contains(&value));
            let unit = a.unit();
            assert!((0.0..1.0).contains(&unit));
        }
        assert_eq!(a.range(5, 5), 5);
        let hits = (0..2000).filter(|_| a.chance(0.35)).count();
        assert!((550..850).contains(&hits), "{hits} of 2000 at 35%");
    }
}
