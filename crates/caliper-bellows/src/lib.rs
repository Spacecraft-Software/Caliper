// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-bellows
//!
//! GPU compute backend for Caliper. Fans the GPU fire — portable compute
//! shaders via `wgpu` (Vulkan, Metal, DX12, OpenGL, WebGPU) from a single
//! WGSL codebase. Eligible stages: AoS↔SoA transpose, denoising, OKLab
//! conversion, k-means quantisation, histogram, Sobel/Scharr, morphology,
//! and connected-component labelling for very large images.
//!
//! **Design constraint:** minimise PCIe traffic. Pixel data uploads once at
//! decode time; intermediate buffers stay GPU-resident; only quantised layer
//! masks return to host memory (typically 30–60× smaller than the input).
//!
//! See [PRD §6.2].
//!
//! [PRD §6.2]: https://Caliper.Steelbore.com/prd#62-gpu-backend--bellows-optional

#![forbid(unsafe_code)]

// Adapter enumeration, WGSL shaders, and GPU-resident buffer choreography land
// with v0.3.0 §3 of TODO.md.
