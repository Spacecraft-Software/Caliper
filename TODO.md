<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
Steelbore Theme: Void Navy (#000027) bg, Molten Amber (#D98E32) body,
Steel Blue (#4B7EB0) H1, Radium Green (#50FA7B) H2, Liquid Coolant (#8BE9FD) H3
Fonts: Share Tech Mono (headings), Inconsolata (body)
-->

# CALIPER — Backlog

**Canonical, tickable task list. Maps 1:1 to `PLAN.md` phases and `PRD.md` §17 roadmap.**

| Field            | Value                                          |
|------------------|------------------------------------------------|
| Project          | Caliper                                        |
| Document         | TODO.md                                        |
| Document Version | 1.0                                            |
| Document Date    | 2026-05-12T00:00:00Z                           |
| Author           | Mohamed Hammad                                 |
| License          | GPL-3.0-or-later                               |
| Governing PRD    | `PRD.md` v2.0                                  |
| Companion        | `PLAN.md` v1.0                                 |

Use GFM checkbox syntax. Tick items as work lands. Every task names its source PRD section.

---

## 0. Workspace Bootstrap (pre-v0.1.0)

Pre-conditions for every milestone below. Land before opening any v0.1.0 PR.

- [ ] Initialize `git init` at `/steelbore/caliper/`; first commit on `main`.
- [ ] Create root `Cargo.toml` workspace — `resolver = "2"`, `edition = "2024"`, `rust-version = "1.82.0"` (PRD §2).
- [ ] Add `rust-toolchain.toml` pinning to 1.82.0 with `components = ["rustfmt", "clippy", "miri"]`.
- [ ] Add `rustfmt.toml` matching Steelbore Rust Guidelines.
- [ ] Add `clippy.toml` + workspace `[lints]` table; `forbid(unsafe_code)` everywhere except `caliper-anvil`.
- [ ] Add `deny.toml` — GPL-3.0-or-later compatible license allow-list; advisory database; banned-crate list (PRD §16).
- [ ] Add `.gitignore` (target/, *.bk, etc.) and `.editorconfig`.
- [ ] Add `LICENSE` containing GPL-3.0-or-later full text.
- [ ] Scaffold all 14 crate directories under `crates/<name>/` with stub `Cargo.toml`, `lib.rs` (or `main.rs`), and SPDX header on every file (PRD §2.1, §4).
- [ ] Create `xtask/` for cross-cutting build tasks: SPDX audit, UTF-8 audit, cross-shell tests, model-hash verify.
- [ ] Add `flake.nix` + `flake.lock` for reproducible dev shell (PRD §14.2).
- [ ] Author `README.md` — project summary, install, quick-start, link to PRD/PLAN/TODO.
- [ ] Author `CONTRIBUTING.md` — commit conventions, PR process, CI gates.
- [ ] Author `AGENTS.md` — instructions for AI agents using the codebase (PRD §17.1).
- [ ] Author `CLAUDE.md` — Claude Code-specific guidance (PRD §17.1).
- [ ] Author `SKILL.md` — discoverable skill manifest for agentic clients (PRD §17.1).
- [ ] Stand up `.github/workflows/ci.yml` — clippy, test, fmt, deny, vet jobs.
- [ ] Stand up `.github/workflows/fuzz.yml` — nightly fuzz job.
- [ ] Stand up `.github/workflows/release.yml` — tagged-release pipeline (stub for now).

---

## 1. v0.1.0 — Foundation (CPU-only) — `PRD.md` §17.1

### 1.1 `caliper-anvil` — SIMD primitives (PRD §6.1.2)

- [ ] SSE2 baseline kernels — color distance, edge detection, pixel blending.
- [ ] SSE4.1 kernels where they beat SSE2.
- [ ] AVX2 + FMA3 kernels — color quantization, histogram, 8-wide pixel ops.
- [ ] NEON kernels — color distance, edge kernels, quantization (aarch64).
- [ ] Runtime CPU-feature dispatcher; build-script `cfg` flags for compile-time gating.
- [ ] Scalar reference impl for every kernel; equivalence test per ISA.
- [ ] Miri-clean unsafe; `cargo miri test -p caliper-anvil` green in CI.

### 1.2 `caliper-crucible` — Pipeline orchestrator (PRD §5, §8)

- [ ] Seven-stage DAG types (`Stage0..Stage7`).
- [ ] `PipelineConfig` struct mirroring the JSON in PRD §7.2.
- [ ] Synchronous scheduler executing stages in declared order.
- [ ] Per-stage timing instrumentation surfaced in `metrics.stages_ms`.
- [ ] Stage backend selection plumbing (`BackendChoice::Cpu | Gpu | Npu`).

### 1.3 `caliper-assayer` — Adaptive router (PRD §7.2)

- [ ] Thumbnail (1/8 res) feature extraction.
- [ ] Histogram analysis + silhouette score for color count.
- [ ] Gradient magnitude stats for edge density.
- [ ] Local-variance + frequency-domain noise estimation.
- [ ] Rule-based image classifier (logo/line-art/technical-drawing/pixel-art/illustration/photo/map).
- [ ] Image-class routing table from PRD §7.3 implemented as data.
- [ ] Emit `PipelineConfig` consumed by Crucible.
- [ ] Override pathway: `--config <file>` deserializes user-authored config.
- [ ] Runtime budget ≤ 2 % of total trace time — assert in tests.

### 1.4 `caliper-smelt` — Color quantization (PRD §7.1, §8.3)

- [ ] OKLab color-space conversion (uses anvil SIMD).
- [ ] k-means with k-means++ init.
- [ ] Deterministic seeded RNG so identical inputs → identical palettes.
- [ ] `--colors auto` via silhouette analysis.
- [ ] `--bw` mode → Sauvola adaptive threshold + Otsu fallback.
- [ ] Median-cut quantizer (alternative).
- [ ] Octree quantizer (alternative).

### 1.5 `caliper-etch` — Edge detection (PRD §7.1, §8.4)

- [ ] SIMD Sobel 3×3 separable.
- [ ] SIMD Scharr 3×3 separable.
- [ ] Morphological ops (erode/dilate/open/close) — anvil-vectorized.
- [ ] Connected-component labeling via union-find.

### 1.6 `caliper-burin` — Contour tracing (PRD §7.1, §8.5)

- [ ] Suzuki-Abe boundary following with hole detection.
- [ ] Parent-child contour hierarchy.
- [ ] Visvalingam-Whyatt simplification — default tolerance 1.0 px.
- [ ] Douglas-Peucker as alternative.
- [ ] Corner detection via angular threshold (default 60°) + curvature analysis.
- [ ] Path-level Rayon parallelism — disconnected contours independent.

### 1.7 `caliper-temper` — Path fitting (PRD §7.1, §8.6)

- [ ] O(n) least-squares Bézier fitting (VTracer-lineage algorithm).
- [ ] Curve optimization — Selinger segment merging.
- [ ] Path modes: `pixel`, `polygon`, `spline`.
- [ ] `--simplify-tolerance` and `--corner-threshold` plumbing.

### 1.8 `caliper-cast` — SVG writer (PRD §8.8)

- [ ] Streaming SVG 1.1 writer via `quick-xml`.
- [ ] Configurable `--path-precision` (default 3).
- [ ] Optional CSS classes for paths.
- [ ] UTF-8 without BOM; LF line endings.
- [ ] Output deterministic in z-order.

### 1.9 `caliper-weld` — Stitching (PRD §8.7)

- [ ] Stage scaffolded; CPU-only single-tile pass for v0.1.0.
- [ ] Tile boundary stitching deferred to v0.2.0 (lands with batch).

### 1.10 `caliper-trace` — Public API (PRD §11.1)

- [ ] `TracingConfig` builder.
- [ ] `trace(&DynamicImage, &TracingConfig) -> Result<VectorDocument>` free function.
- [ ] `VectorDocument::{to_svg, paths, palette, metrics}` for v0.1.0.
- [ ] `CaliperError` enum with `#[from]` conversions.
- [ ] Re-exports for all stage-crate public types.

### 1.11 `caliper` binary — CLI surface (PRD §9)

- [ ] clap derive structure with global flags from PRD §9.2.
- [ ] Caliper-specific flags from PRD §9.3 (subset relevant to v0.1.0).
- [ ] Sub-command: `trace <input> -o <output>`.
- [ ] Sub-command: `inspect <input>`.
- [ ] Sub-command: `preview <input>`.
- [ ] Sub-command: `palette <input>`.
- [ ] Sub-command: `schema [<command>]` — JSON Schema Draft 2020-12 emission.
- [ ] Sub-command: `describe` — tool manifest.
- [ ] Output-mode detection cascade (PRD §9.4) — TTY vs piped vs `AI_AGENT` etc.
- [ ] JSON envelope (PRD §9.5) — `metadata`, `data`, `error`.
- [ ] Structured error type with runnable hint (PRD §9.7).
- [ ] Canonical exit-code map (PRD §9.6).
- [ ] Agent env-var detection (PRD §9.8).
- [ ] Help text with ≥ 2 examples per sub-command.
- [ ] `--dry-run` on every write-path.
- [ ] `--no-color` / `NO_COLOR` / `FORCE_COLOR` handling.

### 1.12 Assets & docs

- [ ] `assets/error-hint-catalog.json` populated with v0.1.0 hint set (PRD §9.7).
- [ ] `assets/schemas/trace.json`, `inspect.json`, etc. emitted by `caliper schema`.

### 1.13 Tests

- [ ] Unit-test pass on every crate (`cargo test`).
- [ ] Golden-file SVG comparison for 10-image corpus (`tests/golden/`).
- [ ] Agent-env detection test matrix (PRD §9.8).
- [ ] Exit-code map assertions.
- [ ] Schema round-trip — every `--json` output validates against `caliper schema <command>`.
- [ ] Input-validation tests — path traversal, control chars, oversize images rejected.
- [ ] Idempotency test — same input → same output (modulo `metadata.timestamp`).

---

## 2. v0.2.0 — Concurrency & TUI — `PRD.md` §17.2

### 2.1 Concurrency (PRD §6.1.1)

- [ ] Three-level Rayon parallelism: layer × tile × path.
- [ ] Tile decomposition (default 512 × 512; `--tile-size`).
- [ ] Union-find disconnected-contour discovery per tile.
- [ ] Lock-free assembly via `crossbeam::SegQueue`.
- [ ] Deterministic z-order output.
- [ ] NUMA-aware thread affinity where available.
- [ ] Scaling-efficiency benchmark — ≥ 85 % on 20 cores at 4K.

### 2.2 Tile stitching (PRD §8.7)

- [ ] `caliper-weld` boundary stitching — overlap regions merge paths crossing tile edges.
- [ ] Layer stacking (`stacked` default) and `cutout` hierarchical mode.

### 2.3 Batch command (PRD §9.10)

- [ ] `caliper batch <glob> -o <dir>` with cross-file Rayon parallelism.
- [ ] `--stdin` reads NUL- or newline-delimited paths from stdin.
- [ ] `-0` / `--print0` for `xargs -0` round-trip.
- [ ] `--format jsonl` for streaming progress.

### 2.4 Extended SIMD (PRD §6.1.2)

- [ ] AVX-512 kernels in `caliper-anvil` (16-wide; masked boundary ops).
- [ ] SVE / SVE2 kernels (aarch64 ARMv9+).
- [ ] Build profile `release-avx512` (`target-cpu=x86-64-v4`).
- [ ] Build profile `release-avx` (`target-cpu=x86-64-v3`).
- [ ] Build profile `release-portable` (`target-cpu=x86-64-v2`).

### 2.5 PDF writer (PRD §8.8, §13.2)

- [ ] `caliper-cast::pdf` — PDF 1.7 via `pdf-writer` with native vector paths.
- [ ] Validation: `qpdf --check` passes on generated PDFs.

### 2.6 `caliper-tui` — Interactive mode (PRD §10)

- [ ] `ratatui` + `crossterm` skeleton.
- [ ] Left pane — source raster (half-block; Sixel / Kitty where supported).
- [ ] Right pane — live vector preview, rerasterized as parameters change.
- [ ] Bottom pane — parameter sliders.
- [ ] Status bar — shape count, path length, max deviation, elapsed time, accelerator utilization, Assayer recommendation.
- [ ] Steelbore palette applied (six tokens, WCAG AA verified).
- [ ] Dual CUA + Vim keymap from PRD §10.2.
- [ ] Alt-screen buffer preserves scrollback on exit.
- [ ] Reduced-motion preference honored.
- [ ] Suppressed under `AI_AGENT` / `AGENT` / `CI` / `TERM=dumb` — falls back to JSON with stderr warning.
- [ ] `insta` snapshot tests for layout.

---

## 3. v0.3.0 — GPU Acceleration (Bellows) — `PRD.md` §17.3

### 3.1 `caliper-bellows` — wgpu backend (PRD §6.2)

- [ ] `wgpu::Instance` adapter enumeration.
- [ ] Adapter preference: discrete > integrated > CPU fallback.
- [ ] WGSL compute shader: AoS↔SoA transpose.
- [ ] WGSL compute shader: NLM denoising.
- [ ] WGSL compute shader: bilateral denoising.
- [ ] WGSL compute shader: OKLab color-space convert.
- [ ] WGSL compute shader: k-means quantization.
- [ ] WGSL compute shader: histogram (atomic-add reduction).
- [ ] WGSL compute shader: Sobel/Scharr edge detection.
- [ ] WGSL compute shader: morphology (erode/dilate).
- [ ] GPU-resident intermediate textures — only quantized masks return to host.
- [ ] CPU fallback for every shader.

### 3.2 Hardware detection (PRD §6.4)

- [ ] `caliper inspect --hardware` enumerates CPU SIMD, GPU adapters, NPU providers.
- [ ] `--hardware --json` structured output.
- [ ] `--accelerator <auto|cpu|gpu|npu|cpu+gpu|cpu+npu|all>` flag.
- [ ] Auto-selection logic in Assayer.

### 3.3 Benchmarks

- [ ] 1080p PNG → SVG with GPU ≤ 80 ms (PRD §6.5).
- [ ] 4K PNG → SVG with GPU ≤ 220 ms.
- [ ] 8K PNG → SVG with GPU ≤ 700 ms.
- [ ] PCIe-traffic assertion — only mask data crosses the bus on return.

---

## 4. v0.4.0 — NPU Acceleration (Touchstone) — `PRD.md` §17.4

### 4.1 `caliper-touchstone` — ort backend (PRD §6.3)

- [ ] `ort` crate integration; ONNX Runtime version pinned.
- [ ] Execution-provider priority list: TensorRT → CUDA → OpenVINO → VitisAI → QNN → CoreML → DirectML → CPU.
- [ ] Per-provider availability detection at startup.
- [ ] Graceful fallback when preferred provider unavailable.
- [ ] Provider reported in `metadata.hardware.npu` JSON field.

### 4.2 ML edge detection

- [ ] HED INT8-quantized model integration.
- [ ] BDCN INT8-quantized model integration.
- [ ] EDTER-distilled compact model integration.
- [ ] Assayer routes photographic / noisy inputs to ML edges.
- [ ] Inference latency < 100 ms on 40+ TOPS NPU asserted.

### 4.3 Model management (PRD §6.3 + PFA §6)

- [ ] `caliper models list` — show installed models with hashes.
- [ ] `caliper models pull <name>` — explicit-opt-in download; only network call in entire codebase.
- [ ] `caliper models verify` — BLAKE3 hash check against manifest.
- [ ] Audit: grep for any other network I/O outside `caliper models pull` — zero occurrences.
- [ ] Model manifest signed Ed25519 + ML-DSA-65 (PQC hybrid; PRD §12).

### 4.4 Shape classification (PRD §6.3 #4)

- [ ] Tiny shape classifier (< 1 MB) on NPU.
- [ ] Integration in `caliper-temper` primitive RANSAC pathway.

### 4.5 Benchmarks

- [ ] Photographic 4K with ML edges ≤ 1.2 s (PRD §6.5).
- [ ] NPU + GPU + CPU 4K trace ≤ 180 ms.

---

## 5. v0.5.0 — Format Expansion — `PRD.md` §17.5

### 5.1 EPS writer (PRD §8.8, §13.2)

- [ ] `caliper-cast::eps` — EPS 3.0 with proper `%%BoundingBox`.
- [ ] Validates with `ghostscript -dNOPAUSE -dBATCH -sDEVICE=nullpage`.

### 5.2 DXF writer (PRD §8.8, §13.2)

- [ ] `caliper-cast::dxf` — DXF R14, layer-per-color.
- [ ] Imports cleanly in LibreCAD and AutoCAD trial.

### 5.3 RANSAC primitives (PRD §7.1, §8.6)

- [ ] Circle fitting.
- [ ] Ellipse fitting.
- [ ] Rectangle fitting.
- [ ] Rounded-rectangle fitting.
- [ ] Star-polygon fitting.
- [ ] Parallel trials via Rayon.
- [ ] `--primitives auto|true|false` flag.
- [ ] Golden corpus — ≥ 95 % precision on logo set.

### 5.4 Symmetry detection (PRD §7.1)

- [ ] Fourier descriptors per contour.
- [ ] Moment invariants computation.
- [ ] Mirror + rotational symmetry detection.
- [ ] Symmetry enforced as fitting constraint.
- [ ] `--symmetry auto|true|false` flag.

### 5.5 Centerline mode (PRD §9.3)

- [ ] `--mode centerline` — skeletonization + tracing for line-art / technical drawings.
- [ ] Image-class routing table updated.

---

## 6. v1.0.0 — General Availability — `PRD.md` §17.6

### 6.1 `caliper mcp` server (PRD §9.9)

- [ ] `rmcp` integration.
- [ ] Transport: `stdio` (default).
- [ ] Transport: `sse` with `--port`.
- [ ] Transport: `streamable-http`.
- [ ] Lazy schema loading — `tools/list` returns names + one-line descriptions + capability tags only.
- [ ] `tools/get` loads full schema on demand.
- [ ] Capability tags: `read`, `write`, `destructive`, `idempotent`.
- [ ] Round-trip tested against Claude Code, Codex CLI, Cursor.

### 6.2 `libcaliper` C-ABI (PRD §11.2)

- [ ] `cbindgen.toml` configured.
- [ ] `caliper_trace_file` exported.
- [ ] `caliper_config_new` / `_set_*` / `_free` exported.
- [ ] `caliper_status_t` error-code enum.
- [ ] Every FFI function wrapped in `std::panic::catch_unwind`.
- [ ] Headers generated to `target/include/caliper.h`.
- [ ] `cdylib` builds: `.so` / `.dylib` / `.dll`.
- [ ] C example program in `examples/c/`.

### 6.3 WASM build (PRD §14.2)

- [ ] `wasm32-unknown-unknown` target compiles.
- [ ] `wasm-bindgen` + `wasm-bindgen-rayon` integration.
- [ ] JS example in `examples/wasm/`.
- [ ] Bundle size budget < 5 MB compressed.

### 6.4 CI matrix (PRD §14.3)

- [ ] Tier 1: `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`.
- [ ] Tier 2: `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.
- [ ] Tier 3: `wasm32-unknown-unknown`, `riscv64gc-unknown-linux-gnu`.
- [ ] Cross-shell tests on POSIX sh, Bash 5+, Brush, Nushell 0.111+, PowerShell 7.6+, Ion.

### 6.5 Compliance audit (PRD §18)

- [ ] All BLOCKER items in `PLAN.md` §6.1 green.
- [ ] All CRITICAL items in `PLAN.md` §6.2 green.
- [ ] All MAJOR items in `PLAN.md` §6.3 green.

### 6.6 Distribution (PRD §14.2)

- [ ] crates.io publish: `caliper-trace` + each workspace member.
- [ ] AUR PKGBUILD for `caliper`.
- [ ] Flatpak manifest published to Flathub.
- [ ] AppImage build.
- [ ] Homebrew formula in `homebrew-steelbore` tap.
- [ ] MSI installer for Windows.
- [ ] NixOS module in `lattice` repo.
- [ ] Release artifacts signed Ed25519 + ML-DSA-65 (PRD §12).

---

## 7. v1.1.0 — Region-Aware Routing — `PRD.md` §17.7

- [ ] Touchstone semantic segmentation model (text / line-art / fill / photo / background).
- [ ] Crucible accepts per-region `PipelineConfig`.
- [ ] Assayer emits region map alongside global classification.
- [ ] `--upscale {1, 2, 4}` flag (PRD §6.3 #3).
- [ ] Real-ESRGAN INT8 integration.
- [ ] SwinIR INT8 integration.
- [ ] Golden test: 256-px logo upscaled 2× indistinguishable from native 512-px trace.

---

## 8. v1.2.0 — Extended Outputs — `PRD.md` §17.8

- [ ] `caliper-cast::ai` — Adobe Illustrator v8 writer.
- [ ] AI file opens cleanly in Illustrator CC 2025.
- [ ] `caliper-cast::geojson` — GeoJSON writer for GIS workflows.
- [ ] GeoJSON validates against RFC 7946.
- [ ] `caliper-cast::hpgl` — HPGL writer for pen plotters and CNC.
- [ ] HPGL drives HP 7475A emulator.
- [ ] Plugin system — custom preprocessing / post-processing stages.
- [ ] Plugin ABI documented in `docs/plugins.md`.
- [ ] Example plugin in `examples/plugin-grayscale/`.

---

## 9. v2.0.0 — Intelligence — `PRD.md` §17.9

- [ ] Training pipeline for the Assayer classifier (separate `caliper-assayer-train` crate, optional feature).
- [ ] Trained model replaces rule-based classifier.
- [ ] Routing-accuracy benchmark — trained model ≥ 10 % better than rules.
- [ ] Cross-input shared palette in batch mode.
- [ ] Differentiable tracing — gradient-based parameter tuning behind `--experimental`.
- [ ] Diffusion-curve output behind `--experimental`.

---

## 10. Continuous (every phase)

These checks run in CI on every PR and nightly. Never fully "done" — kept green forever.

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] `cargo test --workspace` clean.
- [ ] `cargo deny check` clean (PRD §16).
- [ ] `cargo vet` clean.
- [ ] `cargo +nightly miri test -p caliper-anvil` clean.
- [ ] `cargo +nightly fuzz run <target>` corpus growing; nightly job green for every decoder.
- [ ] Criterion benchmark deltas vs PRD §6.5 reported in every PR description.
- [ ] Every CLI write-path has `--dry-run` test.
- [ ] Every `--json` output validates against `caliper schema <command>`.
- [ ] SPDX header present on every source file (xtask audit).
- [ ] UTF-8 without BOM on every text file (xtask audit).
- [ ] No network I/O outside `caliper models pull` (grep audit in CI).
- [ ] No `unsafe` outside `caliper-anvil` (compile-time forbid).

---

*Forged in Steelbore.*
