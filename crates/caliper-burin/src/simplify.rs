// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Visvalingam-Whyatt polyline simplification.
//!
//! Each interior vertex has an "effective area" equal to the triangle formed
//! with its two neighbours. We repeatedly remove the vertex with the smallest
//! area, recompute the areas of the two adjacent vertices, and stop when the
//! polyline reaches a target size or the smallest area exceeds a tolerance
//! threshold.
//!
//! v0.1.0 uses a straightforward O(n²) implementation — adequate for the few
//! thousand points per contour we see on typical logos and line-art. The
//! heap-based O(n log n) variant lands when contours start exceeding ~10k
//! points (large maps / GIS data).
//!
//! Reference: Visvalingam, M. & Whyatt, J. D. — *Line generalisation by
//! repeated elimination of points* (Cartographic Journal 30(1), 1993).

use crate::trace::Point;

/// Simplify `points` until exactly `target` points remain.
///
/// `target` is clamped to `[2, points.len()]`. For closed rings (where the
/// first and last point coincide) pass the deduplicated open polyline plus
/// the closing duplicate; the function preserves whichever endpoints you
/// gave it.
#[must_use]
pub fn visvalingam_whyatt(points: &[Point], target: usize) -> Vec<Point> {
    let target = target.max(2).min(points.len());
    let mut remaining: Vec<Point> = points.to_vec();
    while remaining.len() > target {
        let (idx, _area) = match smallest_area(&remaining) {
            Some(found) => found,
            None => break,
        };
        remaining.remove(idx);
    }
    remaining
}

/// Simplify `points` until the smallest remaining triangle area exceeds
/// `tolerance` (in squared-pixel units). Always preserves at least the two
/// endpoints.
#[must_use]
pub fn visvalingam_whyatt_until(points: &[Point], tolerance: f32) -> Vec<Point> {
    let mut remaining: Vec<Point> = points.to_vec();
    loop {
        let (idx, area) = match smallest_area(&remaining) {
            Some(found) => found,
            None => break,
        };
        if area >= tolerance {
            break;
        }
        remaining.remove(idx);
        if remaining.len() <= 2 {
            break;
        }
    }
    remaining
}

/// Find the interior vertex with the smallest triangle area, returning
/// `(index, area)`. Returns `None` if the polyline has fewer than 3 points.
fn smallest_area(points: &[Point]) -> Option<(usize, f32)> {
    if points.len() < 3 {
        return None;
    }
    let mut min_idx = 1;
    let mut min_area = f32::INFINITY;
    for i in 1..points.len() - 1 {
        let area = triangle_area(points[i - 1], points[i], points[i + 1]);
        if area < min_area {
            min_area = area;
            min_idx = i;
        }
    }
    Some((min_idx, min_area))
}

/// Twice the unsigned triangle area for integer coordinates.
fn triangle_area(a: Point, b: Point, c: Point) -> f32 {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    #[expect(
        clippy::cast_precision_loss,
        reason = "integer coordinates always fit in f32 for image sizes ≤ 32768"
    )]
    let cross_f = cross.unsigned_abs() as f32;
    cross_f * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fewer_than_three_points_is_pass_through() {
        let pts = vec![Point::new(0, 0), Point::new(10, 0)];
        let out = visvalingam_whyatt(&pts, 1);
        assert_eq!(out, pts);
    }

    #[test]
    fn collinear_points_can_be_removed_completely() {
        // Five collinear points along the x-axis.
        let pts: Vec<Point> = (0..5).map(|i| Point::new(i * 2, 0)).collect();
        let out = visvalingam_whyatt(&pts, 2);
        // Should reduce to the two endpoints.
        assert_eq!(out, vec![Point::new(0, 0), Point::new(8, 0)]);
    }

    #[test]
    fn square_corners_survive() {
        // Square with extra midpoints on each side.
        let pts = vec![
            Point::new(0, 0),
            Point::new(5, 0),
            Point::new(10, 0),
            Point::new(10, 5),
            Point::new(10, 10),
            Point::new(5, 10),
            Point::new(0, 10),
            Point::new(0, 5),
            Point::new(0, 0),
        ];
        // Reduce to 5 points (4 corners + closing repeat).
        let out = visvalingam_whyatt(&pts, 5);
        assert_eq!(out.len(), 5);
        // The four corners must be present.
        for corner in &[
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ] {
            assert!(out.contains(corner), "missing corner {corner:?}");
        }
    }

    #[test]
    fn tolerance_form_respects_threshold() {
        // 21 points along y = 0, plus one large bump at the middle.
        let mut pts: Vec<Point> = (0..10).map(|i| Point::new(i, 0)).collect();
        pts.push(Point::new(10, 10)); // big bump
        pts.extend((11..=20).map(|i| Point::new(i, 0)));

        let out = visvalingam_whyatt_until(&pts, 1.0);
        // The bump must remain (its own triangle area is huge).
        assert!(out.contains(&Point::new(10, 10)));
        // The two endpoints must remain.
        assert!(out.contains(&Point::new(0, 0)));
        assert!(out.contains(&Point::new(20, 0)));
        // Strictly fewer interior points than input. Points immediately
        // adjacent to the bump survive because their triangle area is
        // `0.5 * 1 * 10 = 5.0`, which exceeds the tolerance. That's correct
        // VW behaviour, not a bug — but the rest of the collinear axis
        // points (whose triangle area is 0) MUST be gone.
        assert!(out.len() < pts.len(), "no simplification happened");
        let interior_far_from_bump = out
            .iter()
            .filter(|&&p| p.y == 0 && p.x != 0 && p.x != 20 && (p.x - 10).abs() > 1)
            .count();
        assert_eq!(
            interior_far_from_bump, 0,
            "collinear axis points away from the bump survived: {out:?}"
        );
    }
}
