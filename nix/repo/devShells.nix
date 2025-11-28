{inputs, cell, ...}: let
  inherit (inputs) pkgs devshell treefmt fenix soonix;
  inherit (cell) ci;
in {
  default = devshell.mkShell {
    imports = [soonix.devshellModule];
    packages = [
      pkgs.gcc
      pkgs.cargo-nextest
      fenix.minimal.toolchain
      fenix.rust-analyzer
      (treefmt.mkWrapper pkgs {
        projectRootFile = "flake.nix";
        programs = {
          alejandra.enable = true;
          mdformat.enable = true;
          rustfmt.enable = true;
        };
      })
    ];
    soonix.hooks.ci = ci.soonix;
  };
}
