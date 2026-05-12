// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! k-means clustering in OKLab colour space.
//!
//! Uses k-means++ initialisation (Arthur & Vassilvitskii 2007) for
//! deterministic, well-spread seeds, followed by classic Lloyd's iterations.
//! See [PRD §7.1] and [PRD §8.3].
//!
//! ## Determinism
//!
//! Identical `(pixels, KMeansConfig)` inputs always produce the identical
//! palette. The seed in [`KMeansConfig`] is the only source of randomness;
//! when not overridden it is fixed at `0`.
//!
//! [PRD §7.1]: https://Caliper.Steelbore.com/prd#71-component-algorithms-no-reinvention
//! [PRD §8.3]: https://Caliper.Steelbore.com/prd#83-stage-2--color-quantization

use caliper_anvil::{CentroidIndex, OkLab, nearest_centroid_scalar};
use thiserror::Error;

use crate::rng::Xoshiro256pp;

/// Maximum palette size — bounded by [`CentroidIndex`] (`u8`).
pub const MAX_PALETTE_SIZE: usize = 256;

/// Configuration knobs for [`quantize_kmeans`].
///
/// Sensible defaults exposed via [`KMeansConfig::default`].
#[derive(Debug, Clone)]
pub struct KMeansConfig {
    /// Number of palette entries to generate. Bounded at [`MAX_PALETTE_SIZE`].
    pub palette_size: usize,
    /// Maximum number of Lloyd iterations. Convergence usually arrives in <20.
    pub max_iterations: u32,
    /// Stop once the centroid shift falls below this in OKLab squared distance.
    pub convergence_threshold: f32,
    /// Seed for k-means++ initialisation. Fixed at 0 by default for
    /// reproducibility.
    pub seed: u64,
}

impl Default for KMeansConfig {
    fn default() -> Self {
        Self {
            palette_size: 16,
            max_iterations: 50,
            convergence_threshold: 1e-5,
            seed: 0,
        }
    }
}

/// Result of [`quantize_kmeans`].
#[derive(Debug, Clone)]
pub struct QuantizeResult {
    /// Final palette — `palette_size` colours in OKLab space.
    pub palette: Vec<OkLab>,
    /// Per-pixel centroid assignment, length matches the input pixel slice.
    pub labels: Vec<CentroidIndex>,
    /// Number of Lloyd iterations actually run.
    pub iterations_run: u32,
    /// Final inertia (sum of squared distances from each pixel to its centroid).
    /// Smaller is better; used by `--colors auto` to score palette sizes.
    pub inertia: f32,
}

/// Errors that can prevent quantisation from running.
#[derive(Debug, Error)]
pub enum KMeansError {
    /// Input was empty — at least one pixel is required.
    #[error("input pixel slice is empty")]
    EmptyInput,
    /// Requested palette size is zero or exceeds [`MAX_PALETTE_SIZE`].
    #[error(
        "palette size must be in 1..={max} (got {got})",
        max = MAX_PALETTE_SIZE
    )]
    InvalidPaletteSize {
        /// The palette size that was requested.
        got: usize,
    },
}

/// Quantise an OKLab pixel buffer into a palette of `config.palette_size`
/// entries via k-means++ initialisation + Lloyd's iterations.
///
/// # Errors
///
/// Returns [`KMeansError::EmptyInput`] if `pixels` is empty,
/// [`KMeansError::InvalidPaletteSize`] if `palette_size` is zero or exceeds
/// [`MAX_PALETTE_SIZE`].
pub fn quantize_kmeans(
    pixels: &[OkLab],
    config: &KMeansConfig,
) -> Result<QuantizeResult, KMeansError> {
    if pixels.is_empty() {
        return Err(KMeansError::EmptyInput);
    }
    if config.palette_size == 0 || config.palette_size > MAX_PALETTE_SIZE {
        return Err(KMeansError::InvalidPaletteSize {
            got: config.palette_size,
        });
    }

    // Cap palette size at the number of unique pixels — otherwise k-means++
    // can pick duplicate seeds.
    let effective_k = config.palette_size.min(pixels.len());

    let mut rng = Xoshiro256pp::new(config.seed);
    let mut palette = kpp_init(pixels, effective_k, &mut rng);
    let mut labels = vec![0_u8; pixels.len()];

    let mut iterations_run = 0;
    let mut final_inertia = f32::INFINITY;

    for iter in 0..config.max_iterations {
        iterations_run = iter + 1;

        // Step 1 — assign each pixel to its nearest centroid.
        assign_labels(pixels, &palette, &mut labels);

        // Step 2 — recompute centroids as the mean of their assigned pixels.
        let (new_palette, inertia) = recompute_centroids(pixels, &labels, &palette);
        final_inertia = inertia;

        // Step 3 — check convergence by max centroid shift.
        let max_shift = palette
            .iter()
            .zip(new_palette.iter())
            .map(|(a, b)| a.distance_sq(*b))
            .fold(0.0_f32, f32::max);

        palette = new_palette;
        if max_shift < config.convergence_threshold {
            break;
        }
    }

    // Final label pass against the converged palette so labels match the
    // returned palette exactly.
    assign_labels(pixels, &palette, &mut labels);

    Ok(QuantizeResult {
        palette,
        labels,
        iterations_run,
        inertia: final_inertia,
    })
}

