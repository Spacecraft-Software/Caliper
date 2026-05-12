<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
-->

# Caliper

**A precision raster-to-vector tracing engine.**
*Project Steelbore — Rust-native, heterogeneous-compute, agent-first CLI.*

[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg)](rust-toolchain.toml)
[![Status](https://img.shields.io/badge/status-pre--v0.1.0-yellow.svg)](TODO.md)

Caliper converts pixel-based raster images (PNG, JPEG, BMP, WebP, TIFF, AVIF,
QOI, and more) into mathematically precise vector graphics (SVG, PDF, EPS, DXF)
through a heterogeneous pipeline that exploits every available compute resource:
multi-core CPUs with explicit SIMD (SSE2/SSE4.1/AVX2/AVX-512/NEON/SVE2), GPUs via
portable compute shaders (wgpu), and NPUs via ONNX Runtime execution providers.

Caliper does not invent new tracing algorithms — it composes the best published
ones (Suzuki-Abe contour following, O(n) spline fitting in the VTracer lineage,
RANSAC primitive recognition, OKLab k-means quantization) under a novel adaptive
meta-algorithm called the **Assayer**, which inspects each input and routes it to
the optimal pipeline configuration.

See [`PRD.md`](PRD.md) for the full product requirements, [`PLAN.md`](PLAN.md)
for the implementation roadmap, and [`TODO.md`](TODO.md) for the tickable backlog.

---

## Quick Start

> Pre-v0.1.0 — not yet shipping. The commands below describe the v0.1.0
> surface once it lands.

```sh
# Trace a single image to SVG
caliper trace logo.png -o logo.svg

# Inspect an image and emit the recommended pipeline as JSON
caliper inspect photo.jpg --json

# Batch-trace a directory with cross-file parallelism
caliper batch '**/*.png' -o vectors/

# Quick low-res preview with metrics
caliper preview big-poster.tiff

# Drive Caliper from another agent / pipeline
AI_AGENT=1 caliper trace input.png -o output.svg
```

Every data-returning command supports `--json` with a stable schema published
via `caliper schema <command>`. Every error carries a runnable hint. Every
destructive operation supports `--dry-run`.

---

## Highlights

- **Heterogeneous compute as a first-class concern.** Per-stage CPU / GPU /
  NPU dispatch via a typed pipeline DAG, not bolted-on flags.
- **Adaptive routing.** The Assayer measures each input (colour count, edge
  density, noise, classification) and selects the optimal pipeline.
- **Memory-safe by construction.** Zero `unsafe` outside `caliper-anvil`;
  every SIMD kernel Miri-validated against a scalar reference.
- **Agent-native CLI.** POSIX-compliant, structured JSON, runnable error hints,
  MCP server surface (`caliper mcp`), `AI_AGENT=1` aware.
- **Privacy-friendly.** Zero telemetry. Zero auto-network. ML models opt-in
  via explicit `caliper models pull`.
- **Steelbore-compliant.** GPL-3.0-or-later. ISO 8601 UTC. WCAG 2.1 AA TUI.
  Share Tech Mono / Inconsolata typography. Void Navy palette.

---

## Architecture

Seven-stage DAG with declared compute affinity per stage:

```
Input → Assayer → Decode → Quantize → EdgeDetect → Trace → Fit → Stitch → Cast → Output
        CPU       CPU/GPU   CPU/GPU    CPU/GPU/NPU CPU     CPU+NPU CPU     CPU
```

Full architecture in [`PRD.md` §5](PRD.md#5-architecture-overview).
Workspace topology (14 crates) in [`PLAN.md` §3](PLAN.md#3-workspace-topology).

---

## Project Posture

Caliper is a personal hobby project under the [Steelbore](https://Steelbore.com/)
umbrella. It is offered AS IS with no warranty and no liability — the formal
GPL-3.0-or-later terms in [LICENSE](LICENSE) govern any conflict.

| Aspect         | Stance                                                |
|----------------|-------------------------------------------------------|
| Audience       | Maintainer's own use case                             |
| Pace           | Hobby pace; no service-level commitments              |
| Warranty       | None — see [NOTICE.md](NOTICE.md)                     |
| Liability      | None — see [NOTICE.md](NOTICE.md)                     |
| Contributions  | Welcome; acceptance at maintainer's discretion (see [CONTRIBUTING.md](CONTRIBUTING.md)) |
| Forking        | Encouraged                                            |

The [Steelbore Standard §5](https://Steelbore.com/standard) governs this stance.

---

## Documentation

| Document                          | Purpose                                         |
|-----------------------------------|-------------------------------------------------|
| [`PRD.md`](PRD.md)                | Product Requirements Document (v2.0)            |
| [`PLAN.md`](PLAN.md)              | Implementation plan with phases and gates       |
| [`TODO.md`](TODO.md)              | Tickable backlog from bootstrap through v2.0.0  |
| [`NOTICE.md`](NOTICE.md)          | Warranty / liability disclaimer                 |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute and what gets merged       |
| [`CREDITS.md`](CREDITS.md)        | Third-party prior art and inspiration           |
| [`AGENTS.md`](AGENTS.md)          | Instructions for AI agents working in this repo |
| [`CLAUDE.md`](CLAUDE.md)          | Claude Code-specific guidance                   |
| [`SKILL.md`](SKILL.md)            | Discoverable skill manifest                     |

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)&gt;
Copyright (c) 2026 Mohamed Hammad &nbsp;|&nbsp; License: GPL-3.0-or-later
Project page: [https://Caliper.Steelbore.com/](https://Caliper.Steelbore.com/)

---

*Forged in Steelbore.*
