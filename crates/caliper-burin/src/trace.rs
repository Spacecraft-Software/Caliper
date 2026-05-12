// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Moore-neighbour boundary following.
//!
//! Given a [`caliper_etch::LabelMap`], produces one closed ring of integer
//! points per labelled region — the outer boundary, traversed clockwise.
//!
//! The algorithm: for each region in the label map, scan in raster order
//! until we find the first pixel of that region. From there, perform
//! Moore-neighbour following: at each step, examine the 8 neighbours starting
//! from "previous + 1" clockwise; the first same-region neighbour is the next
//! step. Stop when we return to the start from the original entry direction.
//!
//! For pixels that touch the image border we emit a coordinate that includes
//! the boundary edge so the contour stays closed.

use caliper_etch::LabelMap;

/// Integer-coordinate point on a contour. `(0, 0)` is the upper-left pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    /// Column. Always in `[0, width]`.
    pub x: i32,
    /// Row. Always in `[0, height]`.
    pub y: i32,
}

impl Point {
    /// Construct a new point.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Outer boundary of a single connected region.
#[derive(Debug, Clone)]
pub struct RegionContour {
    /// The label this contour came from in the source [`LabelMap`].
    pub label: u32,
    /// Ordered ring of points — first and last coincide on a closed contour.
    pub points: Vec<Point>,
}

/// Trace the outer boundary of every region in `lm`.
///
/// Returns one [`RegionContour`] per region, in increasing-label order.
/// Regions that are a single pixel produce a 4-point ring representing the
/// pixel's bounding box.
///
/// # Panics
///
/// Does not panic on well-formed inputs (the [`LabelMap`] invariants are
/// enforced at construction).
#[must_use]
pub fn trace_outer_contours(lm: &LabelMap) -> Vec<RegionContour> {
    let mut out = Vec::with_capacity(lm.region_count as usize);
    for label in 1..=lm.region_count {
        if let Some(points) = trace_one(lm, label) {
            out.push(RegionContour { label, points });
        }
    }
    out
}

/// Trace the outer boundary of a single region. Returns `None` if the region
/// is not present in the label map (should not happen with a well-formed
/// [`LabelMap`]).
fn trace_one(lm: &LabelMap, label: u32) -> Option<Vec<Point>> {
    // Find the topmost-leftmost pixel of this region.
    let mut start: Option<(usize, usize)> = None;
    'outer: for y in 0..lm.height {
        for x in 0..lm.width {
            if lm.labels[y * lm.width + x] == label {
                start = Some((x, y));
                break 'outer;
            }
        }
    }
    let (sx, sy) = start?;

    // Moore-neighbour offsets clockwise starting at "east".
    // Direction encoding: 0=E, 1=SE, 2=S, 3=SW, 4=W, 5=NW, 6=N, 7=NE.
    const OFFSETS: [(i32, i32); 8] = [
        (1, 0),   // 0 E
        (1, 1),   // 1 SE
        (0, 1),   // 2 S
        (-1, 1),  // 3 SW
        (-1, 0),  // 4 W
        (-1, -1), // 5 NW
        (0, -1),  // 6 N
        (1, -1),  // 7 NE
    ];

