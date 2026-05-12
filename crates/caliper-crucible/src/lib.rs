// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-crucible
//!
//! Pipeline orchestrator for Caliper. Holds the molten work — receives a
//! [`TracingConfig`] from the caller, drives the seven-stage DAG across the
//! stage crates, and emits a [`caliper_cast::VectorDocument`].
//!
//! For v0.1.0 this orchestrator implements the **polygon-mode CPU path**:
//! decode → OKLab → k-means quantise → per-layer connected components →
//! Moore-neighbour boundary trace → Visvalingam-Whyatt simplify → emit. Spline
//! fitting (`caliper-temper`) and GPU/NPU offloads land in later milestones.
//!
//! See [PRD §5] and [PRD §8].
//!
//! [PRD §5]: https://Caliper.Steelbore.com/prd#5-architecture-overview
//! [PRD §8]: https://Caliper.Steelbore.com/prd#8-pipeline-stages-detailed

#![forbid(unsafe_code)]

pub mod input;
pub mod pipeline;

pub use pipeline::{TracingConfig, trace_image};
