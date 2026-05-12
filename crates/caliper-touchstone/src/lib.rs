// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad
//
//! # caliper-touchstone
//!
//! NPU / ML backend for Caliper. Tests inputs against learned patterns —
//! ONNX Runtime via the `ort` crate, with execution-provider auto-selection
//! across TensorRT / CUDA / OpenVINO / VitisAI / QNN / CoreML / DirectML /
//! CPU.
//!
//! ## ML-augmented pathways
//!
//! 1. **ML edge detection** (HED / BDCN / EDTER-distilled INT8).
//! 2. **Semantic segmentation** for region-aware routing (v1.1.0+).
//! 3. **Super-resolution preprocessing** via Real-ESRGAN / SwinIR (v1.1.0+).
//! 4. **Shape classification** for primitive-fitting assistance (v1.2.0+).
//!
//! ## Privacy
//!
//! Per [PFA §6](https://Steelbore.com/standard), Touchstone performs **no
//! automatic downloads**. ML model assets are pulled only via the explicit
//! `caliper models pull <name>` CLI command, validated by BLAKE3 hash against
//! a signed manifest.
//!
//! See [PRD §6.3].
//!
//! [PRD §6.3]: https://Caliper.Steelbore.com/prd#63-npu-backend--touchstone-optional

#![forbid(unsafe_code)]

// Provider selection, ML edge models, model manager, and shape classifier land
// with v0.4.0 §4 of TODO.md.
