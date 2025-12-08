{
  inputs,
  cell,
  ...
}: let
  inherit (inputs) pkgs devshell treefmt fenix soonix;
  inherit (cell) ci;
in {
  default = devshell.mkShell {
    imports = [soonix.devshellModule];
    packages = [
      pkgs.gcc
      pkgs.rust-analyzer
      pkgs.cargo-nextest
      pkgs.lua-language-server
      fenix.minimal.toolchain
      (treefmt.mkWrapper pkgs {
        projectRootFile = "flake.nix";
        programs = {
          alejandra.enable = true;
          mdformat.enable = true;
          rustfmt.enable = true;
          stylua.enable = true;
        };
        settings.formatter.mdformat.command = let
          pkg = pkgs.python3.withPackages (p: [
            p.mdformat
            p.mdformat-mkdocs
          ]);
        in "${pkg}/bin/mdformat";
      })
    ];
    soonix.hooks.ci = ci.soonix;
    env = {
      KENCHIKU_PATH.eval = "$REN_ROOT/scaffolds";
      LD_LIBRARY_PATH.value = "${pkgs.stdenv.cc.cc.lib}/lib";
    };
  };
}
