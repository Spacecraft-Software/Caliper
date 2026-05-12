// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! AVX2 kernels for x86_64.
//!
//! v0.1.0 implementation: a 128-bit lane-wise distance kernel gated on AVX2
//! detection. The full 256-bit register width is not yet exploited — that
//! lands with the AoS↔SoA transpose in v0.2.0 §2.4 of TODO.md, after which
//! centroid batches will be processed 8-wide. Correctness first.

#![cfg(any(target_arch = "x86_64", target_arch = "x86"))]

use core::arch::x86_64::{__m128, _mm_loadu_ps, _mm_mul_ps, _mm_storeu_ps, _mm_sub_ps};

use crate::{CentroidIndex, OkLab};

/// AVX2 implementation of [`crate::nearest_centroid`]. See the public function
/// for the caller-visible contract.
///
/// # Safety
///
/// Requires AVX2 + FMA3 to be available at runtime. The public dispatcher in
/// `lib.rs` guarantees this via `Cpu::has_avx2()`. Calling this function on a
/// CPU without AVX2 is undefined behaviour.
#[target_feature(enable = "avx2,fma")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "centroid count bounded at u8::MAX + 1 by the public API"
)]
pub(crate) unsafe fn nearest_centroid(pixel: OkLab, centroids: &[OkLab]) -> CentroidIndex {
    debug_assert!(!centroids.is_empty());
    debug_assert!(centroids.len() <= usize::from(CentroidIndex::MAX) + 1);

    // SAFETY: `OkLab` is `repr(C, align(16))` with four `f32` fields, so reading
    // 4 contiguous lanes at its address yields well-defined f32 values. The
    // `_pad` field is initialised to `0.0` by every public constructor.
    let pixel_vec = unsafe { _mm_loadu_ps(core::ptr::from_ref(&pixel).cast::<f32>()) };

    let mut best_idx: usize = 0;
    let mut best_dist = f32::INFINITY;

    for (idx, centroid) in centroids.iter().enumerate() {
        // SAFETY: see `pixel_vec` above; same layout argument applies.
        let centroid_vec =
            unsafe { _mm_loadu_ps(core::ptr::from_ref(centroid).cast::<f32>()) };

        // `_mm_sub_ps` / `_mm_mul_ps` are safe to call when the surrounding
        // function carries the right `target_feature` gate, which it does.
        let diff = _mm_sub_ps(pixel_vec, centroid_vec);
        let sq = _mm_mul_ps(diff, diff);

        // SAFETY: `hsum3` stores 4 lanes into a stack buffer; safe modulo the
        // SSE feature gate enforced by the surrounding `target_feature`.
        let dist = unsafe { hsum3(sq) };

        if dist < best_dist {
            best_dist = dist;
            best_idx = idx;
        }
    }

    best_idx as CentroidIndex
}

/// Sum of lanes 0, 1, and 2 of `v`. Lane 3 is discarded.
///
/// Implementation deliberately stores to the stack and reads back rather than
/// using `_mm_hadd_ps`, which has high latency on Zen 1 and Bulldozer. The
/// compiler typically elides the round trip into shuffle+add anyway.
///
/// # Safety
///
/// Requires SSE.
#[target_feature(enable = "sse")]
#[inline]
unsafe fn hsum3(v: __m128) -> f32 {
    let mut lanes = [0.0_f32; 4];
    // SAFETY: `lanes` is 16 bytes of writable stack memory aligned for f32;
    // `_mm_storeu_ps` accepts any 16-byte-or-larger writable region.
    unsafe { _mm_storeu_ps(lanes.as_mut_ptr(), v) };
    lanes[0] + lanes[1] + lanes[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_scalar_for_dense_palette() {
        if !crate::Cpu::detect().has_avx2() {
            return; // host doesn't support AVX2 — this path is not exercised here
        }
        let palette: Vec<OkLab> = (0..16)
            .map(|i| {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "i is in [0,16); f32 representation is exact"
                )]
                let t = f32::from(i as u8) / 16.0;
                OkLab::new(t, t.mul_add(0.2, -0.1), t.mul_add(-0.3, 0.15))
            })
            .collect();

        for r in (0..=255_u16).step_by(17) {
            for g in (0..=255_u16).step_by(17) {
                for b in (0..=255_u16).step_by(17) {
                    let pixel = OkLab::from_srgb_u8(r as u8, g as u8, b as u8);
                    let scalar = crate::nearest_centroid_scalar(pixel, &palette);
                    // SAFETY: AVX2 was just checked above.
                    let avx = unsafe { nearest_centroid(pixel, &palette) };
                    assert_eq!(scalar, avx, "rgb=({r},{g},{b})");
                }
            }
        }
    }
}
