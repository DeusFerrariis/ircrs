{
  description = "Development shell for Rust development.";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            openssl
            pkg-config
            eza
            gcc
            fd
            zellij
            rustup
            cargo
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              extensions = [ "rust-src" "rust-analyzer"];
            }))
          ];

          shellHook = ''
            alias ls=eza
            alias find=fd
            cp /run/current-system/sw/bin/rust-analyzer ~/.local/share/nvim/mason/bin/
            alias zj="zellij --layout .layout.kdl"
            rustup default nightly
          '';
        };
      }
    );
}
