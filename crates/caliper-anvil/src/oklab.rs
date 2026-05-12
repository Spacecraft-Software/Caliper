// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! OKLab colour space — perceptually-uniform LCh-like space.
//!
//! Caliper uses OKLab for colour-distance comparisons inside k-means because
//! it is *the* perceptually uniform space cheap enough to compute per-pixel
//! (Björn Ottosson, 2020). Euclidean distance in OKLab approximates perceived
//! colour difference much better than Euclidean distance in sRGB.
//!
//! See <https://bottosson.github.io/posts/oklab/> for the original derivation
//! and `CREDITS.md` for the attribution.

/// A colour in the OKLab space.
///
/// Components are stored as `f32` and follow Ottosson's conventions:
///
/// - `l` (lightness) — `0.0` (black) to `~1.0` (white).
/// - `a` (green ↔ red) — typically `[-0.4, +0.4]` for sRGB-displayable colours.
/// - `b` (blue ↔ yellow) — typically `[-0.4, +0.4]` for sRGB-displayable colours.
///
/// The struct is `repr(C)` and packed (`l`, `a`, `b`, padding) so a `&[OkLab]`
/// can be reinterpreted as a `&[f32]` of length `4 * n` when needed by SIMD
/// kernels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C, align(16))]
pub struct OkLab {
    /// Lightness component, `[0.0, 1.0]` for sRGB-displayable colours.
    pub l: f32,
    /// Green ↔ red component.
    pub a: f32,
    /// Blue ↔ yellow component.
    pub b: f32,
    /// Reserved for alpha or padding; not used in distance calculations.
    pub _pad: f32,
}

impl OkLab {
    /// Construct a new [`OkLab`] colour with the given components.
    ///
    /// The padding field is zero-initialised so equality is well-defined.
    #[inline]
    #[must_use]
    pub const fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b, _pad: 0.0 }
    }

    /// Squared Euclidean distance to another colour, in OKLab space.
    ///
    /// k-means cares about ordering of distances, not absolute values, so the
    /// `sqrt` is omitted to avoid a per-pixel transcendental call.
    #[inline]
    #[must_use]
    pub fn distance_sq(self, other: Self) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        dl.mul_add(dl, da.mul_add(da, db * db))
    }

    /// Convert from linear sRGB (each channel in `[0.0, 1.0]`) to OKLab.
    ///
    /// Implements Björn Ottosson's 2020 derivation directly. The matrix
    /// constants here are taken verbatim from
    /// <https://bottosson.github.io/posts/oklab/>.
    #[inline]
    #[must_use]
    #[expect(
        clippy::many_single_char_names,
        reason = "matching Ottosson's published reference formulation"
    )]
    pub fn from_linear_srgb(r: f32, g: f32, b: f32) -> Self {
        // First matrix: linear sRGB to LMS.
        let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

        // Non-linearity — cube root in LMS to compress dynamic range.
        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        // Second matrix: non-linear LMS to OKLab.
        Self::new(
            0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
            1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
            0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
        )
    }

    /// Convert from non-linear sRGB (sRGB-encoded, each channel `[0, 255]`) to OKLab.
    ///
    /// Applies the standard sRGB EOTF (electro-optical transfer function) to
    /// each channel before the linear-sRGB → OKLab transform.
    #[inline]
    #[must_use]
    pub fn from_srgb_u8(r: u8, g: u8, b: u8) -> Self {
        let lr = srgb_eotf(f32::from(r) / 255.0);
        let lg = srgb_eotf(f32::from(g) / 255.0);
        let lb = srgb_eotf(f32::from(b) / 255.0);
        Self::from_linear_srgb(lr, lg, lb)
    }
}

/// The standard sRGB electro-optical transfer function.
///
/// `x` is the sRGB-encoded value in `[0.0, 1.0]`; the return is linear-light
/// intensity in `[0.0, 1.0]`. See IEC 61966-2-1.
#[inline]
#[must_use]
pub fn srgb_eotf(x: f32) -> f32 {
    // 0.04045 is the inflection point above which the curve is a 2.4 gamma;
    // below it, the curve is linear with slope 1/12.92 to avoid noise near
    // black. Constants are from the IEC 61966-2-1 standard.
    if x <= 0.040_45 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference values computed against the canonical Ottosson implementation.
    /// Tolerance is 1e-3 to absorb f32 rounding across the matrix chain.
    #[test]
    fn from_srgb_white_is_l_approximately_one() {
        let white = OkLab::from_srgb_u8(255, 255, 255);
        assert!((white.l - 1.0).abs() < 1e-3, "L={} for white", white.l);
        assert!(white.a.abs() < 1e-3);
        assert!(white.b.abs() < 1e-3);
    }

    #[test]
    fn from_srgb_black_is_origin() {
        let black = OkLab::from_srgb_u8(0, 0, 0);
        assert!(black.l.abs() < 1e-6);
        assert!(black.a.abs() < 1e-6);
        assert!(black.b.abs() < 1e-6);
    }

    #[test]
    fn distance_sq_is_zero_for_same_color() {
        let c = OkLab::new(0.5, 0.1, -0.05);
        assert!(c.distance_sq(c) < 1e-12);
    }

    #[test]
    fn distance_sq_is_symmetric() {
        let a = OkLab::new(0.5, 0.1, -0.05);
        let b = OkLab::new(0.3, -0.1, 0.2);
        assert!((a.distance_sq(b) - b.distance_sq(a)).abs() < 1e-6);
    }
}
