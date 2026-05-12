// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Image input adapters.
//!
//! Converts an [`image::DynamicImage`] into the OKLab pixel buffer that the
//! rest of the pipeline consumes.

use caliper_anvil::OkLab;
use image::{DynamicImage, GenericImageView};

/// Decode every pixel of `img` to OKLab. The output is row-major,
/// length `width * height`.
///
/// Alpha is ignored — fully-transparent pixels collapse to black before
/// conversion. v0.1.0 has no transparency model; the white-on-transparent
/// case is handled by `--background` at SVG emit time.
#[must_use]
pub fn image_to_oklab(img: &DynamicImage) -> Vec<OkLab> {
    let (w, h) = img.dimensions();
    let mut out = Vec::with_capacity((w as usize) * (h as usize));
    let rgb8 = img.to_rgb8();
    for pixel in rgb8.pixels() {
        let [r, g, b] = pixel.0;
        out.push(OkLab::from_srgb_u8(r, g, b));
    }
    out
}

/// Build a binary mask from a label slice, with `1` wherever the label equals
/// `target` and `0` elsewhere. Output is row-major to match the label slice.
#[must_use]
pub fn mask_for_label(labels: &[u8], target: u8) -> Vec<u8> {
    labels
        .iter()
        .map(|&l| if l == target { 1 } else { 0 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    #[test]
    fn small_white_image_round_trip() {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(2, 2, Rgb([255, 255, 255]));
        let dyn_img: DynamicImage = img.into();
        let pixels = image_to_oklab(&dyn_img);
        assert_eq!(pixels.len(), 4);
        for p in pixels {
            assert!((p.l - 1.0).abs() < 1e-3, "L={}", p.l);
        }
    }

    #[test]
    fn mask_picks_correct_label() {
        let labels = vec![0, 1, 2, 1, 1, 0];
        let mask = mask_for_label(&labels, 1);
        assert_eq!(mask, vec![0, 1, 0, 1, 1, 0]);
    }
}
