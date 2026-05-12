// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Connected-component labelling for binary masks.
//!
//! Two-pass union-find — Hoshen-Kopelman style. First pass scans the mask
//! row-major, assigning provisional labels and recording equivalences via a
//! union-find structure. Second pass replaces every provisional label with the
//! root of its equivalence class and renumbers densely from 1.
//!
//! Per [PRD §8.4], the connected-component output is what `caliper-burin`
//! traces; one contour per labelled region. CPU-only — GPU isn't a clear win
//! for this workload at typical Caliper image sizes (PRD §6.2).
//!
//! [PRD §8.4]: https://Caliper.Steelbore.com/prd#84-stage-3--edge-detection

/// 4-vs-8 connectivity for the connected-component scan.
///
/// 4-connectivity considers only the N, S, E, W neighbours of each pixel.
/// 8-connectivity additionally considers the four diagonals — yielding fewer,
/// larger regions when boundaries touch only at corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connectivity {
    /// Four neighbours: N, S, E, W.
    Four,
    /// Eight neighbours: N, S, E, W, NE, NW, SE, SW.
    Eight,
}

/// Dense region-ID map. Background pixels carry label `0`. Foreground pixels
/// carry labels in `1..=region_count`.
#[derive(Debug, Clone)]
pub struct LabelMap {
    /// Width of the map in pixels.
    pub width: usize,
    /// Height of the map in pixels.
    pub height: usize,
    /// Per-pixel label, row-major. Length is `width * height`.
    pub labels: Vec<u32>,
    /// Number of distinct foreground regions found. Labels run `1..=region_count`.
    pub region_count: u32,
}

impl LabelMap {
    /// Label at `(x, y)`. Returns `0` for background.
    #[inline]
    #[must_use]
    pub fn at(&self, x: usize, y: usize) -> u32 {
        self.labels[y * self.width + x]
    }
}

/// Run two-pass connected-component labelling on `mask`.
///
/// `mask` is a row-major binary buffer: any non-zero value is foreground,
/// zero is background. Output labels are dense (`1..=region_count`), with
/// background mapped to `0`.
///
/// # Panics
///
/// Panics if `mask.len() != width * height`.
#[must_use]
pub fn connected_components(
    mask: &[u8],
    width: usize,
    height: usize,
    connectivity: Connectivity,
) -> LabelMap {
    assert_eq!(
        mask.len(),
        width * height,
        "mask length must equal width * height"
    );

    let pixels = width * height;
    let mut labels = vec![0_u32; pixels];

    // Union-find over provisional labels. `parent[0]` is reserved for
    // background so it is never unioned.
    let mut parent: Vec<u32> = vec![0];
    let mut next_label: u32 = 1;

    // First pass — assign provisional labels.
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if mask[idx] == 0 {
                continue;
            }

            // Gather labels from already-scanned neighbours.
            let mut neighbours: [u32; 4] = [0; 4];
            let mut count = 0;

            // Always check N and W in 4-connectivity.
            if y > 0 {
                let l = labels[(y - 1) * width + x];
                if l != 0 {
                    neighbours[count] = l;
                    count += 1;
                }
            }
            if x > 0 {
                let l = labels[y * width + (x - 1)];
                if l != 0 {
                    neighbours[count] = l;
                    count += 1;
                }
            }

            // Diagonals only for 8-connectivity.
            if connectivity == Connectivity::Eight {
                if y > 0 && x > 0 {
                    let l = labels[(y - 1) * width + (x - 1)];
                    if l != 0 {
                        neighbours[count] = l;
                        count += 1;
                    }
                }
                if y > 0 && x + 1 < width {
                    let l = labels[(y - 1) * width + (x + 1)];
                    if l != 0 {
                        neighbours[count] = l;
                        count += 1;
                    }
                }
            }

            if count == 0 {
                // No labelled neighbour — start a new region.
                labels[idx] = next_label;
                parent.push(next_label);
                next_label += 1;
            } else {
                // Use the smallest neighbour's root; union the rest into it.
                let mut root = uf_find(&mut parent, neighbours[0]);
                for &n in &neighbours[1..count] {
                    let other = uf_find(&mut parent, n);
                    if other != root {
                        let (lo, hi) = if other < root {
                            (other, root)
                        } else {
                            (root, other)
                        };
                        parent[hi as usize] = lo;
                        root = lo;
                    }
                }
                labels[idx] = root;
            }
        }
    }

    // Second pass — resolve each pixel to its root, then renumber densely.
    let mut compact: Vec<u32> = vec![0; parent.len()];
    let mut region_count = 0_u32;
    for label in &mut labels {
        if *label == 0 {
            continue;
        }
        let root = uf_find(&mut parent, *label);
        let dense = if compact[root as usize] == 0 {
            region_count += 1;
            compact[root as usize] = region_count;
            region_count
        } else {
            compact[root as usize]
        };
        *label = dense;
    }

    LabelMap {
        width,
        height,
        labels,
        region_count,
    }
}

