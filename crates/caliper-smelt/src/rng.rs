// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Tiny deterministic RNG used by k-means++ initialisation.
//!
//! Determinism is a contract — same seed → same palette. The Steelbore
//! idempotency gate (PLAN.md §7) asserts this in CI.
//!
//! We use the xoshiro256++ variant: 256-bit state, period 2^256 − 1, no zero
//! state, passes BigCrush. The implementation is six lines of arithmetic — no
//! external dependency, no Cargo bloat.
//!
//! Reference: <https://prng.di.unimi.it/>.

/// xoshiro256++ — deterministic, 64-bit-stream RNG.
#[derive(Debug, Clone)]
pub struct Xoshiro256pp {
    state: [u64; 4],
}

impl Xoshiro256pp {
    /// Seed from a `u64`. The single-word seed is expanded to the full 256-bit
    /// state via SplitMix64, the recommended seeder in the xoshiro reference.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut sm = SplitMix64 { state: seed };
        Self {
            state: [sm.next(), sm.next(), sm.next(), sm.next()],
        }
    }

    /// Draw the next `u64` from the stream.
    pub fn next_u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }

    /// Draw a uniform `f32` in `[0.0, 1.0)`.
    #[expect(
        clippy::cast_precision_loss,
        reason = "intentional — RNG truncates to 24 bits of f32 mantissa precision"
    )]
    pub fn next_f32(&mut self) -> f32 {
        // Use the top 24 bits which fit exactly in the f32 mantissa.
        (self.next_u64() >> 40) as f32 / ((1_u32 << 24) as f32)
    }

    /// Draw a `usize` uniformly in `[0, bound)`. Returns 0 if `bound == 0`.
    pub fn next_below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        // Lemire's unbiased range reduction is overkill for our use; classic
        // multiply-shift is fine for k-means seeding.
        let r = self.next_u64();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "bound bounded by palette size ≤ 256 in our use"
        )]
        let scaled = ((u128::from(r) * u128::from(bound as u64)) >> 64) as u64;
        scaled as usize
    }
}

/// SplitMix64 — used only for seeding the main RNG.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Xoshiro256pp::new(0x1234_5678_9ABC_DEF0);
        let mut b = Xoshiro256pp::new(0x1234_5678_9ABC_DEF0);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge_quickly() {
        let mut a = Xoshiro256pp::new(0);
        let mut b = Xoshiro256pp::new(1);
        let mut diff_count = 0;
        for _ in 0..32 {
            if a.next_u64() != b.next_u64() {
                diff_count += 1;
            }
        }
        // Should diverge in essentially every position after the first few draws.
        assert!(diff_count >= 30, "diverged in only {diff_count} of 32 draws");
    }

    #[test]
    fn next_below_respects_bound() {
        let mut rng = Xoshiro256pp::new(42);
        for _ in 0..10_000 {
            let v = rng.next_below(100);
            assert!(v < 100);
        }
    }

    #[test]
    fn next_f32_in_unit_interval() {
        let mut rng = Xoshiro256pp::new(7);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!((0.0..1.0).contains(&v), "out of range: {v}");
        }
    }
}
