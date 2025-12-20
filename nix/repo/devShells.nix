{
  inputs,
  cell,
  ...
}: let
  inherit (inputs) pkgs devshell treefmt fenix devtools-lib;
  inherit (cell) soonix;
  treefmtWrapper = treefmt.mkWrapper pkgs {
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
  };
in {
  default = devshell.mkShell {
    imports = [soonix.devshellModule devtools-lib.devshellModule];
    packages = [
      pkgs.gcc
      pkgs.rust-analyzer
      pkgs.cargo-nextest
      pkgs.lua-language-server
      fenix.minimal.toolchain
      treefmtWrapper
    ];
    env = {
      PATH.prefix = "$REN_ROOT/target/debug";
      KENCHIKU_PATH.eval = "$REN_ROOT/scaffolds";
      LD_LIBRARY_PATH.value = "${pkgs.stdenv.cc.cc.lib}/lib";
    };
    lefthook.config = {
      "pre-commit" = {
        parallel = true;
        jobs = [
          {
            name = "treefmt";
            stage_fixed = true;
            run = "${treefmtWrapper}/bin/treefmt";
            env.TERM = "dumb";
          }
          {
            name = "soonix";
            stage_fixed = true;
            run = "${soonix.packages."soonix:update"}/bin/soonix:update";
          }
        ];
      };
    };
    cocogitto.config = {
      tag_prefix = "v";
      changelog = {
        path = "CHANGELOG.md";
        template = "remote";
        remote = "gitlab.com";
        repository = "kenchiku";
        owner = "TECHNOFAB";
      };
    };
  };
}
