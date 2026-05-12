// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-etch
//!
//! Edge-detection stage. Provides:
//!
//! - [`connected_components`] — union-find labelling of binary masks. Produces
//!   per-pixel region IDs that `caliper-burin` uses to trace contours.
//! - [`sobel_magnitude`] and [`scharr_magnitude`] — classical 3×3 separable
//!   gradient kernels.
//!
//! Layout-wise, masks and gradient buffers are row-major slices indexed by
//! `y * width + x`. This matches the layout produced by `caliper-smelt` and
//! consumed by `caliper-burin`.
//!
//! See [PRD §7.1] and [PRD §8.4].
//!
//! [PRD §7.1]: https://Caliper.Steelbore.com/prd#71-component-algorithms-no-reinvention
//! [PRD §8.4]: https://Caliper.Steelbore.com/prd#84-stage-3--edge-detection

#![forbid(unsafe_code)]

pub mod connected;
pub mod sobel;

pub use connected::{Connectivity, LabelMap, connected_components};
pub use sobel::{scharr_magnitude, sobel_magnitude};
