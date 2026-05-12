// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-cast
//!
//! Output-encoding stage. Streaming writers per format with configurable
//! decimal precision (`--path-precision`, default 3). UTF-8 without BOM. LF
//! line endings. Deterministic byte output — given the same `VectorDocument`
//! the writers always emit identical bytes.
//!
//! ## Formats by version
//!
//! | Format    | Extension | Shipped In | Feature gate |
//! |-----------|-----------|------------|--------------|
//! | SVG 1.1   | `.svg`    | v0.1.0     | `svg` (default) |
//! | PDF 1.7   | `.pdf`    | v0.2.0     | `pdf`        |
//! | EPS 3.0   | `.eps`    | v0.5.0     | `eps`        |
//! | DXF R14   | `.dxf`    | v0.5.0     | `dxf`        |
//!
//! See [PRD §8.8] and [PRD §13.2].
//!
//! [PRD §8.8]: https://Caliper.Steelbore.com/prd#88-stage-7--cast-output-encoding
//! [PRD §13.2]: https://Caliper.Steelbore.com/prd#132-output

#![forbid(unsafe_code)]

pub mod document;

#[cfg(feature = "svg")]
pub mod svg;

pub use document::{Layer, Path, RgbColor, VectorDocument};

#[cfg(feature = "svg")]
pub use svg::write_svg;
