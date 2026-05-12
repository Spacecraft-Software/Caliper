<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
-->

# AGENTS.md

**Instructions for AI agents working in the Caliper repository.**
*This file is the single source of truth that all agentic clients (Claude Code,
Codex CLI, Cursor, Gemini CLI, generic MCP clients) should read first.*

If you are a human reading this — these instructions also apply to you, but
[`CONTRIBUTING.md`](CONTRIBUTING.md) is more directly useful.

---

## What Caliper Is

Rust-native raster-to-vector tracing engine. 14-crate workspace governed by
[`PRD.md`](PRD.md), [`PLAN.md`](PLAN.md), and [`TODO.md`](TODO.md). Pre-v0.1.0
status — most crates are stubs at the moment.

The CLI follows the **Steelbore Dual-Mode Self-Documenting CLI Framework (SFRS)
v1.0.0**. Two co-equal readers — human (TTY) and agent (pipe / JSON) — both
first-class.

---

## Ground Rules for Agents

1. **Read the right doc first.** Code structure questions → `PRD.md` §2.1 + §5
   + §6. Sequencing questions → `PLAN.md`. "What's next?" → `TODO.md`. Never
   invent scope that isn't grounded in those three files.
2. **One sub-command per PR.** Mirroring [CONTRIBUTING.md](CONTRIBUTING.md).
3. **Never weaken safety for speed.** Steelbore Standard §3.1 cardinal rule.
   `unsafe` lives only in `caliper-anvil`, and every block needs a `// SAFETY:`
   comment plus Miri-clean test coverage.
4. **No auto-network.** PFA §6. The only network I/O in the entire codebase is
   `caliper models pull <name>`. Treat any other outbound socket as a bug.
5. **Tip-style hints, not prose.** Every error returns a `hint` field whose
   value is a *runnable command*, not a sentence. See `assets/error-hint-catalog.json`.
6. **JSON is contractual.** Output schema is stable from v1.0.0. Adding fields
   is fine; removing or renaming is breaking. Validate with `caliper schema`.
7. **Don't trap an agent in a TUI.** `AI_AGENT=1` / `AGENT=1` / `CI=true` /
   `TERM=dumb` → JSON only, no color, no `--format explore`, never wait on stdin.

---

## Repository Layout

```
.
├── PRD.md                  # Product requirements (v2.0)
├── PLAN.md                 # Implementation plan (v1.0)
├── TODO.md                 # Tickable backlog (v1.0)
├── README.md               # User-facing intro + project posture
├── NOTICE.md               # No-warranty / no-liability statement
├── CONTRIBUTING.md         # How to contribute
├── CREDITS.md              # Third-party prior art
├── AGENTS.md               # ← you are here
├── CLAUDE.md               # Claude-Code-specific overlay
├── SKILL.md                # Discoverable skill manifest
├── LICENSE                 # GPL-3.0-or-later text
├── Cargo.toml              # Workspace manifest
├── rust-toolchain.toml     # Toolchain pin (1.82.0)
├── rustfmt.toml, clippy.toml, deny.toml
├── flake.nix               # Reproducible Nix dev shell
├── .cargo/config.toml      # Cargo aliases + per-target overrides
├── .github/workflows/      # CI definitions
├── crates/                 # 14 workspace member crates
│   ├── caliper/            # Top-level binary (CLI + TUI dispatcher)
│   ├── caliper-trace/      # Public library API surface
│   ├── caliper-crucible/   # Pipeline orchestrator
│   ├── caliper-assayer/    # Adaptive router (the novel meta-algorithm)
│   ├── caliper-smelt/      # Color quantization
│   ├── caliper-etch/       # Edge detection
│   ├── caliper-burin/      # Contour tracing
│   ├── caliper-temper/     # Path fitting
│   ├── caliper-weld/       # Stitching / assembly
│   ├── caliper-cast/       # Output encoders (SVG/PDF/EPS/DXF)
│   ├── caliper-bellows/    # GPU compute backend (wgpu)
│   ├── caliper-touchstone/ # NPU/ML backend (ort + ONNX Runtime)
│   ├── caliper-anvil/      # SIMD kernels (only place `unsafe` is allowed)
│   └── caliper-tui/        # Ratatui interface
├── xtask/                  # Build / audit / housekeeping binary
├── assets/
│   ├── error-hint-catalog.json
│   ├── schemas/            # JSON Schema 2020-12 per sub-command
│   └── models/             # Touchstone ML model assets (pulled on demand)
└── tests/golden/           # End-to-end golden-file SVG corpus
```