/// k-means++ seeding — picks an initial palette where each new centroid is
/// drawn with probability proportional to the squared distance from the
/// already-chosen centroids. Produces well-separated seeds, dramatically
/// reducing the number of Lloyd iterations needed to converge.
///
/// Reference: Arthur & Vassilvitskii, "k-means++: The Advantages of Careful
/// Seeding" (SODA 2007).
fn kpp_init(pixels: &[OkLab], k: usize, rng: &mut Xoshiro256pp) -> Vec<OkLab> {
    debug_assert!(!pixels.is_empty() && k > 0 && k <= pixels.len());
    let mut palette = Vec::with_capacity(k);

    // Pick the first centroid uniformly at random.
    palette.push(pixels[rng.next_below(pixels.len())]);

    // Sum of squared distances to the nearest existing centroid, per pixel.
    let mut min_sq = vec![f32::INFINITY; pixels.len()];
    update_min_sq(pixels, &palette, &mut min_sq);

    while palette.len() < k {
        let total: f32 = min_sq.iter().sum();
        if total <= 0.0 {
            // All remaining pixels coincide with existing centroids — fill
            // with duplicates so the caller still gets `k` entries.
            palette.push(pixels[rng.next_below(pixels.len())]);
            continue;
        }
        let target = rng.next_f32() * total;
        let mut acc = 0.0_f32;
        let mut pick = pixels.len() - 1;
        for (idx, weight) in min_sq.iter().enumerate() {
            acc += *weight;
            if acc >= target {
                pick = idx;
                break;
            }
        }
        palette.push(pixels[pick]);
        update_min_sq(pixels, &palette, &mut min_sq);
    }

    palette
}

/// Update `min_sq[i]` so it holds the squared distance from `pixels[i]` to
/// the closest entry in `palette`. The latest palette entry alone is enough;
/// we only need to compare against it.
fn update_min_sq(pixels: &[OkLab], palette: &[OkLab], min_sq: &mut [f32]) {
    debug_assert_eq!(pixels.len(), min_sq.len());
    let latest = *palette.last().expect("palette must be non-empty");
    for (pixel, slot) in pixels.iter().zip(min_sq.iter_mut()) {
        let d = pixel.distance_sq(latest);
        if d < *slot {
            *slot = d;
        }
    }
}

/// Assign each pixel to its nearest centroid. Writes into `labels` in place
/// so the caller can reuse a buffer across iterations.
fn assign_labels(pixels: &[OkLab], palette: &[OkLab], labels: &mut [CentroidIndex]) {
    debug_assert_eq!(pixels.len(), labels.len());
    // We use the scalar dispatcher here rather than `nearest_centroid` to
    // keep the helper deterministic regardless of host CPU. The Rayon-parallel
    // SIMD path lands with v0.2.0 §2.1 of TODO.md.
    for (pixel, label) in pixels.iter().zip(labels.iter_mut()) {
        *label = nearest_centroid_scalar(*pixel, palette);
    }
}

