// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-weld
//!
//! Stitching and assembly stage. Merges paths that cross tile boundaries via
//! overlap regions, performs layer stacking (`stacked` default, `cutout`
//! hierarchical), and applies final path optimisation. CPU-only.
//!
//! See [PRD §8.7].
//!
//! [PRD §8.7]: https://Caliper.Steelbore.com/prd#87-stage-6--stitching-and-assembly

#![forbid(unsafe_code)]

// Single-tile pass lands with v0.1.0 §1.9 of TODO.md; cross-tile stitching
// follows in v0.2.0 §2.2 alongside the three-level Rayon scheduler.
