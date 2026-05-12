// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-burin
//!
//! Contour-tracing stage. Engraves boundary curves out of the per-region label
//! maps produced by `caliper-etch`. Uses Moore-neighbour boundary following —
//! a simplified Suzuki-Abe variant that produces ordered closed rings.
//!
//! Hole detection (true Suzuki-Abe with parent-child hierarchy) lands in
//! v0.2.0 §2.2 of TODO.md alongside multi-tile stitching. For v0.1.0 we only
//! trace outer boundaries — sufficient for the first end-to-end SVG.
//!
//! Visvalingam-Whyatt simplification reduces the raw chain-coded contour to a
//! polyline of significant points, the input to `caliper-temper`'s Bézier fit.
//!
//! See [PRD §7.1] and [PRD §8.5].
//!
//! [PRD §7.1]: https://Caliper.Steelbore.com/prd#71-component-algorithms-no-reinvention
//! [PRD §8.5]: https://Caliper.Steelbore.com/prd#85-stage-4--contour-tracing

#![forbid(unsafe_code)]

pub mod simplify;
pub mod trace;

pub use simplify::{visvalingam_whyatt, visvalingam_whyatt_until};
pub use trace::{Point, RegionContour, trace_outer_contours};
