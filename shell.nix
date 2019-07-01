let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  rust_replace = self: super: {
    rust = let
        rust = (super.rustChannelOf { channel = "1.35.0"; }).rust;
      in { rustc = rust; cargo = rust; };
    inherit (self.rust) rustc cargo;
  };
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay rust_replace ]; };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "plcrash.rs";
    buildInputs = [
      rustc cargo
      pkgconfig openssl protobuf
      darwin.apple_sdk.frameworks.Security
    ];
  }
