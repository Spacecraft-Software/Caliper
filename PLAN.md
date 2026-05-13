<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
Steelbore Theme: Void Navy (#000027) bg, Molten Amber (#D98E32) body,
Steel Blue (#4B7EB0) H1, Radium Green (#50FA7B) H2, Liquid Coolant (#8BE9FD) H3
Fonts: Share Tech Mono (headings), Inconsolata (body)
-->

# CALIPER — Implementation Plan

**Engineering execution document derived from `PRD.md` v2.0**

| Field            | Value                                          |
|------------------|------------------------------------------------|
| Project          | Caliper                                        |
| Document         | PLAN.md                                        |
| Document Version | 1.0                                            |
| Document Date    | 2026-05-12T00:00:00Z                           |
| Author           | Mohamed Hammad                                 |
| License          | GPL-3.0-or-later                               |
| Governing PRD    | `PRD.md` v2.0 (2026-04-30)                     |
| Standards        | Steelbore Standard v1.0 + SFRS v1.0.0 + Agentic CLI v1.0 |
| Status           | Draft — ready for execution                    |

---

## 1. Context

Caliper is a Rust-native raster-to-vector tracing engine governed by The Steelbore Standard v1.0 and SFRS v1.0.0 (see `PRD.md` §1). It composes well-grounded published algorithms (Suzuki-Abe contour following, O(n) Bézier fitting, RANSAC primitive recognition, OKLab k-means quantization) under a novel adaptive meta-algorithm — the **Assayer** — that routes each input to the optimal pipeline configuration across heterogeneous CPU/GPU/NPU compute.

This plan translates the PRD's product narrative into an execution document: phased delivery aligned with the PRD §17 roadmap, per-crate workstreams, the critical files to create first, the compliance gates that block release, the verification strategy that proves quality, and the risks that need active mitigation.

**"Done" for v1.0.0** means: all 14 crates in `PRD.md` §2.1 implemented; CPU + GPU + NPU pipelines functioning; CLI + TUI + MCP server + libcaliper FFI all stabilized; the full PRD §18 compliance checklist green; Tier 1 and Tier 2 CI targets passing; distribution channels live for crates.io, AUR, Flatpak, Homebrew, MSI, NixOS, and WASM.

---

## 2. Guiding Principles

Recap of `PRD.md` §4 — these constraints govern every implementation decision.

- **Precision through measurement.** Tracing quality is measurable (sub-pixel deviation, shape count, total path length, curve smoothness, compression ratio) and surfaced live. See `PRD.md` §4.
- **Composition over invention.** Use published algorithms verbatim; the only novel component is the Assayer routing meta-algorithm. See `PRD.md` §4 + §7.2.
- **Hardware-conscious from the ground up.** Three-tier compute (CPU + GPU + NPU) is declared per stage in the pipeline DAG, not bolted on. See `PRD.md` §6.
- **Two co-equal readers.** Every output is rendered for both human-in-TTY and AI-agent-via-pipe — neither rendering converts to the other. See `PRD.md` §4 + §9.
- **Full Steelbore compliance.** GPL-3.0-or-later, SPDX headers on every source file, ISO 8601 UTC, 24-hour time, metric-only units, WCAG 2.1 AA contrast, dual CUA+Vim keybindings, six-token palette, Share Tech Mono / Inconsolata typography, no telemetry, no auto-network, local-storage default. See `PRD.md` §4 + §18.

---

## 3. Workspace Topology

Cargo workspace at the repository root, `resolver = "2"`, `edition = "2024"`, `rust-version = "1.82"`. All 14 crates live under `crates/<name>/`. Public API is re-exported through `caliper-trace`; the binary `caliper` depends only on `caliper-trace` and `caliper-tui`.

### 3.1 Crate Roles (mirrors `PRD.md` §2.1)

| Crate                  | Role                                                              |
|------------------------|-------------------------------------------------------------------|
| `caliper`              | Top-level binary (CLI + TUI dispatcher)                           |
| `caliper-trace`        | Public library API surface (re-exports)                           |
| `caliper-crucible`     | Pipeline orchestrator — drives the seven-stage DAG                |
| `caliper-assayer`      | Adaptive pipeline router — analyzes input, selects algorithms     |
| `caliper-smelt`        | Color quantization (k-means OKLab, median-cut, octree)            |
| `caliper-etch`         | Edge detection (Sobel/Scharr SIMD; HED/BDCN ML pathway)           |
| `caliper-burin`        | Contour tracing (Suzuki-Abe; connected-component labeling)        |
| `caliper-temper`       | Path fitting (O(n) spline; corner detection; primitive RANSAC)    |
| `caliper-weld`         | Tile-boundary stitching and layer assembly                        |
| `caliper-cast`         | Output encoding (SVG, PDF, EPS, DXF writers)                      |
| `caliper-bellows`      | GPU compute backend (wgpu / WGSL compute shaders)                 |
| `caliper-touchstone`   | NPU/ML backend (ort + ONNX Runtime execution providers)           |
| `caliper-anvil`        | SIMD kernels (std::simd portable + std::arch intrinsics)          |
| `caliper-tui`          | Ratatui interface for `--format explore`                          |

### 3.2 Dependency Direction

```
                        caliper (bin)
                            │
              ┌─────────────┴─────────────┐
              ▼                           ▼
       caliper-trace                 caliper-tui
              │
              ▼
        caliper-crucible
   ┌──────────┼──────────────────────────┐
   │          │                          │
   ▼          ▼                          ▼
assayer  {smelt, etch, burin,       {bellows, touchstone}
         temper, weld, cast}                │
                  │                         ▼
                  └──────► caliper-anvil ◄──┘
```

`caliper-anvil` is the SIMD primitives layer — depended on by stage crates, depends on nothing internal. `caliper-bellows` and `caliper-touchstone` are feature-gated optional backends. No cycles. No dependency on `caliper-tui` from any library crate.

---

## 4. Delivery Phases

Each phase corresponds 1:1 with a roadmap milestone in `PRD.md` §17. Phases are mostly sequential; cross-cutting workstreams (§5) thread through all of them.

### 4.1 v0.1.0 — Foundation (CPU-only) — `PRD.md` §17.1

**Scope:** Workspace bootstrap; all 14 crate skeletons; CPU-only pipeline producing SVG output; minimum-viable CLI surface.

**Exit criteria:**
- `caliper trace input.png -o output.svg` succeeds on a 1080p PNG.
- All six v0.1.0 sub-commands (`trace`, `inspect`, `preview`, `palette`, `schema`, `describe`) implemented.
- JSON envelope (`PRD.md` §9.5), structured errors (`PRD.md` §9.7), canonical exit-code map (`PRD.md` §9.6).
- SSE2, AVX2, NEON kernels in `caliper-anvil` Miri-clean.
- `AGENTS.md`, `CLAUDE.md`, `SKILL.md`, `CONTRIBUTING.md`, `README.md` at repo root.

**Dependencies:** None.
**TODO section:** `TODO.md` §1.

### 4.2 v0.2.0 — Concurrency and TUI — `PRD.md` §17.2

**Scope:** Three-level Rayon parallelism; `caliper batch` cross-file parallelism; AVX-512 + SVE2 kernels; PDF writer; `caliper-tui` (`--format explore`).

**Exit criteria:**
- Scaling efficiency ≥ 85 % on 20 cores at 4K (PRD §6.5).
- TUI passes `insta` snapshot tests; dual CUA + Vim keybindings (PRD §10.2).
- PDF 1.7 output validates with `qpdf --check`.

**Dependencies:** v0.1.0.
**TODO section:** `TODO.md` §2.

### 4.3 v0.3.0 — GPU Acceleration (Bellows) — `PRD.md` §17.3

**Scope:** `caliper-bellows` wgpu backend; GPU-eligible stages (transpose, denoise, OKLab convert, quantize, histogram, Sobel/Scharr, morphology); hardware detection; `--accelerator` flag.

**Exit criteria:**
- `caliper inspect --hardware --json` enumerates CPU SIMD level, GPU adapters, NPU providers.
- 4K trace with `--accelerator gpu` ≤ 220 ms on RTX 4070 (PRD §6.5).
- PCIe traffic confirmed minimized: only quantized layer masks return to host.

**Dependencies:** v0.2.0.
**TODO section:** `TODO.md` §3.

### 4.4 v0.4.0 — NPU Acceleration (Touchstone) — `PRD.md` §17.4

**Scope:** `caliper-touchstone` ort/ONNX Runtime integration; ML edge detection (HED/BDCN INT8); `caliper models` command tree; BLAKE3 model verification.

**Exit criteria:**
- Execution-provider auto-select works across at least three of {CUDA, TensorRT, OpenVINO, CoreML, DirectML, CPU}.
- `caliper models pull <name>` is the *only* code path that performs network I/O; PFA §6 audit clean.
- Photographic 4K ML-edge trace ≤ 1.2 s (PRD §6.5).

**Dependencies:** v0.3.0.
**TODO section:** `TODO.md` §4.

### 4.5 v0.5.0 — Format Expansion — `PRD.md` §17.5

**Scope:** EPS 3.0 and DXF R14 writers; RANSAC primitive fitting; symmetry detection and enforcement; `--mode centerline`.

**Exit criteria:**
- EPS validates with `ghostscript -dNOPAUSE -dBATCH`.
- DXF imports cleanly into AutoCAD/LibreCAD.
- RANSAC primitives detected for logos with ≥ 95 % precision on a golden corpus.

**Dependencies:** v0.4.0.
**TODO section:** `TODO.md` §5.

### 4.6 v1.0.0 — General Availability — `PRD.md` §17.6

**Scope:** `caliper mcp` server; `libcaliper` C-ABI freeze; WASM build; Tier 1/2/3 CI matrix; compliance audit; full distribution.

**Exit criteria:**
- All BLOCKER and CRITICAL items in §6 below ticked.
- `caliper mcp --transport stdio` round-trips with Claude Code, Codex CLI, Cursor.
- `cbindgen` headers stable; `libcaliper.so/.dylib/.dll` shipping.
- Public packages live on crates.io, AUR, Flatpak, Homebrew, MSI, NixOS module, WASM.

**Dependencies:** v0.5.0.
**TODO section:** `TODO.md` §6.

### 4.7 v1.1.0 — Region-Aware Routing — `PRD.md` §17.7

**Scope:** Semantic segmentation model in Touchstone; per-region pipeline config in Crucible; `--upscale` via Real-ESRGAN / SwinIR.

**Exit criteria:**
- Mixed-content document (text + photo + line-art) traces with per-region routing without global compromise.
- `--upscale 2` on a 256-px logo produces output indistinguishable from native 512-px tracing on a golden test.

**Dependencies:** v1.0.0.
**TODO section:** `TODO.md` §7.

### 4.8 v1.2.0 — Extended Outputs — `PRD.md` §17.8

**Scope:** AI v8, GeoJSON, HPGL writers; plugin system.

**Exit criteria:**
- AI files open in Adobe Illustrator CC 2025 without warnings.
- GeoJSON validates against RFC 7946.
- HPGL drives an HP 7475A emulator.
- Plugin ABI documented; one third-party plugin example in `examples/`.

**Dependencies:** v1.0.0 (parallel with v1.1.0).
**TODO section:** `TODO.md` §8.

### 4.9 v2.0.0 — Intelligence — `PRD.md` §17.9

**Scope:** Trained Assayer model; cross-input palette sharing; differentiable tracing; diffusion-curve research preview.

**Exit criteria:**
- Trained Assayer beats rule-based classifier by ≥ 10 % on routing accuracy benchmark.
- Diffusion-curve output behind `--experimental` feature flag.

**Dependencies:** v1.1.0 + v1.2.0.
**TODO section:** `TODO.md` §9.

---

## 5. Cross-Cutting Workstreams

These threads run continuously across all phases. They are tracked in `TODO.md` §10.

### 5.1 CLI / UX — `PRD.md` §9

Every new sub-command MUST land with:
- `--json`, `--format`, `--fields` support.
- Canonical exit code mapped (PRD §9.6).
- Structured error with runnable hint (PRD §9.7).
- `--dry-run` if it writes.
- Schema published via `caliper schema <command>`.
- Help text with ≥ 2 examples.
- Tests for agent-env vars (`AI_AGENT`, `AGENT`, `CI`, `CLAUDECODE`, `CURSOR_AGENT`, `GEMINI_CLI`).

### 5.2 TUI — `PRD.md` §10

`caliper-tui` is built on `ratatui` + `crossterm`. All UI work obeys:
- Steelbore six-token palette (Void Navy `#000027`, Molten Amber `#D98E32`, Steel Blue `#4B7EB0`, Radium Green `#50FA7B`, Red Oxide `#FF5C5C`, Liquid Coolant `#8BE9FD`).
- Dual CUA + Vim keybindings — both bound to every action.
- Alt-screen buffer used so scrollback survives exit.
- Suppressed under `AI_AGENT` / `AGENT` / `CI` / `TERM=dumb` — never trap an agent.
- Reduced-motion preference honored.

### 5.3 Library / FFI — `PRD.md` §11

- Rust API in `caliper-trace`: `TracingConfig` builder, `trace()` free function, `VectorDocument` with format-specific writers, all errors as `Result<T, CaliperError>`.
- C-ABI via `cbindgen`. Every FFI function wrapped in `std::panic::catch_unwind`; panics never cross the boundary.
- ABI stability frozen at v1.0.0; subsequent breaks require major-version bump.

### 5.4 Security Hardening — `PRD.md` §12

- Zero `unsafe` outside `caliper-anvil`; all anvil `unsafe` Miri-validated.
- Release builds: `-Crelocation-model=pic`, `-Cstack-protector=strong`, `-Zsanitizer=cfi` where available.
- Input validation: path canonicalization, allow-list, control-character rejection at parse, bounds-check vs schema min/max, no shell interpolation.
- Image decoder fuzz corpus maintained; nightly `cargo fuzz` CI job.
- `--max-dimension` enforced before allocation (default 32 768 × 32 768).
- PFA §6: zero network I/O outside `caliper models pull`.
- Release artifacts signed Ed25519 + ML-DSA-65 (PQC hybrid per Standard §3.3).

### 5.5 Format Support — `PRD.md` §13

Input formats (PNG, JPEG, BMP, WebP, TIFF, AVIF, QOI, TGA, PNM, ICO, DDS, OpenEXR, farbfeld) are delivered with v0.1.0 via the `image` crate.

Output formats roll out per the version column in PRD §13.2:
- v1.0: SVG 1.1, PDF 1.7.
- v1.1: EPS 3.0, DXF R14.
- v1.2: AI v8, GeoJSON, HPGL.

### 5.6 Build & Distribution — `PRD.md` §14

Build profiles `dev`, `release`, `release-native`, `release-portable`, `release-avx`, `release-avx512` target Microsoft's x86_64 microarchitecture levels. Distribution channels enumerated in §4.6 above.

### 5.7 Testing — `PRD.md` §15

Twelve categories: unit, integration, fuzz, Miri, benchmark, visual regression, cross-platform, JSON schema, exit codes, agent env, idempotency, input validation, cross-shell, UTF-8. Mapped to concrete commands in §7 below.

### 5.8 Dependency Policy — `PRD.md` §16

- All deps GPL-3.0-or-later compatible.
- `cargo-deny` license check in CI.
- No proc-macro crates beyond `clap` and `serde`.
- Pinned `rust-toolchain.toml` (MSRV 1.82.0).
- `cargo-vet` in CI; trust roots minimal.

---

## 6. Compliance Gates

Mirrors `PRD.md` §18, partitioned by severity. Each gate names the crate or file that enforces it. All BLOCKER + CRITICAL items must be green before any GA release.

### 6.1 BLOCKER (must pass before v1.0.0)

| # | Gate                                                          | Enforced By                                        |
|---|---------------------------------------------------------------|----------------------------------------------------|
| 1 | ISO 8601 + UTC timestamps everywhere                          | `caliper-trace` time module; `jiff` dependency     |
| 2 | UTF-8 without BOM                                             | `caliper-cast` writers; CI grep job                |
| 3 | POSIX-first default output                                    | `caliper` bin output mode cascade                  |
| 4 | `--json` on every data-returning command                      | `caliper` clap derive macros + tests               |
| 5 | stdout = data only, stderr = everything else                  | `caliper-trace` logging; tracing subscriber config |
| 6 | Structured errors on stderr in machine mode                   | `caliper-trace::CaliperError`                      |
| 7 | Path canonicalization + allow-list validation                 | `caliper` input-validation module                  |
| 8 | Non-TTY destructive ops require `--yes` / `--force`           | `caliper` clap argument validators                 |
| 9 | `caliper schema <command>` emits JSON Schema Draft 2020-12    | `caliper` schema command + `schemars` integration  |
| 10| GPL-3.0-or-later headers on every source file                 | `xtask` SPDX-check; `deny.toml`                    |
| 11| Memory safety — `unsafe` confined to `caliper-anvil`          | `clippy.toml`; crate-level `forbid(unsafe_code)`   |
| 12| `caliper-anvil unsafe` Miri-clean                             | CI `cargo miri test -p caliper-anvil`              |

### 6.2 CRITICAL (must pass before v1.0.0)

| # | Gate                                                          | Enforced By                                        |
|---|---------------------------------------------------------------|----------------------------------------------------|
| 1 | `NO_COLOR` honored                                            | `caliper` color-detection cascade                  |
| 2 | `--dry-run` on every write command                            | `caliper` clap; tests in §7                        |
| 3 | `AI_AGENT=1` → JSON + no-color + non-interactive              | `caliper` env-detection module                     |
| 4 | Every error response has non-empty runnable `hint`            | `assets/error-hint-catalog.json`; type-level check |
| 5 | `--fields` honored on every list/get command                  | `caliper` field-projection helper                  |
| 6 | MCP server uses lazy schema loading                           | `caliper mcp` (rmcp integration)                   |
| 7 | Control-character rejection on string arguments               | `caliper` input-validation module                  |
| 8 | PQC readiness — model signature verification BLAKE3 + Ed25519 | `caliper-touchstone` model loader                  |
| 9 | PFA — no auto-network, local-storage default                  | `caliper models pull` is the only network call     |

### 6.3 MAJOR (target before v1.0.0; soft-gate)

| # | Gate                                                          | Enforced By                                        |
|---|---------------------------------------------------------------|----------------------------------------------------|
| 1 | AGENTS.md, CLAUDE.md, SKILL.md, CONTRIBUTING.md at repo root  | Repo root files; CI presence check                 |
| 2 | `--format jsonl` on streaming-eligible commands               | `caliper batch`, `caliper inspect`                 |
| 3 | JSON output omits null fields                                 | `serde(skip_serializing_if = "Option::is_none")`   |
| 4 | JSON output compact (not pretty) when stdout non-TTY          | `caliper` output formatter                         |
| 5 | Steelbore palette in TUI                                      | `caliper-tui` theme module                         |
| 6 | Dual CUA + Vim keybindings in TUI                             | `caliper-tui` keymap module                        |
| 7 | Cross-shell tests pass on POSIX sh, Bash 5+, Brush, Nushell 0.111+, PowerShell 7.6+, Ion | CI matrix |

---

## 7. Verification & Test Plan

Maps `PRD.md` §15's twelve test categories to runnable commands and crates.

| Category              | Command                                                              | Crate(s)                                       |
|-----------------------|----------------------------------------------------------------------|------------------------------------------------|
| Unit                  | `cargo test`                                                         | All crates                                     |
| Integration           | `cargo test --test golden_files`                                     | `caliper` (root)                               |
| Fuzz                  | `cargo +nightly fuzz run decode_png` (and one target per decoder)    | `fuzz/`                                        |
| Miri                  | `cargo +nightly miri test -p caliper-anvil`                          | `caliper-anvil`                                |
| Benchmark             | `cargo bench --bench end_to_end`                                     | `caliper-trace` (criterion)                    |
| Visual regression     | `cargo insta test`                                                   | `caliper-tui`                                  |
| Cross-platform        | CI matrix (Tier 1/2/3 from PRD §2)                                   | `.github/workflows/ci.yml`                     |
| JSON schema           | `cargo test --test schema_roundtrip`                                 | `caliper`                                      |
| Exit codes            | `cargo test --test exit_codes`                                       | `caliper`                                      |
| Agent env detection   | `cargo test --test agent_env`                                        | `caliper`                                      |
| Idempotency           | `cargo test --test idempotent_trace`                                 | `caliper-trace`                                |
| Input validation      | `cargo test --test input_validation`                                 | `caliper`                                      |
| Cross-shell roundtrip | `xtask cross-shell-test`                                             | `xtask`                                        |
| UTF-8 / no BOM        | `xtask utf8-audit`                                                   | `xtask`                                        |

### 7.1 Benchmark Targets — `PRD.md` §6.5

| Workload                              | CPU only | CPU + GPU | CPU + GPU + NPU |
|---------------------------------------|----------|-----------|-----------------|
| 1080p PNG → SVG (16 colors)           | < 200 ms | < 80 ms   | < 60 ms         |
| 4K PNG → SVG (16 colors)              | < 600 ms | < 220 ms  | < 180 ms        |
| 8K PNG → SVG (16 colors)              | < 2.0 s  | < 700 ms  | < 600 ms        |
| Gigapixel BW scan → SVG               | < 30 s   | < 10 s    | < 8 s           |
| Photographic 4K (ML edges)            | N/A      | N/A       | < 1.2 s         |
| Batch 1000× 1080p PNGs                | < 60 s   | < 25 s    | < 22 s          |
| Scaling efficiency (20 cores)         | ≥ 85 %   | ≥ 85 %    | ≥ 85 %          |
| Peak memory (4K, 16 colors)           | < 512 MB | < 768 MB  | < 1024 MB       |

Criterion runs report deltas vs these targets in every PR. Statistical regression alerts gate the GA release.

---

## 8. Critical Files (to be created)

The first sprint touches these paths. Roles map to PRD sections in parentheses.

### 8.1 Workspace Root

- `Cargo.toml` — workspace manifest, resolver "2", edition 2024
- `rust-toolchain.toml` — pin to 1.82.0
- `rustfmt.toml`, `clippy.toml` — style + lint config
- `deny.toml` — license allow-list (GPL-3.0-or-later compatible)
- `.gitignore`, `.editorconfig`
- `flake.nix`, `flake.lock` — reproducible dev shell (PRD §14.2 Nix)
- `AGENTS.md`, `CLAUDE.md`, `SKILL.md`, `CONTRIBUTING.md`, `README.md` (PRD §17.1)
- `LICENSE` (GPL-3.0-or-later full text)
- `.github/workflows/ci.yml`, `release.yml`, `fuzz.yml`
- `xtask/Cargo.toml`, `xtask/src/main.rs` — SPDX audit, cross-shell tests, UTF-8 audit

### 8.2 Asset Files

- `assets/error-hint-catalog.json` — canonical runnable-hint catalog (PRD §9.7)
- `assets/schemas/*.json` — JSON Schema Draft 2020-12 per sub-command
- `assets/models/.gitkeep` — Touchstone model directory (downloaded on demand)

### 8.3 Per-Crate Skeletons

Each `crates/<name>/` gets `Cargo.toml`, `src/lib.rs` (or `src/main.rs` for the binary), and an `SPDX-License-Identifier: GPL-3.0-or-later` header at the top of every source file.

- `crates/caliper/{Cargo.toml, src/main.rs, src/cli.rs, src/env.rs, src/output.rs, src/error.rs}`
- `crates/caliper-trace/{Cargo.toml, src/lib.rs, src/config.rs, src/document.rs}`
- `crates/caliper-crucible/{Cargo.toml, src/lib.rs, src/dag.rs, src/scheduler.rs}`
- `crates/caliper-assayer/{Cargo.toml, src/lib.rs, src/features.rs, src/router.rs}`
- `crates/caliper-smelt/{Cargo.toml, src/lib.rs, src/kmeans.rs, src/oklab.rs}`
- `crates/caliper-etch/{Cargo.toml, src/lib.rs, src/sobel.rs, src/scharr.rs}`
- `crates/caliper-burin/{Cargo.toml, src/lib.rs, src/suzuki_abe.rs, src/simplify.rs}`
- `crates/caliper-temper/{Cargo.toml, src/lib.rs, src/bezier.rs, src/corners.rs, src/ransac.rs}`
- `crates/caliper-weld/{Cargo.toml, src/lib.rs}`
- `crates/caliper-cast/{Cargo.toml, src/lib.rs, src/svg.rs, src/pdf.rs, src/eps.rs, src/dxf.rs}`
- `crates/caliper-bellows/{Cargo.toml, src/lib.rs, shaders/*.wgsl}`
- `crates/caliper-touchstone/{Cargo.toml, src/lib.rs, src/providers.rs, src/models.rs}`
- `crates/caliper-anvil/{Cargo.toml, src/lib.rs, src/sse2.rs, src/avx2.rs, src/avx512.rs, src/neon.rs, src/sve2.rs}`
- `crates/caliper-tui/{Cargo.toml, src/lib.rs, src/theme.rs, src/keymap.rs, src/widgets/*.rs}`

---

## 9. Risk Register

| Risk                                                  | Likelihood | Impact | Mitigation                                                                                                |
|-------------------------------------------------------|------------|--------|-----------------------------------------------------------------------------------------------------------|
| ONNX Runtime ABI / `ort` crate churn                  | Med        | High   | Pin `ort` minor; quarterly upgrade window; isolate provider selection in `caliper-touchstone::providers`. |
| `wgpu` backend portability (Vulkan/Metal/DX12/GL)     | Med        | Med    | CI matrix runs `caliper trace --accelerator gpu` on each backend; WGSL only — no native shader leakage.   |
| SIMD `unsafe` correctness across SSE2/AVX2/AVX-512/NEON/SVE2 | Med        | High   | Miri job in CI; scalar reference comparison test per kernel; nightly `cargo fuzz` of anvil kernels.       |
| Cross-shell argv handling (Nushell, Ion, PowerShell, Brush) | Med        | Med    | `xtask cross-shell-test` runs every CLI surface in each shell; argv arrays only; no shell interpolation.  |
| MCP transport / `rmcp` API instability                | High       | Med    | Pin `rmcp` version; integration tests against Claude Code + Codex CLI + Cursor; transport behind feature. |
| GPL-3.0 dependency-license drift                      | Low        | High   | `cargo-deny check` blocks CI; `cargo-vet` audits new crates; manual review for every dep addition.        |

---

## 10. Open Questions

Items surfaced during implementation that need decisions. Populated as work progresses.

- _(none yet)_

---

## 11. References

- `PRD.md` v2.0 — product requirements (this plan's source of truth)
- The Steelbore Standard v1.0 — `github.com/Steelbore/steelbore-standard`
- Steelbore SFRS v1.0.0 — Dual-Mode Self-Documenting CLI Framework
- Steelbore Agentic CLI Standard v1.0
- Microsoft Pragmatic Rust Guidelines — `microsoft.github.io/rust-guidelines`
- `TODO.md` — sequenced backlog tied to this plan's phases

---

*Forged in Steelbore.*
