// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! End-to-end smoke test you can eyeball — generates a synthetic 4-colour
//! image, traces it, writes both the source PNG and the result SVG to
//! `examples/output/`.
//!
//! Usage:
//!
//! ```sh
//! cargo run --example trace_synthetic
//! ```

use std::fs;
use std::path::Path;

use caliper_trace::{TracingConfig, VectorDocumentExt, trace};
use image::{DynamicImage, ImageBuffer, Rgb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Write into the workspace `target/` directory so the artefacts stay
    // gitignored and never accidentally land in source control.
    let out_dir = Path::new("target/examples");
    fs::create_dir_all(out_dir)?;

    // 64x64 image: red top-left, green top-right, blue bottom-left, Steelbore
    // Molten Amber bottom-right. Steelbore palette swatch in raster form.
    let side = 64;
    let img: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_fn(side, side, |x, y| {
        let h = side / 2;
        match (x < h, y < h) {
            (true, true) => Rgb([0xFF, 0x5C, 0x5C]),  // Red Oxide
            (false, true) => Rgb([0x50, 0xFA, 0x7B]), // Radium Green
            (true, false) => Rgb([0x4B, 0x7E, 0xB0]), // Steel Blue
            (false, false) => Rgb([0xD9, 0x8E, 0x32]), // Molten Amber
        }
    });
    let dyn_img = DynamicImage::ImageRgb8(img);

    let png_path = out_dir.join("synthetic.png");
    dyn_img.save(&png_path)?;
    println!("wrote {}", png_path.display());

    let cfg = TracingConfig {
        colors: 4,
        simplify_tolerance: 0.5,
        filter_speckle: 4,
        ..TracingConfig::default()
    };
    let doc = trace(&dyn_img, &cfg)?;
    let svg = doc.to_svg_string()?;

    let svg_path = out_dir.join("synthetic.svg");
    fs::write(&svg_path, &svg)?;
    println!("wrote {}", svg_path.display());

    println!(
        "VectorDocument: {} layers, {} paths, {} vertices, {} bytes of SVG",
        doc.layers.len(),
        doc.path_count(),
        doc.vertex_count(),
        svg.len(),
    );

    Ok(())
}
