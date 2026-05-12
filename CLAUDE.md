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

## Before You Edit Any `.rs` File

Invoke the `rust-guidelines` skill. It is mandatory for every Rust touch, no
exceptions. Caliper's workspace `Cargo.toml` enforces the lint set, but the
skill's per-area guidance (universal, library, performance, safety, FFI, AI)
catches things the linter cannot.

For non-Rust work, the relevant skills are:

| You are about to …                          | Skill to invoke           |
|---------------------------------------------|---------------------------|
| Touch any `.rs` file                        | `rust-guidelines`         |
| Touch any CLI surface, `--json`, schema     | `steelbore-cli-standard`  |
| Touch agentic UX, AGENTS.md, MCP            | `steelbore-agentic-cli`   |
| Touch TUI, palette, typography              | `steelbore-standard` + `steelbore-theme-factory` |
| Touch documents (PRD/PLAN/TODO/README/docs) | `steelbore-document-format` |
| Run a shell command                         | `steelbore-cli-preference` + `steelbore-cli-shell` |
| Install missing software                    | `steelbore-missing-pkg`   |
| Generic Steelbore compliance question       | `steelbore-standard`      |

---

## Preferred Tools

- **File reads** → `Read`. Never `cat`.
- **File edits** → `Edit` for diffs, `Write` for new files / full rewrites.
- **Search** → `rg` (ripgrep), `fd` for filenames. Never bare `grep` or `find`.
  See `steelbore-cli-preference` for the full mapping.
- **JSON inspection** → `jaq`, not `jq`.
- **Tests** → `cargo test --workspace`. For TUI snapshots: `cargo insta test`.
- **Lints** → `cargo clippy --workspace --all-targets -- -D warnings`.
- **Format** → `cargo fmt`.

---

## Output Style

- Match the [Steelbore Standard §12](https://Steelbore.com/standard) date/time
  rules in every commit message, log line, and PR description: ISO 8601 UTC
  with `Z` suffix. 24-hour time. Metric units.
- Steelbore palette in any colored output. Void Navy `#000027` is the canonical
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

---

## Where to Stash Notes Between Sessions

Use the conversation memory (`/home/mj/.claude/projects/-steelbore-caliper/memory/`).
Anything you learn about the user's workflow preferences, project context, or
non-obvious decisions belongs there — not in PR descriptions or comments.

The PRD / PLAN / TODO are the *project's* memory and are versioned in git.
Memory files are *agent's* memory and are session-local.

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)&gt;
Copyright (c) 2026 Mohamed Hammad &nbsp;|&nbsp; License: GPL-3.0-or-later
[https://Caliper.Steelbore.com/](https://Caliper.Steelbore.com/)
