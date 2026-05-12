// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! NEON kernels for aarch64.
//!
//! NEON is part of the AArch64 baseline ABI, so detection always succeeds on
//! aarch64. The intrinsics are still gated with `target_feature` for clarity
//! and to keep the pattern symmetric with the x86 paths.

#![cfg(target_arch = "aarch64")]

use core::arch::aarch64::{float32x4_t, vgetq_lane_f32, vld1q_f32, vmulq_f32, vsubq_f32};

use crate::{CentroidIndex, OkLab};

/// NEON implementation of [`crate::nearest_centroid`].
///
/// # Safety
///
/// Requires NEON, which is always available on aarch64. The public dispatcher
/// in `lib.rs` guards this with a `Cpu::has_neon()` check for symmetry.
#[target_feature(enable = "neon")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "centroid count bounded at u8::MAX + 1 by the public API"
)]
pub(crate) unsafe fn nearest_centroid(pixel: OkLab, centroids: &[OkLab]) -> CentroidIndex {
    debug_assert!(!centroids.is_empty());
    debug_assert!(centroids.len() <= usize::from(CentroidIndex::MAX) + 1);

    // SAFETY: `OkLab` is `repr(C, align(16))` with four `f32` fields; a
    // contiguous 4-lane load from its address is well-defined. The `_pad`
    // field is zero-initialised by every public constructor.
    let pixel_vec = unsafe { vld1q_f32(core::ptr::from_ref(&pixel).cast::<f32>()) };

    let mut best_idx: usize = 0;
    let mut best_dist = f32::INFINITY;

    for (idx, centroid) in centroids.iter().enumerate() {
        // SAFETY: see `pixel_vec` above.
        let c_vec = unsafe { vld1q_f32(core::ptr::from_ref(centroid).cast::<f32>()) };
        // SAFETY: NEON intrinsics over fully-initialised lane vectors.
        let diff = unsafe { vsubq_f32(pixel_vec, c_vec) };
        // SAFETY: NEON intrinsic.
        let sq = unsafe { vmulq_f32(diff, diff) };
        // SAFETY: `hsum3` reads the first three lanes via `vgetq_lane_f32`.
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
/// # Safety
///
/// Requires NEON.
#[target_feature(enable = "neon")]
#[inline]
unsafe fn hsum3(v: float32x4_t) -> f32 {
    // SAFETY: NEON intrinsic; extracts the four scalar lanes individually.
    let l0 = unsafe { vgetq_lane_f32::<0>(v) };
    // SAFETY: NEON intrinsic.
    let l1 = unsafe { vgetq_lane_f32::<1>(v) };
    // SAFETY: NEON intrinsic.
    let l2 = unsafe { vgetq_lane_f32::<2>(v) };
    l0 + l1 + l2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_scalar_for_dense_palette() {
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
                    // SAFETY: NEON is always present on aarch64.
                    let neon = unsafe { nearest_centroid(pixel, &palette) };
                    assert_eq!(scalar, neon, "rgb=({r},{g},{b})");
                }
            }
        }
    }
}
