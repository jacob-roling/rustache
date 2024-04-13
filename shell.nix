let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustVersion = "latest";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
  };
in
pkgs.mkShell {
  buildInputs = [
    rust
  ] ++ (with pkgs; [
    pkg-config
  ]);

  RUST_BACKTRACE = 1;

  # shellHook = with pkgs; ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath [
  # ]}"'';
}
