// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # xtask
//!
//! Cross-cutting audit and housekeeping tasks for the Caliper workspace.
//! Implements the BLOCKER/CRITICAL gates from PLAN.md §6 that can be checked
//! mechanically — SPDX headers, UTF-8 encoding, network-I/O audit,
//! cross-shell smoke tests.
//!
//! Invoke via the cargo alias declared in `.cargo/config.toml`:
//!
//! ```sh
//! cargo xtask spdx-audit
//! cargo xtask utf8-audit
//! cargo xtask network-audit
//! cargo xtask cross-shell-test
//! ```

#![forbid(unsafe_code)]

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Cross-cutting audit and housekeeping tasks for the Caliper workspace.
#[derive(Parser, Debug)]
#[command(name = "xtask", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Verify every source file carries a GPL-3.0-or-later SPDX header.
    SpdxAudit,
    /// Verify every text file is UTF-8 without a byte-order mark.
    Utf8Audit,
    /// Verify no source file performs network I/O outside `caliper models pull`.
    NetworkAudit,
    /// Smoke-test the Caliper CLI under POSIX sh, Bash, Brush, Nushell, PowerShell, Ion.
    CrossShellTest,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xtask: {err:#}");
            ExitCode::FAILURE
        },
    }
}

fn run(cli: &Cli) -> Result<()> {
    match cli.command {
        Cmd::SpdxAudit => spdx_audit(),
        Cmd::Utf8Audit => utf8_audit(),
        Cmd::NetworkAudit => network_audit(),
        Cmd::CrossShellTest => cross_shell_test(),
    }
}

fn spdx_audit() -> Result<()> {
    // Implementation lands with v0.1.0 §0 of TODO.md (workspace bootstrap).
    // Walks all tracked source files and asserts SPDX-License-Identifier
    // header is the first or second line. Exit codes per PRD §9.6:
    // - 0 on clean audit
    // - 1 on any missing/incorrect header
    eprintln!("xtask spdx-audit: not yet implemented");
    Ok(())
}

fn utf8_audit() -> Result<()> {
    // Walks all tracked text files, asserts UTF-8 decode succeeds, and
    // asserts the file does not begin with EF BB BF (UTF-8 BOM).
    eprintln!("xtask utf8-audit: not yet implemented");
    Ok(())
}

fn network_audit() -> Result<()> {
    // Greps for forbidden patterns (`reqwest`, `ureq`, `hyper::Client`, raw
    // socket calls) outside `crates/caliper/src/models.rs`. PFA §6 — only
    // `caliper models pull` may touch the network.
    eprintln!("xtask network-audit: not yet implemented");
    Ok(())
}

fn cross_shell_test() -> Result<()> {
    // Runs a curated set of CLI invocations under POSIX sh, Bash 5+, Brush,
    // Nushell 0.111+, PowerShell 7.6+, and Ion. Asserts identical stdout
    // bytes for `--json` invocations across all shells. See TODO.md §6.4.
    eprintln!("xtask cross-shell-test: not yet implemented");
    Ok(())
}
