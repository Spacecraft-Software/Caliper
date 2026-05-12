// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-anvil
//!
//! SIMD primitives layer for Caliper. This is the **only** crate in the
//! workspace permitted to contain `unsafe` blocks — every other crate
//! enforces `#![forbid(unsafe_code)]` at the crate level.
//!
//! Anvil exposes safe Rust wrappers over architecture-specific kernels. The
//! `unsafe` is confined to small, exhaustively-tested intrinsic shims; the
//! public API is entirely safe.
//!
//! ## Backends
//!
//! Runtime dispatch (see [PRD §6.1.2]) picks the best available kernel:
//!
//! | ISA          | Architecture       |
//! |--------------|--------------------|
//! | SSE2 / SSE4.1| x86_64 baseline    |
//! | AVX2 + FMA3  | x86_64 Haswell+    |
//! | AVX-512      | x86_64 Skylake-X+  |
//! | NEON         | aarch64            |
//! | SVE / SVE2   | aarch64 ARMv9+     |
//! | WASM SIMD128 | wasm32             |
//!
//! Every kernel ships with a scalar reference impl plus a property-test
//! equivalence harness so SIMD vs scalar parity is provable.
//!
//! ## Safety
//!
//! All `unsafe` blocks carry a `// SAFETY:` comment and are validated under
//! Miri in CI (`cargo +nightly miri test -p caliper-anvil`). The
//! `undocumented_unsafe_blocks` lint is enabled at the workspace level so an
//! unsafe block without a safety comment fails the build.
//!
//! [PRD §6.1.2]: https://Caliper.Steelbore.com/prd#612-simd-instruction-set-exploitation

// Anvil is the only crate in the workspace that may use `unsafe`. We rely on
// `unsafe_op_in_unsafe_fn` (workspace lint) to force explicit `unsafe { … }`
// blocks inside every unsafe fn, so SAFETY comments document each call site.

pub mod cpu;
pub mod oklab;

mod scalar;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;

#[cfg(target_arch = "aarch64")]
mod neon;

pub use cpu::Cpu;
pub use oklab::OkLab;

/// Index into a centroid slice — the value k-means stores per pixel.
///
/// k is bounded at 256 by the `--colors` flag (PRD §9.3), so `u8` is always
/// sufficient. Using a small integer keeps the per-pixel label buffer cache-friendly.
pub type CentroidIndex = u8;

/// Find the index of the nearest centroid in [`OkLab`] space.
///
/// This is the inner loop of k-means quantisation (PRD §8.3). Runtime dispatch
/// picks the best available implementation for the current CPU.
///
/// # Panics
///
/// Panics if `centroids` is empty or contains more than [`u8::MAX`] entries.
/// k is bounded at 256 by the public `--colors` flag, so a caller respecting
/// the documented range never panics.
#[inline]
#[must_use]
pub fn nearest_centroid(pixel: OkLab, centroids: &[OkLab]) -> CentroidIndex {
    assert!(!centroids.is_empty(), "centroids slice must not be empty");
    assert!(
        centroids.len() <= usize::from(CentroidIndex::MAX) + 1,
        "centroids slice must fit in CentroidIndex (≤ 256 entries)"
    );

    let cpu = Cpu::detect();

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    if cpu.has_avx2() {
        // SAFETY: We just checked the AVX2 feature flag at runtime via
        // `Cpu::has_avx2`, which uses `is_x86_feature_detected!`. The kernel
        // requires AVX2 + FMA3; both are bundled under the AVX2 feature gate
        // on every x86_64 CPU that supports it.
        return unsafe { avx2::nearest_centroid(pixel, centroids) };
    }

    #[cfg(target_arch = "aarch64")]
    if cpu.has_neon() {
        // SAFETY: NEON is mandatory on aarch64, but Rust still treats the
        // intrinsics as unsafe. `Cpu::has_neon` returns true unconditionally
        // on aarch64 to make the call-site symmetric with the x86 paths.
        return unsafe { neon::nearest_centroid(pixel, centroids) };
    }

    let _ = cpu; // suppress unused-binding warning on platforms with no SIMD path
    scalar::nearest_centroid(pixel, centroids)
}

/// Scalar reference implementation — exposed for testing and as a fallback
/// for callers who want deterministic, ISA-independent behaviour.
#[inline]
#[must_use]
pub fn nearest_centroid_scalar(pixel: OkLab, centroids: &[OkLab]) -> CentroidIndex {
    scalar::nearest_centroid(pixel, centroids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_centroids_panics() {
        let pixel = OkLab::new(0.5, 0.0, 0.0);
        let result = std::panic::catch_unwind(|| nearest_centroid(pixel, &[]));
        assert!(result.is_err());
    }

    #[test]
    fn nearest_picks_closest_centroid() {
        let palette = [
            OkLab::new(0.0, 0.0, 0.0),  // black
            OkLab::new(1.0, 0.0, 0.0),  // white-ish
            OkLab::new(0.5, 0.2, -0.1), // mid
        ];
        // A near-black pixel should snap to index 0.
        assert_eq!(nearest_centroid(OkLab::new(0.05, 0.01, -0.01), &palette), 0);
        // A near-white pixel should snap to index 1.
        assert_eq!(nearest_centroid(OkLab::new(0.95, 0.02, 0.0), &palette), 1);
    }
}