/// Union-find with path compression.
fn uf_find(parent: &mut [u32], mut x: u32) -> u32 {
    while parent[x as usize] != x {
        let grand = parent[parent[x as usize] as usize];
        parent[x as usize] = grand;
        x = grand;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny helper to read a fixed-size ASCII mask. `.` = background, `#` = foreground.
    fn parse_mask(art: &str) -> (Vec<u8>, usize, usize) {
        let lines: Vec<&str> = art.lines().filter(|l| !l.is_empty()).collect();
        let height = lines.len();
        let width = lines[0].len();
        let mut mask = Vec::with_capacity(width * height);
        for line in lines {
            assert_eq!(line.len(), width, "uneven mask widths");
            for c in line.chars() {
                mask.push(if c == '#' { 1 } else { 0 });
            }
        }
        (mask, width, height)
    }

    #[test]
    fn empty_mask_has_no_regions() {
        let mask = vec![0_u8; 16];
        let lm = connected_components(&mask, 4, 4, Connectivity::Four);
        assert_eq!(lm.region_count, 0);
        assert!(lm.labels.iter().all(|&l| l == 0));
    }

    #[test]
    fn single_pixel_is_one_region() {
        let mut mask = vec![0_u8; 9];
        mask[4] = 1; // centre of 3x3
        let lm = connected_components(&mask, 3, 3, Connectivity::Four);
        assert_eq!(lm.region_count, 1);
        assert_eq!(lm.at(1, 1), 1);
    }

    #[test]
    fn two_separated_blobs() {
        let (mask, w, h) = parse_mask("\
##....##
##....##
##....##
........
........
........
........
........
");
        let lm = connected_components(&mask, w, h, Connectivity::Four);
        assert_eq!(lm.region_count, 2);
        // Top-left blob is region 1; top-right is region 2.
        assert_eq!(lm.at(0, 0), 1);
        assert_eq!(lm.at(7, 0), 2);
    }

    #[test]
    fn diagonal_blobs_merge_under_eight_connectivity() {
        let (mask, w, h) = parse_mask("\
#...
.#..
..#.
...#
");
        let four = connected_components(&mask, w, h, Connectivity::Four);
        let eight = connected_components(&mask, w, h, Connectivity::Eight);
        assert_eq!(four.region_count, 4, "4-conn: each pixel is its own region");
        assert_eq!(eight.region_count, 1, "8-conn: the diagonal is one region");
    }

    #[test]
    fn u_shape_merges_through_equivalence() {
        // Two columns of foreground joined at the bottom — first-pass labels
        // diverge at the top and converge at the bottom; the union-find must
        // collapse them into a single region.
        let (mask, w, h) = parse_mask("\
#..#
#..#
#..#
####
");
        let lm = connected_components(&mask, w, h, Connectivity::Four);
        assert_eq!(lm.region_count, 1);
        // Confirm both top corners share the same dense label.
        assert_eq!(lm.at(0, 0), lm.at(3, 0));
    }

    #[test]
    fn checker_pattern_under_four_connectivity() {
        let (mask, w, h) = parse_mask("\
#.#.#.
.#.#.#
#.#.#.
.#.#.#
");
        let lm = connected_components(&mask, w, h, Connectivity::Four);
        // Every foreground pixel is its own region (no shared edge between them).
        let expected = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| mask[y * w + x] == 1)
            .count();
        assert_eq!(lm.region_count as usize, expected);
    }

    #[test]
    #[should_panic = "mask length must equal width * height"]
    fn wrong_size_mask_panics() {
        let mask = vec![0_u8; 7];
        let _ = connected_components(&mask, 3, 3, Connectivity::Four);
    }
}
