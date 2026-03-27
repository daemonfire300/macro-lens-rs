{
  description = "Crate packaging and development environment for macro-lens";

  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, crane, flake-utils, nixpkgs, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        toolchain = pkgs.rust-bin.stable."1.85.1".default.override {
          extensions = [ "clippy" "rust-analyzer" "rust-src" "rustfmt" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || pkgs.lib.hasSuffix ".stderr" path;
        };
        commonArgs = {
          inherit src;
          cargoVendorDir = craneLib.vendorCargoDeps {
            cargoLock = ./Cargo.lock;
            src = ./.;
          };
          pname = "macro-lens-workspace";
          version = "2.0.0";
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          cargoExtraArgs = "--workspace";
        });

        mkCrate = packageName: craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--package ${packageName}";
          pname = packageName;
        });
        ciApp = pkgs.writeShellApplication {
          name = "ci";
          runtimeInputs = [ pkgs.nix ];
          text = ''
            set -euo pipefail
            exec nix flake check --print-build-logs "$@"
          '';
        };
      in {
        apps = {
          ci = {
            type = "app";
            program = "${ciApp}/bin/ci";
            meta.description = "Run the canonical Nix CI checks locally";
          };
          default = self.apps.${system}.ci;
        };

        packages = {
          default = mkCrate "macro-lens";
          macro-lens = mkCrate "macro-lens";
          macro-lens-derive = mkCrate "macro-lens-derive";
          macro-lens-macros = mkCrate "macro-lens-macros";
        };

        checks = {
          inherit (self.packages.${system})
            macro-lens
            macro-lens-derive
            macro-lens-macros;

          cargo-fmt = craneLib.cargoFmt {
            inherit src;
          };

          cargo-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace --all-targets --all-features -- -D warnings";
          });

          cargo-test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--workspace";
          });

          cargo-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--workspace --no-deps";
          });
        };

        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            pkgs.cargo-expand
            pkgs.eza
            pkgs.fd
          ];

          shellHook = ''
            export RUST_SRC_PATH="${toolchain}/lib/rustlib/src/rust/library"
          '';
        };
      });
}
