// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Scalar reference implementations.
//!
//! Every SIMD kernel in anvil has a paired scalar implementation here.
//! Property tests assert SIMD == scalar so any divergence becomes a test
//! failure rather than a silent correctness bug.

use crate::{CentroidIndex, OkLab};

/// Scalar reference for [`crate::nearest_centroid`]. Linear scan, no SIMD.
#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "centroid count bounded at u8::MAX + 1 by the public API"
)]
pub(crate) fn nearest_centroid(pixel: OkLab, centroids: &[OkLab]) -> CentroidIndex {
    debug_assert!(!centroids.is_empty());
    debug_assert!(centroids.len() <= usize::from(CentroidIndex::MAX) + 1);

    let mut best_idx: usize = 0;
    let mut best_dist = f32::INFINITY;

    for (idx, c) in centroids.iter().enumerate() {
        let dist = pixel.distance_sq(*c);
        if dist < best_dist {
            best_dist = dist;
            best_idx = idx;
        }
    }
    best_idx as CentroidIndex
}
