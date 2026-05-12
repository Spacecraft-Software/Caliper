<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
-->

# Credits

Caliper composes well-grounded prior art rather than inventing new algorithms.
This file is the human-readable counterpart to the SPDX headers and the license
metadata that Cargo surfaces mechanically: it credits the authors and prior work
whose ideas, implementations, or specifications materially shaped Caliper.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for license terms of contributions to
Caliper itself.

---

## Algorithms

| Work                                                                              | Author(s)                  | License      | Source                                                | Scope                                                                                          |
|-----------------------------------------------------------------------------------|----------------------------|--------------|-------------------------------------------------------|------------------------------------------------------------------------------------------------|
| *Potrace: a polygon-based tracing algorithm* (2003)                               | Peter Selinger             | Paper / GPL-2.0+ | https://potrace.sourceforge.net/                  | Reference quality benchmark for bi-level tracing; Selinger segment-merging used in `caliper-temper`. |
| *Topological Structural Analysis of Digitized Binary Images by Border Following* (1985) | Satoshi Suzuki & Keiichi Abe | Paper        | CVGIP 30(1)                                           | Boundary-following algorithm implemented in `caliper-burin`.                                   |
| *Line generalisation by repeated elimination of points* (1993)                    | M. Visvalingam & J. D. Whyatt | Paper        | Cartographic Journal 30(1)                            | Path simplification in `caliper-burin`.                                                        |
| VTracer                                                                           | visioncortex               | MIT          | https://github.com/visioncortex/vtracer               | O(n) Bézier-fitting lineage; hierarchical clustering ideas in `caliper-temper`.                |
| *The OKLab colour space* (2020)                                                   | Björn Ottosson             | Article / public-domain math | https://bottosson.github.io/posts/oklab          | Perceptually-uniform colour space used by `caliper-smelt` for k-means quantisation.            |
| *Holistically-Nested Edge Detection* (HED, 2015)                                  | Saining Xie & Zhuowen Tu   | Paper        | https://arxiv.org/abs/1504.06375                      | ML edge-detection model integrated via `caliper-touchstone`.                                   |
| *Bi-Directional Cascade Network for Perceptual Edge Detection* (BDCN, 2019)       | He et al.                  | Paper        | https://arxiv.org/abs/1902.10903                      | Alternative ML edge-detection model in `caliper-touchstone`.                                   |
| RANSAC primitive fitting                                                          | Fischler & Bolles (1981)   | Paper / public-domain idiom | CACM 24(6)                              | Circle / ellipse / rectangle / star fitting in `caliper-temper`.                               |

## Tooling & Specifications

| Work                                                       | Author(s)                  | License        | Source                                          | Scope                                                                                |
|------------------------------------------------------------|----------------------------|----------------|-------------------------------------------------|--------------------------------------------------------------------------------------|
| Microsoft Pragmatic Rust Guidelines                        | Microsoft Corporation      | MIT            | https://github.com/microsoft/rust-guidelines    | Style, naming, safety, and API-design rules followed across the workspace.           |
| Steelbore Rust Guidelines                                  | Mohamed Hammad             | GPL-3.0-or-later | https://github.com/UnbreakableMJ/rust-guidelines | Steelbore adaptation of the Microsoft guidelines.                                  |
| Model Context Protocol (MCP)                               | Anthropic & contributors   | MIT            | https://modelcontextprotocol.io                 | Protocol implemented by `caliper mcp`.                                               |
| ONNX Runtime                                               | Microsoft & contributors   | MIT            | https://onnxruntime.ai                          | Underlies `caliper-touchstone` via the `ort` Rust bindings.                          |
| wgpu                                                       | gfx-rs project             | MIT / Apache-2.0 | https://wgpu.rs                              | Portable compute-shader backend powering `caliper-bellows`.                          |
| Ratatui                                                    | Ratatui contributors       | MIT            | https://ratatui.rs                              | TUI framework powering `caliper-tui`.                                                |
| Rayon                                                      | Niko Matsakis & contributors | MIT / Apache-2.0 | https://docs.rs/rayon                        | Work-stealing parallelism throughout the pipeline.                                   |

## Standards

Caliper conforms to (but does not redistribute) the following standards. They
are listed here for transparency; license metadata is not separately tracked.

- POSIX.1-2024 (IEEE Std 1003.1-2024) — CLI behaviour
- ISO 8601:2019 — date / time / duration format
- WCAG 2.1 Level AA — colour contrast for the TUI
- SVG 1.1 (W3C Recommendation) — primary output format
- PDF 1.7 (ISO 32000-1) — PDF output
- DXF R14 (Autodesk) — DXF output
- RFC 7946 (GeoJSON) — GeoJSON output (v1.2.0+)
- JSON Schema 2020-12 — `caliper schema` output

## Acknowledgements

The Caliper architecture would not exist without the open-source ecosystem that
made every dependency in [`PRD.md` §16](PRD.md#16-dependency-tree) possible.
Particular thanks to the maintainers of `image`, `rayon`, `crossbeam`, `wgpu`,
`ort`, `clap`, `ratatui`, `crossterm`, `pdf-writer`, `quick-xml`, `serde`,
`jiff`, `tracing`, `rmcp`, and `blake3` — every one is load-bearing.

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)&gt;
[https://Caliper.Steelbore.com/](https://Caliper.Steelbore.com/)
