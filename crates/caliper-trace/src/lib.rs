// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-trace
//!
//! Public library API surface for the Caliper raster-to-vector tracing engine.
//! Re-exports types from internal stage crates (`caliper-crucible`,
//! `caliper-assayer`, `caliper-cast`, optionally `caliper-bellows` and
//! `caliper-touchstone`).
//!
//! ## Example
//!
//! ```no_run
//! use caliper_trace::{TracingConfig, VectorDocument, VectorDocumentExt, trace};
//!
//! let img = image::open("logo.png").expect("open image");
//! let config = TracingConfig::default();
//! let doc: VectorDocument = trace(&img, &config).expect("trace");
//!
//! let svg_bytes = doc.to_svg_string().expect("emit SVG");
//! std::fs::write("logo.svg", svg_bytes).expect("write");
//! ```
//!
//! ## Features
//!
//! - `svg` (default) — Enables the streaming SVG 1.1 writer.
//! - `pdf` — Enables the PDF 1.7 writer (v0.2.0+).
//! - `gpu` — Enables the [`caliper-bellows`](https://crates.io/crates/caliper-bellows)
//!   wgpu backend.
//! - `npu` — Enables the [`caliper-touchstone`](https://crates.io/crates/caliper-touchstone)
//!   ONNX Runtime backend.
//! - `all` — Enables every feature above.
//!
//! See [PRD §11] for the full API contract.
//!
//! [PRD §11]: https://Caliper.Steelbore.com/prd#11-library-api

#![forbid(unsafe_code)]

pub use caliper_anvil::OkLab;
pub use caliper_cast::{Layer, Path, RgbColor, VectorDocument};
pub use caliper_crucible::{TracingConfig, pipeline::TraceError, trace_image};
pub use caliper_etch::Connectivity;

#[cfg(feature = "svg")]
pub use caliper_cast::svg::{SvgConfig, write_svg};

/// Top-level convenience: trace `img` according to `config` and return a
/// [`VectorDocument`]. Thin wrapper over [`caliper_crucible::trace_image`] —
/// exposed at the crate root so the canonical call site is `caliper_trace::trace`.
///
/// # Errors
///
/// Forwards any [`TraceError`] from the underlying pipeline.
pub fn trace(
    img: &image::DynamicImage,
    config: &TracingConfig,
) -> Result<VectorDocument, TraceError> {
    trace_image(img, config)
}

#[cfg(feature = "svg")]
impl VectorDocumentExt for VectorDocument {
    fn to_svg_string(&self) -> std::io::Result<String> {
        let mut buf = Vec::new();
        write_svg(&mut buf, self, &SvgConfig::default())?;
        // SVG is always ASCII / UTF-8; conversion is infallible.
        Ok(String::from_utf8(buf).expect("SVG writer emits UTF-8"))
    }
}

/// Convenience helpers on [`VectorDocument`] for the common "trace then save"
/// flow.
#[cfg(feature = "svg")]
pub trait VectorDocumentExt {
    /// Emit the document as an SVG 1.1 string with default precision.
    ///
    /// # Errors
    ///
    /// Propagates any I/O error from the underlying writer.
    fn to_svg_string(&self) -> std::io::Result<String>;
}

