// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Mohamed Hammad

//! Runtime CPU feature detection.
//!
//! This module is the dispatch sentinel for [`crate::nearest_centroid`] and
//! every other anvil entry point that has a SIMD path. It memoises a single
//! [`Cpu`] value at startup so feature-flag queries are sub-nanosecond on the
//! hot path.

use std::sync::OnceLock;

/// Detected CPU capabilities relevant to anvil's kernels.
///
/// Construct via [`Cpu::detect`]; the result is cached after first call.
/// The bit set surfaces on the JSON envelope as `metadata.hardware.cpu.simd`
/// (PRD §9.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cpu {
    sse2: bool,
    sse41: bool,
    avx2: bool,
    avx512f: bool,
    neon: bool,
    sve2: bool,
}

impl Cpu {
    /// Detect available CPU features. Result is memoised on first call.
    #[must_use]
    pub fn detect() -> Self {
        static CACHE: OnceLock<Cpu> = OnceLock::new();
        *CACHE.get_or_init(Self::detect_uncached)
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    fn detect_uncached() -> Self {
        Self {
            sse2: is_x86_feature_detected!("sse2"),
            sse41: is_x86_feature_detected!("sse4.1"),
            avx2: is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma"),
            avx512f: is_x86_feature_detected!("avx512f"),
            neon: false,
            sve2: false,
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_uncached() -> Self {
        Self {
            sse2: false,
            sse41: false,
            avx2: false,
            avx512f: false,
            // NEON is mandatory in AArch64 baseline; report it unconditionally.
            neon: true,
            sve2: std::arch::is_aarch64_feature_detected!("sve2"),
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    fn detect_uncached() -> Self {
        Self::default()
    }

    /// `true` if SSE2 is available (x86_64 baseline, always true on `x86_64-*`).
    #[must_use]
    pub fn has_sse2(&self) -> bool {
        self.sse2
    }

    /// `true` if SSE4.1 is available.
    #[must_use]
    pub fn has_sse41(&self) -> bool {
        self.sse41
    }

    /// `true` if AVX2 + FMA3 are both available.
    ///
    /// We require both because anvil's AVX2 kernels use FMA for fused-multiply-add.
    /// Every CPU that ships AVX2 also ships FMA3, but the runtime detection
    /// is explicit for safety.
    #[must_use]
    pub fn has_avx2(&self) -> bool {
        self.avx2
    }

    /// `true` if AVX-512F is available.
    #[must_use]
    pub fn has_avx512f(&self) -> bool {
        self.avx512f
    }

    /// `true` if NEON is available (always true on aarch64).
    #[must_use]
    pub fn has_neon(&self) -> bool {
        self.neon
    }

    /// `true` if SVE2 is available.
    #[must_use]
    pub fn has_sve2(&self) -> bool {
        self.sve2
    }

    /// Human-readable label for the highest-level kernel that will be selected.
    ///
    /// Surfaces in `--json` output as `metadata.hardware.cpu.simd` (PRD §9.5).
    #[must_use]
    pub fn dispatch_label(&self) -> &'static str {
        if self.avx512f {
            "avx512"
        } else if self.avx2 {
            "avx2"
        } else if self.sse41 {
            "sse4.1"
        } else if self.sse2 {
            "sse2"
        } else if self.sve2 {
            "sve2"
        } else if self.neon {
            "neon"
        } else {
            "scalar"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_is_cacheable() {
        let a = Cpu::detect();
        let b = Cpu::detect();
        assert_eq!(a, b);
    }

    #[test]
    fn dispatch_label_is_non_empty() {
        let cpu = Cpu::detect();
        assert!(!cpu.dispatch_label().is_empty());
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    #[test]
    fn x86_64_baseline_has_sse2() {
        // SSE2 is mandatory in the x86_64 ABI.
        let cpu = Cpu::detect();
        assert!(cpu.has_sse2());
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn aarch64_baseline_has_neon() {
        let cpu = Cpu::detect();
        assert!(cpu.has_neon());
    }
}
