// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-assayer
//!
//! Adaptive pipeline router — the novel meta-algorithm in Caliper. The Assayer
//! measures input image characteristics (effective colour count, edge density,
//! noise level, image class, resolution, colour space, hardware capabilities)
//! and emits a [`PipelineConfig`] consumed by `caliper-crucible`.
//!
//! See [PRD §7.2] for the routing rationale and [PRD §7.3] for the
//! image-class routing table.
//!
//! [PRD §7.2]: https://Caliper.Steelbore.com/prd#72-the-assayer-adaptive-pipeline-router-novel-contribution
//! [PRD §7.3]: https://Caliper.Steelbore.com/prd#73-image-class-routing-table

#![forbid(unsafe_code)]

// Feature extractors, rule-based classifier, and PipelineConfig type land with
// v0.1.0 §1.3 of TODO.md.
