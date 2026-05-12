// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-smelt
//!
//! Color quantization stage. Separates pixels into colour layers using
//! perceptually-uniform k-means in OKLab (default), with median-cut and octree
//! available as alternatives via `--quantizer`. See [PRD §8.3].
//!
//! Bypassed to Sauvola / Otsu adaptive thresholding when `--bw` is set.
//!
//! [PRD §8.3]: https://Caliper.Steelbore.com/prd#83-stage-2--color-quantization

#![forbid(unsafe_code)]

pub mod kmeans;
mod rng;

pub use kmeans::{KMeansConfig, QuantizeResult, quantize_kmeans};
