{ example ? "1", ... }:

let
  moz_overlay = import (
    builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz
  );

  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustChannels.nightly.rust-std
    rustChannels.nightly.rust
    rustChannels.nightly.rustc
    rustChannels.nightly.cargo

    cmake
    gcc
    openssl
    pkgconfig
  ];

  LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
}