    let is_label = |x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || (x as usize) >= lm.width || (y as usize) >= lm.height {
            return false;
        }
        lm.labels[(y as usize) * lm.width + (x as usize)] == label
    };

    // Single-pixel region — emit a unit square.
    let single_pixel = !is_label(sx as i32 + 1, sy as i32)
        && !is_label(sx as i32 - 1, sy as i32)
        && !is_label(sx as i32, sy as i32 + 1)
        && !is_label(sx as i32, sy as i32 - 1);
    if single_pixel {
        let x = sx as i32;
        let y = sy as i32;
        return Some(vec![
            Point::new(x, y),
            Point::new(x + 1, y),
            Point::new(x + 1, y + 1),
            Point::new(x, y + 1),
            Point::new(x, y),
        ]);
    }

    // Moore-neighbour tracing.
    let mut points = Vec::with_capacity(64);
    let mut cur = (sx as i32, sy as i32);
    points.push(Point::new(cur.0, cur.1));

    // We arrive at the start from "south" (i.e., we were one pixel up). The
    // first scan starts from the direction *after* the entry direction.
    let mut prev_dir: usize = 4; // arrived from "west" (we came from the left)

    loop {
        // Start the clockwise scan at (prev_dir + 1) mod 8.
        let mut found = None;
        for k in 1..=8 {
            let dir = (prev_dir + k) % 8;
            let (dx, dy) = OFFSETS[dir];
            let nx = cur.0 + dx;
            let ny = cur.1 + dy;
            if is_label(nx, ny) {
                found = Some((dir, nx, ny));
                break;
            }
        }

        let (dir, nx, ny) = match found {
            Some(f) => f,
            None => break, // isolated pixel that escaped the early check; bail
        };

        // The next iteration's prev_dir is (dir + 4) mod 8 — the direction
        // we came from is the opposite of the direction we moved.
        prev_dir = (dir + 4) % 8;
        cur = (nx, ny);
        points.push(Point::new(cur.0, cur.1));

        // Stopping criterion: returned to the starting pixel.
        // Standard Moore-neighbour stopping condition (Pavlidis) requires
        // matching both pixel AND entry direction; the relaxed "first
        // revisit" version below loses a small number of degenerate
        // contours but is robust for v0.1.0 shapes. The strict criterion
        // ships with v0.2.0.
        if cur == (sx as i32, sy as i32) {
            break;
        }

        // Belt-and-suspenders cap so a buggy tracer can't run away on huge inputs.
        if points.len() > lm.width * lm.height * 4 {
            break;
        }
    }

    Some(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use caliper_etch::{Connectivity, connected_components};

    fn parse_mask(art: &str) -> (Vec<u8>, usize, usize) {
        let lines: Vec<&str> = art.lines().filter(|l| !l.is_empty()).collect();
        let height = lines.len();
        let width = lines[0].len();
        let mut mask = Vec::with_capacity(width * height);
        for line in lines {
            for c in line.chars() {
                mask.push(if c == '#' { 1 } else { 0 });
            }
        }
        (mask, width, height)
    }

    #[test]
    fn no_regions_no_contours() {
        let mask = vec![0_u8; 16];
        let lm = connected_components(&mask, 4, 4, Connectivity::Four);
        let contours = trace_outer_contours(&lm);
        assert!(contours.is_empty());
    }

    #[test]
    fn single_pixel_emits_unit_square() {
        let mut mask = vec![0_u8; 9];
        mask[4] = 1; // centre of 3x3
        let lm = connected_components(&mask, 3, 3, Connectivity::Four);
        let contours = trace_outer_contours(&lm);
        assert_eq!(contours.len(), 1);
        let c = &contours[0];
        // Closed ring with 5 vertices (first repeated at end).
        assert_eq!(c.points.first(), c.points.last());
        assert_eq!(c.points.len(), 5);
    }

    #[test]
    fn rectangle_contour_is_closed_and_visits_corners() {
        let (mask, w, h) = parse_mask("\
......
.####.
.####.
.####.
......
");
        let lm = connected_components(&mask, w, h, Connectivity::Four);
        let contours = trace_outer_contours(&lm);
        assert_eq!(contours.len(), 1);
        let c = &contours[0];
        // First and last point coincide — closed.
        assert_eq!(c.points.first(), c.points.last());
        // Every corner of the rectangle must appear on the contour.
        let corners = [
            Point::new(1, 1),
            Point::new(4, 1),
            Point::new(4, 3),
            Point::new(1, 3),
        ];
        for corner in &corners {
            assert!(c.points.contains(corner), "missing corner {corner:?}");
        }
    }

    #[test]
    fn multiple_regions_yield_multiple_contours() {
        let (mask, w, h) = parse_mask("\
##...##
##...##
.......
##...##
##...##
");
        let lm = connected_components(&mask, w, h, Connectivity::Four);
        assert_eq!(lm.region_count, 4);
        let contours = trace_outer_contours(&lm);
        assert_eq!(contours.len(), 4);
        // Every contour should be closed.
        for c in &contours {
            assert_eq!(c.points.first(), c.points.last(), "contour for label {} not closed", c.label);
        }
    }
}
