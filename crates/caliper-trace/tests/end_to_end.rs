// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! End-to-end integration tests for the Caliper public API.
//!
//! The v0.1.0 contract is: synthetic image → `trace(...)` → `VectorDocument`
//! → `to_svg_string()` → valid SVG markup containing path data for each layer.

use caliper_trace::{TracingConfig, VectorDocumentExt, trace};
use image::{DynamicImage, ImageBuffer, Rgb};

/// Build a small 4-quadrant test image — red/green/blue/black corners.
/// Lets us exercise the four-colour quantization + four-region trace.
fn four_quadrants(side: u32) -> DynamicImage {
    let img = ImageBuffer::from_fn(side, side, |x, y| {
        let h = side / 2;
        match (x < h, y < h) {
            (true, true) => Rgb([255, 0, 0]),    // TL: red
            (false, true) => Rgb([0, 255, 0]),   // TR: green
            (true, false) => Rgb([0, 0, 255]),   // BL: blue
            (false, false) => Rgb([0, 0, 0]),    // BR: black
        }
    });
    DynamicImage::ImageRgb8(img)
}

#[test]
fn four_quadrants_produces_four_layer_svg() {
    let img = four_quadrants(32);
    let doc = trace(
        &img,
        &TracingConfig {
            colors: 4,
            simplify_tolerance: 0.5,
            filter_speckle: 4,
            ..TracingConfig::default()
        },
    )
    .expect("trace should succeed");

    assert_eq!(doc.width, 32);
    assert_eq!(doc.height, 32);
    assert_eq!(doc.layers.len(), 4, "expected 4 colour layers");
    assert!(doc.path_count() >= 4, "expected at least one path per layer");

    let svg = doc.to_svg_string().expect("emit SVG");

    // Well-formed SVG envelope.
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>\n"));
    assert!(svg.contains(r#"width="32""#));
    assert!(svg.contains(r#"height="32""#));
    assert!(svg.contains(r#"viewBox="0 0 32 32""#));

    // At least four `<path` elements (one per layer).
    let path_count = svg.matches("<path ").count();
    assert!(path_count >= 4, "expected ≥ 4 <path> elements, found {path_count}\n{svg}");

    // Every layer should be a closed polygon — every path ends with `Z`.
    let z_count = svg.matches('Z').count();
    assert!(z_count >= 4, "expected ≥ 4 closed paths, got {z_count}");

    // No UTF-8 BOM (PRD §15 BLOCKER).
    let bytes = svg.as_bytes();
    assert!(bytes.len() >= 3);
    assert_ne!(&bytes[0..3], [0xEF, 0xBB, 0xBF], "SVG output must not start with a BOM");
}

#[test]
fn determinism_pixels_to_svg() {
    let img = four_quadrants(32);
    let cfg = TracingConfig {
        colors: 4,
        seed: 0xC0FFEE,
        ..TracingConfig::default()
    };
    let a = trace(&img, &cfg).unwrap().to_svg_string().unwrap();
    let b = trace(&img, &cfg).unwrap().to_svg_string().unwrap();
    assert_eq!(a, b, "identical inputs must produce identical SVG bytes");
}

#[test]
fn solid_color_image_produces_one_layer() {
    let img = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(8, 8, Rgb([100, 150, 200])));
    let doc = trace(
        &img,
        &TracingConfig {
            colors: 4,
            filter_speckle: 1,
            ..TracingConfig::default()
        },
    )
    .expect("trace solid image");

    // Even with colors=4 requested, a solid image can only produce one
    // distinct cluster of pixels.
    let non_empty_layers = doc.layers.iter().filter(|l| !l.paths.is_empty()).count();
    assert_eq!(non_empty_layers, 1, "solid image should produce one populated layer");

    let svg = doc.to_svg_string().unwrap();
    assert!(svg.contains("<path "));
}
