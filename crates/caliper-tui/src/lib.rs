// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-tui
//!
//! Interactive TUI for Caliper (`--format explore`). Built on `ratatui` +
//! `crossterm`. Three-pane layout: source raster (left) / live vector preview
//! (right) / parameter sliders (bottom), with a status bar that shows shape
//! count, path length, max deviation, elapsed time, CPU/GPU/NPU utilisation,
//! active SIMD level, and the Assayer's current recommendation.
//!
//! ## Steelbore theme
//!
//! The TUI uses the six-token Steelbore palette with Void Navy `#000027` as
//! the mandatory background. WCAG 2.1 AA contrast verified for every
//! foreground / background pairing. Reduced-motion preference honoured.
//!
//! ## Keybindings
//!
//! Dual CUA + Vim mappings — both bound to every action. See
//! [PRD §10.2](https://Caliper.Steelbore.com/prd#102-dual-cua--vim-keybindings-standard-7).
//!
//! ## Agent suppression
//!
//! `--format explore` requires a TTY. When `AI_AGENT` / `AGENT` / `CI` /
//! `TERM=dumb` is set, Caliper falls back to JSON mode and emits a warning on
//! stderr rather than trapping the agent in a TUI loop.

#![forbid(unsafe_code)]

// Ratatui skeleton, palette, keymap, and live preview land with v0.2.0 §2.6
// of TODO.md.
