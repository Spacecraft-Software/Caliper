// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Sobel and Scharr 3×3 separable gradient kernels.
//!
//! Both produce gradient magnitude in `[0, 1]` for an input grayscale image
//! normalised to `[0, 1]`. The kernels are *separable* (vertical + horizontal
//! 1-D passes) so the inner loop touches only three samples per output pixel
//! per axis — friendly to both SIMD widening and cache locality.
//!
//! v0.1.0 ships scalar reference implementations. SIMD-accelerated paths
//! land via `caliper-anvil` in v0.2.0 §2.4 of TODO.md.
//!
//! See [PRD §7.1] and [PRD §8.4].
//!
//! [PRD §7.1]: https://Caliper.Steelbore.com/prd#71-component-algorithms-no-reinvention
//! [PRD §8.4]: https://Caliper.Steelbore.com/prd#84-stage-3--edge-detection

/// 3×3 Sobel gradient magnitude.
///
/// Kernels:
/// - Gx = `[-1 0 1; -2 0 2; -1 0 1]`
/// - Gy = `[-1 -2 -1; 0 0 0; 1 2 1]`
///
/// Output magnitude is `sqrt(Gx² + Gy²) / 4` so that a unit-step edge maps to
/// approximately 1.0. The factor of 4 normalises the kernel sum.
///
/// Border rows and columns receive zero output to avoid invalid memory access.
///
/// # Panics
///
/// Panics if `gray.len() != width * height`.
#[must_use]
pub fn sobel_magnitude(gray: &[f32], width: usize, height: usize) -> Vec<f32> {
    apply_3x3_gradient(gray, width, height, SOBEL_GX, SOBEL_GY, 1.0 / 4.0)
}

/// 3×3 Scharr gradient magnitude.
///
/// Kernels:
/// - Gx = `[-3 0 3; -10 0 10; -3 0 3]`
/// - Gy = `[-3 -10 -3; 0 0 0; 3 10 3]`
///
/// Scharr has better rotational symmetry than Sobel — at the cost of slightly
/// larger coefficients. Output normalised by `1/16`, the sum of one row of
/// positive coefficients.
///
/// # Panics
///
/// Panics if `gray.len() != width * height`.
#[must_use]
pub fn scharr_magnitude(gray: &[f32], width: usize, height: usize) -> Vec<f32> {
    apply_3x3_gradient(gray, width, height, SCHARR_GX, SCHARR_GY, 1.0 / 16.0)
}

const SOBEL_GX: [f32; 9] = [-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0];
const SOBEL_GY: [f32; 9] = [-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0];
const SCHARR_GX: [f32; 9] = [-3.0, 0.0, 3.0, -10.0, 0.0, 10.0, -3.0, 0.0, 3.0];
const SCHARR_GY: [f32; 9] = [-3.0, -10.0, -3.0, 0.0, 0.0, 0.0, 3.0, 10.0, 3.0];

/// Generic 3×3 gradient-magnitude kernel runner.
fn apply_3x3_gradient(
    gray: &[f32],
    width: usize,
    height: usize,
    kx: [f32; 9],
    ky: [f32; 9],
    scale: f32,
) -> Vec<f32> {
    assert_eq!(
        gray.len(),
        width * height,
        "grayscale length must equal width * height"
    );

    let mut out = vec![0.0_f32; gray.len()];
    if width < 3 || height < 3 {
        return out;
    }

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut gx = 0.0_f32;
            let mut gy = 0.0_f32;
            for dy in 0..3 {
                for dx in 0..3 {
                    let sample = gray[(y + dy - 1) * width + (x + dx - 1)];
                    let k_idx = dy * 3 + dx;
                    gx = sample.mul_add(kx[k_idx], gx);
                    gy = sample.mul_add(ky[k_idx], gy);
                }
            }
            out[y * width + x] = (gx * gx + gy * gy).sqrt() * scale;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constant images produce zero gradient everywhere.
    #[test]
    fn sobel_constant_image_is_zero() {
        let gray = vec![0.5_f32; 8 * 8];
        let mag = sobel_magnitude(&gray, 8, 8);
        for v in mag {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn scharr_constant_image_is_zero() {
        let gray = vec![0.5_f32; 8 * 8];
        let mag = scharr_magnitude(&gray, 8, 8);
        for v in mag {
            assert!(v.abs() < 1e-6);
        }
    }

    /// A vertical step edge — left half black, right half white. Sobel should
    /// produce a sharp positive response on the boundary column.
    #[test]
    fn sobel_detects_vertical_step() {
        let width = 8;
        let height = 8;
        let mut gray = vec![0.0_f32; width * height];
        for y in 0..height {
            for x in (width / 2)..width {
                gray[y * width + x] = 1.0;
            }
        }
        let mag = sobel_magnitude(&gray, width, height);

        // Boundary column (x = width/2 - 1 or x = width/2): high response.
        let boundary = mag[3 * width + (width / 2)];
        assert!(boundary > 0.5, "boundary mag={boundary}");

        // Far-left and far-right columns inside the safe area: low response.
        let far_left = mag[3 * width + 1];
        let far_right = mag[3 * width + (width - 2)];
        assert!(far_left.abs() < 1e-6);
        assert!(far_right.abs() < 1e-6);
    }

    /// Scharr should respond identically to vertical edges (rotational
    /// symmetry property — Scharr is "better than Sobel" precisely at oblique
    /// angles, but they agree on cardinal directions modulo scale).
    #[test]
    fn scharr_detects_horizontal_step() {
        let width = 8;
        let height = 8;
        let mut gray = vec![0.0_f32; width * height];
        for y in (height / 2)..height {
            for x in 0..width {
                gray[y * width + x] = 1.0;
            }
        }
        let mag = scharr_magnitude(&gray, width, height);

        // Boundary row.
        let boundary = mag[(height / 2) * width + 4];
        assert!(boundary > 0.5);

        // Top and bottom rows (in the safe area): low.
        let top = mag[1 * width + 4];
        let bottom = mag[(height - 2) * width + 4];
        assert!(top.abs() < 1e-6);
        assert!(bottom.abs() < 1e-6);
    }

    #[test]
    fn tiny_images_return_zero() {
        let gray = vec![0.0_f32, 1.0, 1.0, 0.0];
        let mag = sobel_magnitude(&gray, 2, 2);
        assert!(mag.iter().all(|&v| v == 0.0));
    }

    #[test]
    #[should_panic = "grayscale length must equal width * height"]
    fn wrong_size_panics() {
        let _ = sobel_magnitude(&[1.0, 2.0, 3.0], 2, 2);
    }
}
