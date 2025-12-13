{inputs, ...}: let
  inherit (inputs) parent pkgs fenix;
in rec {
  default = kenchiku;
  kenchiku =
    (pkgs.makeRustPlatform {
      inherit (fenix.minimal) cargo rustc;
    }).buildRustPackage {
      pname = "kenchiku";
      version = "latest";
      src = parent.self;
      cargoLock.lockFile = "${parent.self}/Cargo.lock";
      LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
    };
}