---

## Where to Find Things When You Don't Know

| You want to …                                  | Look at …                                                          |
|------------------------------------------------|--------------------------------------------------------------------|
| Understand the seven-stage DAG                 | `PRD.md` §5 + §8                                                   |
| Pick a crate for new functionality             | `PRD.md` §2.1 (crate role table)                                   |
| Know which milestone something belongs to      | `TODO.md` (sections §1 through §9 map to v0.1.0 … v2.0.0)          |
| Add a new CLI flag                             | `PRD.md` §9.2 (global) + §9.3 (caliper-specific). Wire in `crates/caliper/src/cli.rs`. |
| Add a new sub-command                          | `PRD.md` §9.1. Plumb through `caliper schema <command>` and the canonical exit-code map (§9.6). |
| Add a new dependency                           | Update `[workspace.dependencies]` in root `Cargo.toml`. `cargo deny check` must stay green. |
| Add a new output format                        | New writer in `caliper-cast/src/<format>.rs`. Update `PRD.md` §13.2 if introducing a new format. |
| Add SIMD for a new ISA                         | New module in `caliper-anvil/src/<isa>.rs`. Add Miri coverage. Update build dispatch in `src/lib.rs`. |
| Add a new ML model                             | Update `caliper-touchstone/src/models.rs`. Add manifest entry with BLAKE3 hash. Never auto-download. |
| Investigate output-mode detection              | `PRD.md` §9.4 cascade. Implementation will land in `crates/caliper/src/env.rs`. |
| Find canonical exit codes                      | `PRD.md` §9.6. Implementation in `crates/caliper/src/error.rs`.    |

---

## How Each Command Should Feel to an Agent

- **Discoverable:** `caliper --help`, `caliper <verb> --help`, `caliper describe`,
  `caliper schema [<command>]` — these are how you learn the surface without
  guessing. Use them.
- **Predictable:** Same input + same flags → same `data` payload. The only
  intentional variability is `metadata.timestamp` and `metadata.hardware` (which
  depends on the runner).
- **Structured:** Pipe `--json` to `jaq` / `jq`. Every record is a single JSON
  document with `metadata` / `data` (success) or `error` (failure).
- **Runnable errors:** When you hit `exit_code != 0`, read `error.hint` — it is
  a literal command you can execute to recover or get more context.
- **Stream-safe:** Long-running batches emit `--format jsonl` with one record
  per line. No partial-record writes; no embedded newlines in field values.

---

## Environment Variables Caliper Watches

| Variable        | Effect                                                          |
|-----------------|------------------------------------------------------------------|
| `AI_AGENT=1`    | JSON output, no color, no TUI, `--yes` implicit, minimal stderr |
| `AGENT=1`       | Same as `AI_AGENT`                                               |
| `CI=true`       | JSON output, no color, no TUI, normal verbosity                  |
| `CLAUDECODE=1`  | Informational only — recorded in `metadata.invoking_agent`       |
| `CURSOR_AGENT=1`| Informational only                                               |
| `GEMINI_CLI=1`  | Informational only                                               |
| `TERM=dumb`     | No color, no TUI; do not infer agent intent                      |
| `NO_COLOR`      | No color (per `no-color.org` standard)                           |
| `FORCE_COLOR`   | Force color even when not detected                               |
| `CALIPER_CONFIG`| Override default config-file location                            |
| `CALIPER_CACHE` | Override default cache dir (Touchstone model store)              |

Set `AI_AGENT=1` when invoking Caliper from a tool-use loop. You will get
clean JSON on stdout, structured errors on stderr, and no interactive prompts.

---

## What NOT to Do

- **Don't add `unsafe` outside `caliper-anvil`.** The workspace lints forbid it.
- **Don't introduce telemetry, analytics, or any outbound HTTP/DNS** beyond
  `caliper models pull`.
- **Don't bypass `--dry-run`** on write paths. Every write operation must
  support `--dry-run` and the test suite asserts it.
- **Don't pretty-print JSON when stdout is non-TTY.** Use compact JSON.
- **Don't emit log records to stdout.** stdout is for data only. Logs go to
  stderr.
- **Don't catch panics across the FFI boundary in `caliper-anvil`.** Panics
  inside FFI boundaries are caught only in `crates/caliper-trace/src/ffi.rs`.

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)&gt;
Copyright (c) 2026 Mohamed Hammad &nbsp;|&nbsp; License: GPL-3.0-or-later
[https://Caliper.Steelbore.com/](https://Caliper.Steelbore.com/)
