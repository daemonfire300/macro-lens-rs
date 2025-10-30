{
  description = "TODO";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {

        devShells.default = let
          stable-rust = pkgs.rust-bin.stable.latest;
          rust-core = stable-rust.default;
          stable-rust-analyzer = stable-rust.rust-analyzer;
        in with pkgs;
        mkShell {
          buildInputs =
            [ openssl pkg-config eza fd rust-core stable-rust-analyzer ];

          shellHook = ''
            echo "Entering system-specific shell <$SHELL>"
            #exec $SHELL
          '';
        };
      });
}
