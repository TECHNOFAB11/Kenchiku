{inputs, ...}: let
  inherit (inputs) pkgs devshell treefmt fenix;
in {
  default = devshell.mkShell {
    packages = [
      pkgs.gcc
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
  };
}
