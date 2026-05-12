// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! End-to-end trace pipeline for v0.1.0.
//!
//! Wires the stage crates together for the polygon-mode CPU path:
//!
//! ```text
//! image::DynamicImage
//!   │
//!   ▼ caliper-anvil::OkLab     (input.rs)
//! Vec<OkLab>
//!   │
//!   ▼ caliper-smelt::quantize_kmeans
//! palette + per-pixel labels
//!   │
//!   ▼ per-label binary mask + caliper-etch::connected_components
//! one LabelMap per palette colour
//!   │
//!   ▼ caliper-burin::trace_outer_contours
//! one RegionContour list per palette colour
//!   │
//!   ▼ caliper-burin::visvalingam_whyatt_until
//! simplified polylines
//!   │
//!   ▼ assemble caliper_cast::VectorDocument
//! VectorDocument ready for SVG / PDF / EPS / DXF emit
//! ```

use caliper_burin::{RegionContour, visvalingam_whyatt_until};
use caliper_cast::{Layer, Path, RgbColor, VectorDocument};
use caliper_etch::{Connectivity, connected_components};
use caliper_smelt::{KMeansConfig, quantize_kmeans};
use image::DynamicImage;
use thiserror::Error;

use crate::input::{image_to_oklab, mask_for_label};

/// Top-level tracing configuration. Mirrors the PRD §9.3 CLI flag surface,
/// minus flags that only matter for output formats we haven't shipped yet.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Number of palette colours. Bounded `[2, 256]`.
    pub colors: usize,
    /// Visvalingam-Whyatt tolerance in squared-pixel units.
    pub simplify_tolerance: f32,
    /// Filter out contours with fewer than this many vertices after simplification.
    /// Eliminates pixel-level speckle from quantisation.
    pub filter_speckle: usize,
    /// Connectivity for the region-labeling pass.
    pub connectivity: Connectivity,
    /// Seed for k-means++ initialisation. Default `0` for reproducibility.
    pub seed: u64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            colors: 16,
            simplify_tolerance: 1.0,
            filter_speckle: 4,
            connectivity: Connectivity::Four,
            seed: 0,
        }
    }
}

