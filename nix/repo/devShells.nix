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
    settings.formatter.mdformat = {
      command = let
        pkg = pkgs.python3.withPackages (p: [
          p.mdformat
          p.mdformat-mkdocs
        ]);
      in "${pkg}/bin/mdformat";
      excludes = ["CHANGELOG.md"];
    };
  };
in {
  default = devshell.mkShell {
    imports = [soonix.devshellModule devtools-lib.devshellModule];
    packages = [
      pkgs.gcc
      pkgs.rust-analyzer
      pkgs.cargo-nextest
      pkgs.cargo-edit
      pkgs.lua-language-server
      pkgs.complgen
      fenix.minimal.toolchain
      treefmtWrapper
    ];
    env = {
      PATH.prefix = "$REN_ROOT/target/debug";
      KENCHIKU_PATH.eval = "$REN_ROOT/scaffolds";
      LD_LIBRARY_PATH.value = "${pkgs.stdenv.cc.cc.lib}/lib";
    };
    task.",".tasks = {
      "update-completions" = {
        dir = "completions";
        cmd =
          # sh
          ''
            complgen --bash kenchiku.bash kenchiku.usage
            complgen --zsh kenchiku.zsh kenchiku.usage
            complgen --fish kenchiku.fish kenchiku.usage
          '';
      };
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
      ignore_merge_commits = true;
      pre_bump_hooks = [
        "cargo build --release"
        "cargo set-version {{version}}"
        "cargo check --release"
        "git add :/Cargo.lock"
      ];
      changelog = {
        authors = [
          {
            username = "TECHNOFAB";
            signature = "technofab";
          }
        ];
        path = "CHANGELOG.md";
        template = "remote";
        remote = "gitlab.com";
        repository = "kenchiku";
        owner = "TECHNOFAB";
      };
    };
  };
}
