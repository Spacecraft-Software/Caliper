<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
-->

# CLAUDE.md

**Claude-Code-specific guidance for the Caliper repository.**

Start by reading [`AGENTS.md`](AGENTS.md) — that is the canonical agentic
manual. This file layers Claude-Code-specific conventions on top: which tools
to prefer, what to do before editing, and how to keep CI green without burning
turns.

---

## Architecture in 30 Seconds

14-crate workspace. The binary `caliper` depends only on `caliper-trace`
(public API re-exports) and `caliper-tui`. `caliper-trace` re-exports from
`caliper-crucible` (pipeline orchestrator). `caliper-crucible` drives the
seven-stage DAG across the stage crates:

```
Assayer → Smelt → Etch → Burin → Temper → Weld → Cast
(route)   (quant) (edge) (trace) (fit)    (stitch) (write)
```

`caliper-anvil` is the SIMD primitives layer — the only crate allowed
`unsafe`. `caliper-bellows` (wgpu GPU) and `caliper-touchstone`
(ONNX Runtime NPU) are feature-gated optional backends and excluded from
`default-members` in the workspace `Cargo.toml`. `caliper-tui` is the
`--format explore` ratatui interface, never depended on by library crates.

Status: pre-v0.1.0. Most crates are scaffolds with module-level docs;
[`TODO.md §1`](TODO.md) drives the current sprint.

---

## Commands

### Build / test / lint
- `cargo build --workspace` — builds default-members (CPU-only surface).
- `cargo build -p caliper-bellows` / `-p caliper-touchstone` — GPU / NPU
  crates are excluded from `default-members` (heavy native deps: wgpu,
  ort/ONNX Runtime), so build them by name.
- `cargo test --workspace` — full test suite.
- `cargo test -p <crate> <name>` — run a single test (e.g.
  `cargo test -p caliper-smelt kmeans::tests::converges`).
- `cargo test --test schema_roundtrip` — JSON-schema round-trip;
  a failure is P0 per pitfall #4 below.
- `cargo +nightly miri test -p caliper-anvil` — Miri-validates the only
  crate where `unsafe` is allowed.
- `cargo insta test` — TUI snapshot tests (`caliper-tui`).
- `cargo clippy --workspace --all-targets -- -D warnings` — CI lint gate.
- `cargo fmt --all --check` — CI format gate.

### Workspace audits (via `cargo xtask`, aliased in `.cargo/config.toml`)
- `cargo xtask spdx-audit` — every source file has a GPL-3.0-or-later header.
- `cargo xtask utf8-audit` — UTF-8 without BOM on every text file.
- `cargo xtask network-audit` — no network I/O outside `caliper models pull`.
- `cargo xtask cross-shell-test` — CLI round-trip under POSIX sh / Bash /
  Brush / Nushell / PowerShell / Ion.
- `cargo deny check` (alias: `cargo deny`) — license + advisory + sources
  audits.

### Dev shell
- `nix develop` — reproducible toolchain pinned in `flake.nix` +
  `rust-toolchain.toml` (Rust 1.88.0, Edition 2024).

---

## Before You Edit Any `.rs` File

Invoke the `rust-guidelines` skill. It is mandatory for every Rust touch, no
exceptions. Caliper's workspace `Cargo.toml` enforces the lint set, but the
skill's per-area guidance (universal, library, performance, safety, FFI, AI)
catches things the linter cannot.

For non-Rust work, the relevant skills are:

| You are about to …                          | Skill to invoke           |
|---------------------------------------------|---------------------------|
| Touch any `.rs` file                        | `rust-guidelines`         |
| Touch any CLI surface, `--json`, schema     | `spacecraft-cli-standard`  |
| Touch agentic UX, AGENTS.md, MCP            | `spacecraft-agentic-cli`   |
| Touch TUI, palette, typography              | `spacecraft-standard` + `spacecraft-theme-factory` |
| Touch documents (PRD/PLAN/TODO/README/docs) | `spacecraft-document-format` |
| Run a shell command                         | `spacecraft-cli-preference` + `spacecraft-cli-shell` |
| Install missing software                    | `spacecraft-missing-pkg`   |
| Generic Spacecraft Software compliance question | `spacecraft-standard`  |

---

## Preferred Tools

- **File reads** → `Read`. Never `cat`.
- **File edits** → `Edit` for diffs, `Write` for new files / full rewrites.
- **Search** → `rg` (ripgrep), `fd` for filenames. Never bare `grep` or `find`.
  See `spacecraft-cli-preference` for the full mapping.
- **JSON inspection** → `jaq`, not `jq`.
- **Tests** → `cargo test --workspace`. For TUI snapshots: `cargo insta test`.
- **Lints** → `cargo clippy --workspace --all-targets -- -D warnings`.
- **Format** → `cargo fmt`.

---

## Output Style

- Match the [Spacecraft Software Standard §12](https://SpacecraftSoftware.org/standard) date/time
  rules in every commit message, log line, and PR description: ISO 8601 UTC
  with `Z` suffix. 24-hour time. Metric units.
- Spacecraft Software palette in any colored output. Void Navy `#000027` is the canonical
  background.

---

## Common Pitfalls

1. **"Just add `unsafe` for speed."** No. Anvil is the only crate that allows
   it. If you need a SIMD intrinsic in another stage crate, expose a safe
   wrapper from anvil and depend on it.
2. **"Pretty-print the JSON so it's readable."** Pretty-printing is only for
   TTY output. Pipe / file output must be compact (`serde_json::to_writer`,
   not `to_writer_pretty`).
3. **"Just unwrap, it can't fail."** Public APIs return `Result<T, CaliperError>`.
   Internal panics are only for genuine programming errors per
   [M-PANIC-ON-BUG](https://microsoft.github.io/rust-guidelines/universal/#M-PANIC-ON-BUG).
4. **"Skip the schema test, the round-trip is obvious."** No. CI runs
   `cargo test --test schema_roundtrip` and a failing round-trip is a P0.
5. **"Cache the model auto-download in `~/.cache`."** No. PFA §6 — no network
   I/O outside `caliper models pull`. The cache is read-only at runtime.
6. **"`cargo build --workspace` is green, so my GPU change is fine."** No.
   `caliper-bellows` and `caliper-touchstone` are excluded from
   `default-members`. Build them by name (`cargo build -p caliper-bellows`)
   before claiming a GPU/NPU change works.

---

## Where to Stash Notes Between Sessions

Use the conversation memory (`/home/mj/.claude/projects/-spacecraft-software-caliper/memory/`).
Anything you learn about the user's workflow preferences, project context, or
non-obvious decisions belongs there — not in PR descriptions or comments.

The PRD / PLAN / TODO are the *project's* memory and are versioned in git.
Memory files are *agent's* memory and are session-local.

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)&gt;
Copyright (c) 2026 Mohamed Hammad &nbsp;|&nbsp; License: GPL-3.0-or-later
[https://Caliper.SpacecraftSoftware.org/](https://Caliper.SpacecraftSoftware.org/)
