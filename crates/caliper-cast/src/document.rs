// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Format-neutral vector document.
//!
//! Every Caliper output format consumes the same [`VectorDocument`]. Writers
//! (`svg`, `pdf`, `eps`, `dxf`) are thin format-specific encoders over this
//! representation.

/// An 8-bit-per-channel sRGB colour with no alpha. Used as the fill colour
/// per path. Caliper's palette is colour-only; transparency is per-document
/// not per-path (set via `--background`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    /// Red channel `[0, 255]`.
    pub r: u8,
    /// Green channel `[0, 255]`.
    pub g: u8,
    /// Blue channel `[0, 255]`.
    pub b: u8,
}

impl RgbColor {
    /// Construct from individual channel bytes.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// A single path — an ordered ring of points. Closed (first point repeated
/// at the end). For v0.1.0 paths are polylines; spline data joins in v0.2.0
/// via `caliper-temper`.
#[derive(Debug, Clone)]
pub struct Path {
    /// Vertex list in document coordinates.
    pub points: Vec<(f32, f32)>,
}

/// One palette colour and every path filled with it.
#[derive(Debug, Clone)]
pub struct Layer {
    /// Fill colour.
    pub color: RgbColor,
    /// All paths in this layer. Z-order is the position in this slice.
    pub paths: Vec<Path>,
}

/// Top-level vector document — the format-neutral output of the pipeline.
#[derive(Debug, Clone)]
pub struct VectorDocument {
    /// Width of the underlying raster, in pixels.
    pub width: u32,
    /// Height of the underlying raster, in pixels.
    pub height: u32,
    /// Per-colour layers, in z-order (background first, foreground last).
    pub layers: Vec<Layer>,
}

impl VectorDocument {
    /// Construct an empty document of the given dimensions.
    #[must_use]
    pub fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            layers: Vec::new(),
        }
    }

    /// Total number of paths across all layers.
    #[must_use]
    pub fn path_count(&self) -> usize {
        self.layers.iter().map(|l| l.paths.len()).sum()
    }

    /// Total number of vertices across all paths.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.layers
            .iter()
            .flat_map(|l| l.paths.iter())
            .map(|p| p.points.len())
            .sum()
    }
}
