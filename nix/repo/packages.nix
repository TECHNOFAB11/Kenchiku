{inputs, ...}: let
  inherit (inputs) parent pkgs fenix;
in rec {
  default = kenchiku;
  kenchiku =
    (pkgs.makeRustPlatform {
      inherit (fenix.minimal) cargo rustc;
    }).buildRustPackage rec {
      pname = "kenchiku";
      version = "latest";
      src = parent.self;
      cargoLock.lockFile = "${src}/Cargo.lock";
      LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";

      postInstall = ''
        mkdir -p $out/share/kenchiku
        cp $src/schema.lua $out/share/kenchiku/schema.lua
      '';
    };
}