/// Errors that can happen during tracing.
#[derive(Debug, Error)]
pub enum TraceError {
    /// Empty input image.
    #[error("input image has zero dimensions")]
    EmptyImage,
    /// Invalid `colors` value — outside `[2, 256]`.
    #[error("colors must be in 2..=256 (got {got})")]
    InvalidColors {
        /// The invalid value supplied by the caller.
        got: usize,
    },
    /// Underlying k-means failure (empty input, bad palette size, etc).
    #[error("quantization failed: {0}")]
    Quantize(#[from] caliper_smelt::kmeans::KMeansError),
}

/// Trace `img` end-to-end into a [`VectorDocument`].
///
/// # Errors
///
/// - [`TraceError::EmptyImage`] if `img` has zero width or height.
/// - [`TraceError::InvalidColors`] if `config.colors` is outside `[2, 256]`.
/// - [`TraceError::Quantize`] on any k-means failure.
pub fn trace_image(
    img: &DynamicImage,
    config: &TracingConfig,
) -> Result<VectorDocument, TraceError> {
    let (w, h) = image::GenericImageView::dimensions(img);
    if w == 0 || h == 0 {
        return Err(TraceError::EmptyImage);
    }
    if !(2..=256).contains(&config.colors) {
        return Err(TraceError::InvalidColors { got: config.colors });
    }

    // Stage 1 — decode and convert to OKLab.
    let pixels = image_to_oklab(img);

    // Stage 2 — k-means quantization in OKLab.
    let qr = quantize_kmeans(
        &pixels,
        &KMeansConfig {
            palette_size: config.colors,
            seed: config.seed,
            ..KMeansConfig::default()
        },
    )?;

    // Per-layer: build mask → CC label → trace contours → simplify → emit polygons.
    let mut document = VectorDocument::empty(w, h);
    for label_idx in 0_u8..qr.palette.len().min(256) as u8 {
        // Stage 3a — build a binary mask of this label.
        let mask = mask_for_label(&qr.labels, label_idx);
        if !mask.iter().any(|&b| b != 0) {
            // Centroid present in palette but no pixels actually use it
            // (can happen with degenerate k-means inputs); skip.
            continue;
        }

        // Stage 3b — connected components.
        let lm = connected_components(&mask, w as usize, h as usize, config.connectivity);

        // Stage 4 — Moore-neighbour boundary trace.
        let raw = caliper_burin::trace_outer_contours(&lm);

        // Stages 5–6 — simplify and emit polygons.
        let mut paths = Vec::with_capacity(raw.len());
        for RegionContour { points, .. } in raw {
            let int_pts = visvalingam_whyatt_until(&points, config.simplify_tolerance);
            if int_pts.len() < config.filter_speckle {
                continue;
            }
            // Convert integer contour to f32 path coordinates.
            let mut f_pts: Vec<(f32, f32)> = int_pts
                .iter()
                .map(|p| (p.x as f32, p.y as f32))
                .collect();
            // Make sure the path is closed for the SVG writer.
            if let (Some(&first), Some(&last)) = (f_pts.first(), f_pts.last()) {
                if first != last {
                    f_pts.push(first);
                }
            }
            paths.push(Path { points: f_pts });
        }
        if paths.is_empty() {
            continue;
        }

        // Convert the centroid (an OKLab colour) back to sRGB for output.
        let color = oklab_to_srgb_u8(qr.palette[usize::from(label_idx)]);
        document.layers.push(Layer { color, paths });
    }

    Ok(document)
}

/// Convert an OKLab colour to 8-bit sRGB for output. Round-trips
/// `OkLab::from_srgb_u8` to within 1 ULP for in-gamut colours.
fn oklab_to_srgb_u8(c: caliper_anvil::OkLab) -> RgbColor {
    // Reverse of Björn Ottosson's transform (see anvil/src/oklab.rs).
    let l_ = c.l + 0.396_337_777_4 * c.a + 0.215_803_757_9 * c.b;
    let m_ = c.l - 0.105_561_345_8 * c.a - 0.063_854_172_8 * c.b;
    let s_ = c.l - 0.089_484_177_5 * c.a - 1.291_485_548_0 * c.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r_lin = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_9 * s;
    let g_lin = -1.268_438_0 * l + 2.609_757_4 * m - 0.341_319_4 * s;
    let b_lin = -0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s;

    RgbColor::new(
        encode_srgb(r_lin),
        encode_srgb(g_lin),
        encode_srgb(b_lin),
    )
}

/// sRGB OETF (encode linear-light to sRGB-encoded `[0, 255]`).
fn encode_srgb(linear: f32) -> u8 {
    let clamped = linear.clamp(0.0, 1.0);
    let nonlinear = if clamped <= 0.003_130_8 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to [0,1] then scaled to [0,255]; cast is well-defined"
    )]
    let byte = (nonlinear * 255.0 + 0.5) as u8;
    byte
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    /// Build a simple 2-colour image: top half red, bottom half blue.
    fn two_color_image() -> DynamicImage {
        let w = 16;
        let h = 16;
        let img = ImageBuffer::from_fn(w, h, |_x, y| {
            if y < h / 2 {
                Rgb([255, 0, 0])
            } else {
                Rgb([0, 0, 255])
            }
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn end_to_end_two_color_image() {
        let img = two_color_image();
        let cfg = TracingConfig {
            colors: 2,
            simplify_tolerance: 0.5,
            filter_speckle: 4,
            ..TracingConfig::default()
        };
        let doc = trace_image(&img, &cfg).expect("trace must succeed");

        assert_eq!(doc.width, 16);
        assert_eq!(doc.height, 16);
        // Two distinct layers (red and blue).
        assert_eq!(doc.layers.len(), 2);

        // Each layer should have exactly one path (one connected region).
        for layer in &doc.layers {
            assert_eq!(layer.paths.len(), 1);
        }

        // The layer colours should be near the input colours.
        let colors: Vec<RgbColor> = doc.layers.iter().map(|l| l.color).collect();
        let near_red = colors
            .iter()
            .any(|c| c.r > 200 && c.g < 60 && c.b < 60);
        let near_blue = colors
            .iter()
            .any(|c| c.r < 60 && c.g < 60 && c.b > 200);
        assert!(near_red, "no near-red layer: {colors:?}");
        assert!(near_blue, "no near-blue layer: {colors:?}");
    }

    #[test]
    fn empty_image_errors() {
        // image::DynamicImage::new_rgb8(0, 0) panics — use a 1×1 then resize to 0×0
        // by constructing the buffer manually.
        let img = DynamicImage::ImageRgb8(ImageBuffer::new(0, 0));
        let err = trace_image(&img, &TracingConfig::default()).unwrap_err();
        assert!(matches!(err, TraceError::EmptyImage));
    }

    #[test]
    fn invalid_colors_count_errors() {
        let img = two_color_image();
        let err = trace_image(
            &img,
            &TracingConfig {
                colors: 1,
                ..TracingConfig::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, TraceError::InvalidColors { got: 1 }));
    }

    #[test]
    fn determinism_holds_end_to_end() {
        let img = two_color_image();
        let cfg = TracingConfig {
            colors: 2,
            seed: 0xCAFE_F00D,
            ..TracingConfig::default()
        };
        let a = trace_image(&img, &cfg).unwrap();
        let b = trace_image(&img, &cfg).unwrap();
        assert_eq!(a.layers.len(), b.layers.len());
        for (la, lb) in a.layers.iter().zip(b.layers.iter()) {
            assert_eq!(la.color, lb.color);
            assert_eq!(la.paths.len(), lb.paths.len());
            for (pa, pb) in la.paths.iter().zip(lb.paths.iter()) {
                assert_eq!(pa.points, pb.points);
            }
        }
    }
}