/// Compute the mean of all pixels assigned to each centroid plus the total
/// inertia (sum of squared distances).
///
/// `prev_palette` provides the fallback colour for any centroid that ended
/// up with zero assigned pixels — Lloyd's algorithm is undefined for empty
/// clusters; keeping the previous position is the standard handling.
fn recompute_centroids(
    pixels: &[OkLab],
    labels: &[CentroidIndex],
    prev_palette: &[OkLab],
) -> (Vec<OkLab>, f32) {
    debug_assert_eq!(pixels.len(), labels.len());
    let k = prev_palette.len();

    let mut sum_l = vec![0.0_f64; k];
    let mut sum_a = vec![0.0_f64; k];
    let mut sum_b = vec![0.0_f64; k];
    let mut count = vec![0_u64; k];

    for (pixel, &label) in pixels.iter().zip(labels.iter()) {
        let idx = usize::from(label);
        sum_l[idx] += f64::from(pixel.l);
        sum_a[idx] += f64::from(pixel.a);
        sum_b[idx] += f64::from(pixel.b);
        count[idx] += 1;
    }

    let mut new_palette = Vec::with_capacity(k);
    for i in 0..k {
        if count[i] == 0 {
            new_palette.push(prev_palette[i]);
        } else {
            #[expect(
                clippy::cast_precision_loss,
                reason = "count fits comfortably in f64 precision for image-sized inputs"
            )]
            let n = count[i] as f64;
            new_palette.push(OkLab::new(
                (sum_l[i] / n) as f32,
                (sum_a[i] / n) as f32,
                (sum_b[i] / n) as f32,
            ));
        }
    }

    let mut inertia = 0.0_f32;
    for (pixel, &label) in pixels.iter().zip(labels.iter()) {
        inertia += pixel.distance_sq(new_palette[usize::from(label)]);
    }

    (new_palette, inertia)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_palette(k: usize) -> Vec<OkLab> {
        (0..k)
            .map(|i| {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "i bounded at MAX_PALETTE_SIZE"
                )]
                let t = i as f32 / k.max(1) as f32;
                OkLab::new(t, t.mul_add(0.4, -0.2), t.mul_add(-0.3, 0.15))
            })
            .collect()
    }

    /// Build a synthetic image with N copies of each palette colour, lightly
    /// perturbed so k-means has work to do.
    fn synthetic_image(palette: &[OkLab], copies_per_color: usize, jitter: f32) -> Vec<OkLab> {
        let mut pixels = Vec::with_capacity(palette.len() * copies_per_color);
        for (i, c) in palette.iter().enumerate() {
            for j in 0..copies_per_color {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "j bounded at small test sizes"
                )]
                let dl = jitter * ((j as f32).mul_add(0.13, i as f32 * 0.07).sin());
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "j bounded at small test sizes"
                )]
                let da = jitter * ((j as f32).mul_add(0.17, i as f32 * 0.11).cos());
                pixels.push(OkLab::new(c.l + dl, c.a + da, c.b));
            }
        }
        pixels
    }

    #[test]
    fn empty_input_errors() {
        let result = quantize_kmeans(&[], &KMeansConfig::default());
        assert!(matches!(result, Err(KMeansError::EmptyInput)));
    }

    #[test]
    fn zero_palette_errors() {
        let pixels = vec![OkLab::new(0.5, 0.0, 0.0)];
        let result = quantize_kmeans(
            &pixels,
            &KMeansConfig {
                palette_size: 0,
                ..Default::default()
            },
        );
        assert!(matches!(
            result,
            Err(KMeansError::InvalidPaletteSize { got: 0 })
        ));
    }

    #[test]
    fn oversized_palette_errors() {
        let pixels = vec![OkLab::new(0.5, 0.0, 0.0)];
        let result = quantize_kmeans(
            &pixels,
            &KMeansConfig {
                palette_size: 257,
                ..Default::default()
            },
        );
        assert!(matches!(
            result,
            Err(KMeansError::InvalidPaletteSize { got: 257 })
        ));
    }

    #[test]
    fn recovers_synthetic_palette() {
        // 8-colour synthetic palette with 64 lightly-jittered copies of each.
        let true_palette = synthetic_palette(8);
        let pixels = synthetic_image(&true_palette, 64, 0.005);

        let result = quantize_kmeans(
            &pixels,
            &KMeansConfig {
                palette_size: 8,
                max_iterations: 100,
                ..Default::default()
            },
        )
        .expect("k-means must succeed on non-empty input");

        assert_eq!(result.palette.len(), 8);
        assert_eq!(result.labels.len(), pixels.len());
        // The recovered palette should sit within `jitter` of the true palette
        // (modulo permutation). For each true centroid, find the closest
        // recovered centroid and assert the distance is small.
        for true_c in &true_palette {
            let best = result
                .palette
                .iter()
                .map(|p| p.distance_sq(*true_c))
                .fold(f32::INFINITY, f32::min);
            assert!(best < 0.001, "true centroid {true_c:?} missing; best dist² = {best}");
        }
    }

    #[test]
    fn deterministic_with_fixed_seed() {
        let pixels = synthetic_image(&synthetic_palette(8), 64, 0.005);

        let config = KMeansConfig {
            palette_size: 8,
            max_iterations: 50,
            seed: 0xDEAD_BEEF,
            ..Default::default()
        };

        let a = quantize_kmeans(&pixels, &config).unwrap();
        let b = quantize_kmeans(&pixels, &config).unwrap();

        assert_eq!(a.palette, b.palette);
        assert_eq!(a.labels, b.labels);
        assert_eq!(a.iterations_run, b.iterations_run);
    }

    #[test]
    fn inertia_decreases_with_more_palette_entries() {
        let pixels = synthetic_image(&synthetic_palette(16), 32, 0.02);

        let small = quantize_kmeans(
            &pixels,
            &KMeansConfig {
                palette_size: 4,
                max_iterations: 50,
                ..Default::default()
            },
        )
        .unwrap();
        let large = quantize_kmeans(
            &pixels,
            &KMeansConfig {
                palette_size: 16,
                max_iterations: 50,
                ..Default::default()
            },
        )
        .unwrap();

        assert!(
            large.inertia < small.inertia,
            "inertia must strictly decrease with more centroids; small={} large={}",
            small.inertia, large.inertia
        );
    }
}
