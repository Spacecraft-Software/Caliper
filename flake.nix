# SPDX-License-Identifier: GPL-3.0-or-later
#
# Caliper — reproducible dev shell.
#
# Usage:
#   nix develop          # drops you into a shell with the toolchain pinned
#   nix flake update     # refresh inputs
#
# This flake intentionally does NOT build the binary — that lives in cargo.
# It provisions the *environment* (Rust toolchain, system libs for wgpu, ort,
# image decoders) so cargo can do its job.

{
  description = "Caliper — Rust-native raster-to-vector tracing engine.";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pin to the toolchain declared in rust-toolchain.toml.
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # System libs needed by wgpu (Bellows) and the image decoder set.
        runtimeLibs = with pkgs; [
          vulkan-loader
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          wayland
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          name = "caliper-dev";

          buildInputs = with pkgs; [
            rust

            # Cargo tooling
            cargo-deny
            cargo-fuzz
            cargo-nextest
            cargo-udeps
            cargo-vet
            cargo-watch

            # Native deps for image / pdf
            pkg-config
            cmake

            # CLI tools we prefer (steelbore-cli-preference)
            ripgrep
            fd
            jaq
            bat
            eza
            xh

            # Validators referenced in PLAN.md §4 exit-criteria
            qpdf
            ghostscript

            # Shell roster for cross-shell tests (TODO.md §6.4)
            nushell
            powershell
          ] ++ runtimeLibs;

          # Make wgpu / Vulkan / Wayland findable at runtime.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

          shellHook = ''
            echo "Caliper dev shell — toolchain: $(rustc --version)"
            echo "Run 'cargo build --workspace' to verify the bootstrap."
          '';
        };

        # Stub package — actual release builds come from `cargo build` driven
        # by .github/workflows/release.yml; v1.0.0 §6.6 of TODO.md installs the
        # NixOS module under github:Steelbore/Bravais.
        packages = { };
      });
}
