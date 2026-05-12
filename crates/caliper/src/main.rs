// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper — binary entry point
//!
//! Top-level binary for Caliper. Dispatches to:
//!
//! - CLI sub-commands (`trace`, `batch`, `inspect`, `preview`, `benchmark`,
//!   `palette`, `config`, `models`, `schema`, `describe`) — see [PRD §9.1].
//! - Interactive TUI (`--format explore`) — see [PRD §10].
//! - MCP server (`caliper mcp`) — see [PRD §9.9].
//!
//! Output-mode detection cascade (PRD §9.4) decides between human (TTY,
//! colour, tables) and machine (JSON, compact, no colour) per invocation.
//!
//! [PRD §9.1]: https://Caliper.Steelbore.com/prd#91-noun-verb-command-structure
//! [PRD §10]: https://Caliper.Steelbore.com/prd#10-tui-interface---format-explore
//! [PRD §9.9]: https://Caliper.Steelbore.com/prd#99-mcp-server-surface-caliper-mcp

#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    // CLI scaffolding lands with v0.1.0 §1.11 of TODO.md. Stub returns
    // exit code 1 (general failure) per PRD §9.6 so accidental invocation
    // of the pre-implementation binary surfaces a clear non-zero status.
    eprintln!(
        "caliper: pre-v0.1.0 — CLI not yet implemented. See TODO.md and PRD.md \
         at https://Caliper.Steelbore.com/."
    );
    std::process::ExitCode::from(1)
}
