<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
-->

# Contributing to Caliper

Thanks for your interest. Caliper is a personal hobby project under the
[Steelbore](https://Steelbore.com/) umbrella — see [`README.md`](README.md) for
the full project posture.

This document covers: how to set up a dev environment, what gets merged, the
commit/PR conventions, the CI gates a change must pass, and the embargo process
for security issues.

---

## Scope & Acceptance

Contributions are **welcome** but acceptance is at the maintainer's sole
discretion. Rejection reflects fit with the roadmap, not the quality of your
work. To maximise the chance of merge:

- Open an issue **before** writing code for anything non-trivial. Cite the PRD
  or TODO section your change targets.
- Stay aligned with `PRD.md` v2.0, `PLAN.md` v1.0, and the milestone the work
  belongs to in `TODO.md`. Out-of-scope features will be deferred or declined.
- Keep PRs small. One concern per PR.

---

## Development Setup

Caliper builds on Linux, macOS, and Windows. The reproducible path is via Nix:

```sh
nix develop                           # drops you into a shell with everything pinned
cargo build --workspace
cargo test --workspace
```

Without Nix, install the Rust toolchain pinned in
[`rust-toolchain.toml`](rust-toolchain.toml) (1.82.0) and the components listed
there (`rustfmt`, `clippy`, `miri`).

Optional system deps:

- For the `gpu` feature: Vulkan / Metal / DX12 / WebGPU drivers (handled by
  `wgpu` at runtime).
- For the `npu` feature: ONNX Runtime shared libraries. The `ort` crate picks
  these up automatically.
- For the WASM target: `wasm-pack` or `cargo-component`.

---

## Commit Conventions

We follow Conventional Commits with these scopes:

| Scope        | Use for                                           |
|--------------|---------------------------------------------------|
| `anvil`      | SIMD kernels                                      |
| `crucible`   | Pipeline orchestrator / DAG                       |
| `assayer`    | Adaptive router                                   |
| `smelt`      | Color quantization                                |
| `etch`       | Edge detection                                    |
| `burin`      | Contour tracing                                   |
| `temper`     | Path fitting                                      |
| `weld`       | Tile stitching                                    |
| `cast`       | Output writers (svg / pdf / eps / dxf / …)        |
| `bellows`    | GPU backend                                       |
| `touchstone` | NPU backend                                       |
| `tui`        | Ratatui interface                                 |
| `cli`        | `caliper` binary front-end                        |
| `trace`      | Public library API                                |
| `xtask`      | Build tooling                                     |
| `ci`         | GitHub Actions / workflow files                   |
| `docs`       | README / PRD / PLAN / TODO / per-crate docs       |
| `deps`       | Dependency bumps                                  |

Types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `chore`, `build`.
Subject in imperative mood, max 72 chars.

Every commit must be signed off with `Signed-off-by:` (DCO). By signing off
you confirm the [Developer Certificate of Origin](https://developercertificate.org/).
The DCO is recorded as `Signed-off-by: Your Name <you@example.com>` in the
commit trailer (`git commit -s`).

## License of Contributions

By submitting a contribution you agree that it is licensed under
**GPL-3.0-or-later**, the same license as the rest of Caliper, and that you have
the right to submit it.

---

## CI Gates

Every PR must pass the following checks. Many of these correspond to the
BLOCKER and CRITICAL items in [`PLAN.md` §6](PLAN.md#6-compliance-gates).

| Gate                                                          | Command                                            |
|---------------------------------------------------------------|----------------------------------------------------|
| Format                                                        | `cargo fmt --check`                                |
| Lint                                                          | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests                                                         | `cargo test --workspace`                           |
| License audit                                                 | `cargo deny check`                                 |
| Supply-chain audit                                            | `cargo vet`                                        |
| `unsafe` correctness                                          | `cargo +nightly miri test -p caliper-anvil`        |
| SPDX header on every source file                              | `cargo xtask spdx-audit`                           |
| UTF-8 without BOM on every text file                          | `cargo xtask utf8-audit`                           |
| `--json` round-trips against the published schema             | `cargo test --test schema_roundtrip`               |
| No network I/O outside `caliper models pull`                  | `cargo xtask network-audit`                        |

PRs that touch performance-critical paths must include criterion deltas in the
description, compared against the targets in [PRD §6.5](PRD.md#65-benchmark-targets).

---

## Code Style

- All Rust code follows the [Steelbore Rust Guidelines](https://github.com/Steelbore/rust-guidelines)
  (Microsoft Pragmatic Rust Guidelines + Steelbore overlays). The lints in
  workspace `Cargo.toml` enforce most of this mechanically.
- All source files start with `// SPDX-License-Identifier: GPL-3.0-or-later`.
- Public APIs need `Debug` and `Display` where applicable (M-PUBLIC-DEBUG / M-PUBLIC-DISPLAY).
- `unsafe` is forbidden outside `caliper-anvil`. New `unsafe` in anvil needs
  a `// SAFETY:` block plus a Miri test.
- ISO 8601 UTC timestamps with `Z` suffix. 24-hour time. Metric units.

---

## TUI / Visual Work

- Steelbore six-token palette only (see [`PRD.md` §10.3](PRD.md#103-steelbore-visual-theme-standard-8)).
- WCAG 2.1 AA contrast for every foreground / background pairing.
- Dual CUA + Vim keybindings — both bound to every action.
- Reduced-motion preference respected.

---

## <a id="security"></a>Security

Report security issues privately to
**[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)** with the
subject line `[Caliper Security]`. Please **do not** open public issues for
unpatched vulnerabilities.

Embargo policy:

1. Acknowledge within 7 days.
2. Coordinated disclosure target: 90 days from first report.
3. CVE assignment via the GitHub Security Advisory pathway.
4. Credit in `CREDITS.md` and the release notes (unless you ask otherwise).

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)&gt;
[https://Caliper.Steelbore.com/](https://Caliper.Steelbore.com/)
