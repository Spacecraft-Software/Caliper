// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-temper
//!
//! Path-fitting stage. Hardens raw contours into final curves: O(n)
//! least-squares Bézier fitting (VTracer-lineage), Selinger segment-merging
//! curve optimisation, and parallel RANSAC primitive recognition (circle,
//! ellipse, rectangle, rounded-rectangle, star).
//!
//! Path modes: `pixel` (no fitting), `polygon` (linear), `spline` (full Bézier).
//! Optional Touchstone NPU shape classification when the `npu` feature is on.
//!
//! See [PRD §8.6].
//!
//! [PRD §8.6]: https://Caliper.Steelbore.com/prd#86-stage-5--path-fitting

#![forbid(unsafe_code)]

// O(n) Bézier fitter, Selinger optimisation, RANSAC primitives, and symmetry
// detection land per v0.1.0 §1.7 and v0.5.0 §5 of TODO.md.
