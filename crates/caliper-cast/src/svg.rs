// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! SVG 1.1 streaming writer.
//!
//! Emits valid SVG 1.1 markup straight to a [`std::io::Write`] sink — no
//! intermediate string allocation, no XML DOM. UTF-8 without BOM. LF line
//! endings. Deterministic output: identical document + precision → identical
//! bytes.
//!
//! Coordinate precision is controlled by `path_precision` (default 3 decimal
//! places). Smaller precision → smaller output; quality loss is sub-pixel
//! until precision dips below 2.

use std::io::{self, Write};

use crate::document::{Path, VectorDocument};

/// Writer configuration. Sensible defaults via [`SvgConfig::default`].
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// Decimal places in path coordinates (PRD §9.3 — `--path-precision`).
    pub path_precision: usize,
    /// Optional fixed background colour for the root SVG element.
    pub background: Option<crate::document::RgbColor>,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            path_precision: 3,
            background: None,
        }
    }
}

/// Write `doc` as SVG 1.1 to `out`.
///
/// # Errors
///
/// Propagates any [`io::Error`] from the underlying writer.
pub fn write_svg<W: Write>(
    out: &mut W,
    doc: &VectorDocument,
    config: &SvgConfig,
) -> io::Result<()> {
    // Header — no XML declaration so a downstream pipe can append fragments.
    // We declare the SVG namespace inline.
    write!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = doc.width,
        h = doc.height,
    )?;
    out.write_all(b"\n")?;

    if let Some(bg) = config.background {
        write!(
            out,
            r##"<rect width="100%" height="100%" fill="#{:02x}{:02x}{:02x}"/>"##,
            bg.r, bg.g, bg.b,
        )?;
        out.write_all(b"\n")?;
    }

    for layer in &doc.layers {
        for path in &layer.paths {
            if path.points.is_empty() {
                continue;
            }
            write!(out, r##"<path fill="#"##)?;
            write!(out, "{:02x}{:02x}{:02x}", layer.color.r, layer.color.g, layer.color.b)?;
            out.write_all(br#"" d=""#)?;
            write_path_data(out, path, config.path_precision)?;
            out.write_all(br#""/>"#)?;
            out.write_all(b"\n")?;
        }
    }

    out.write_all(b"</svg>\n")?;
    Ok(())
}

/// Emit a single `<path>` element's `d` attribute payload — `M x y L x y …
/// L x y Z`. Closes the path if the first and last points coincide.
fn write_path_data<W: Write>(out: &mut W, path: &Path, precision: usize) -> io::Result<()> {
    debug_assert!(!path.points.is_empty());
    let (x0, y0) = path.points[0];

    write!(out, "M")?;
    write_coord(out, x0, precision)?;
    out.write_all(b",")?;
    write_coord(out, y0, precision)?;

    // Determine whether the ring is closed.
    let last = *path.points.last().expect("non-empty checked above");
    let closed = path.points.len() >= 2
        && (last.0 - x0).abs() <= f32::EPSILON
        && (last.1 - y0).abs() <= f32::EPSILON;

    // Iterate the interior points, skipping the trailing duplicate when closed.
    let interior_end = if closed {
        path.points.len() - 1
    } else {
        path.points.len()
    };

    if interior_end > 1 {
        out.write_all(b"L")?;
        for (i, (x, y)) in path.points[1..interior_end].iter().enumerate() {
            if i > 0 {
                out.write_all(b" ")?;
            }
            write_coord(out, *x, precision)?;
            out.write_all(b",")?;
            write_coord(out, *y, precision)?;
        }
    }

    if closed {
        out.write_all(b"Z")?;
    }
    Ok(())
}

/// Emit a single coordinate at `precision` decimal places, trimming trailing
/// zeros so `1.5` does not render as `1.500`.
fn write_coord<W: Write>(out: &mut W, v: f32, precision: usize) -> io::Result<()> {
    // We format into a small stack buffer first because trimming trailing
    // zeros from `{:.precision$}` is easier on a string than on a writer.
    let mut buf = format!("{v:.precision$}");
    if buf.contains('.') {
        while buf.ends_with('0') {
            buf.pop();
        }
        if buf.ends_with('.') {
            buf.pop();
        }
    }
    if buf.is_empty() || buf == "-" {
        buf.push('0');
    }
    out.write_all(buf.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Layer, RgbColor};

    fn small_doc() -> VectorDocument {
        VectorDocument {
            width: 100,
            height: 100,
            layers: vec![Layer {
                color: RgbColor::new(0xD9, 0x8E, 0x32), // Molten Amber
                paths: vec![Path {
                    points: vec![
                        (10.0, 10.0),
                        (90.0, 10.0),
                        (90.0, 90.0),
                        (10.0, 90.0),
                        (10.0, 10.0),
                    ],
                }],
            }],
        }
    }

    #[test]
    fn round_trip_produces_valid_svg_skeleton() {
        let doc = small_doc();
        let mut buf = Vec::new();
        write_svg(&mut buf, &doc, &SvgConfig::default()).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("<svg "));
        assert!(s.contains(r#"width="100""#));
        assert!(s.contains(r#"height="100""#));
        assert!(s.contains(r##"fill="#d98e32""##));
        assert!(s.contains("M10,10L90,10 90,90 10,90Z"));
        assert!(s.ends_with("</svg>\n"));
    }

    #[test]
    fn deterministic_output() {
        let doc = small_doc();
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_svg(&mut a, &doc, &SvgConfig::default()).unwrap();
        write_svg(&mut b, &doc, &SvgConfig::default()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn empty_document_emits_valid_svg() {
        let doc = VectorDocument::empty(50, 50);
        let mut buf = Vec::new();
        write_svg(&mut buf, &doc, &SvgConfig::default()).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("<svg "));
        assert!(s.ends_with("</svg>\n"));
        assert!(!s.contains("<path"));
    }

    #[test]
    fn precision_trims_trailing_zeros() {
        let doc = VectorDocument {
            width: 100,
            height: 100,
            layers: vec![Layer {
                color: RgbColor::new(0x12, 0x34, 0x56), // distinctive non-zero fill
                paths: vec![Path {
                    points: vec![(1.25, 2.5), (3.75, 4.0), (1.25, 2.5)],
                }],
            }],
        };
        let mut buf = Vec::new();
        write_svg(
            &mut buf,
            &doc,
            &SvgConfig {
                path_precision: 4,
                ..Default::default()
            },
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        // Coordinates must not retain trailing zeros — `1.2500`, `2.5000`,
        // `4.0000`, `3.7500` should all collapse to their minimal form.
        assert!(!s.contains("1.2500"), "1.2500 not trimmed in {s}");
        assert!(!s.contains("2.5000"), "2.5000 not trimmed in {s}");
        assert!(!s.contains("3.7500"), "3.7500 not trimmed in {s}");
        assert!(!s.contains("4.0000"), "4.0000 not trimmed in {s}");
        assert!(s.contains("M1.25,2.5"), "trimmed move-to missing in {s}");
        // Closed-path Z because the ring repeats the start.
        assert!(s.contains("Z"));
    }

    #[test]
    fn open_polyline_lacks_z() {
        let doc = VectorDocument {
            width: 100,
            height: 100,
            layers: vec![Layer {
                color: RgbColor::new(0, 0, 0),
                paths: vec![Path {
                    points: vec![(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)],
                }],
            }],
        };
        let mut buf = Vec::new();
        write_svg(&mut buf, &doc, &SvgConfig::default()).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("M0,0L10,10 20,0"));
        assert!(!s.contains("Z"));
    }

    #[test]
    fn no_bom_in_output() {
        let doc = small_doc();
        let mut buf = Vec::new();
        write_svg(&mut buf, &doc, &SvgConfig::default()).unwrap();
        // UTF-8 BOM is EF BB BF; SVG output must NOT include it (PRD §15).
        assert!(buf.len() >= 3 && &buf[0..3] != [0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn background_renders_when_set() {
        let doc = small_doc();
        let mut buf = Vec::new();
        write_svg(
            &mut buf,
            &doc,
            &SvgConfig {
                background: Some(RgbColor::new(0, 0, 0x27)), // Void Navy
                ..Default::default()
            },
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(r##"<rect width="100%" height="100%" fill="#000027""##));
    }
}
