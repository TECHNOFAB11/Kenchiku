{
  inputs = {
    devshell-lib.url = "gitlab:rensa-nix/devshell?dir=lib";
    nix-gitlab-ci-lib.url = "gitlab:TECHNOFAB/nix-gitlab-ci/3.1.0?dir=lib";
    soonix-lib.url = "gitlab:TECHNOFAB/soonix?dir=lib";
    nixmkdocs-lib.url = "gitlab:TECHNOFAB/nixmkdocs?dir=lib";
    fenix = {
      url = "github:nix-community/fenix";
      flake = false;
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      flake = false;
    };
  };
  outputs = i:
    i
    // {
      devshell = i.devshell-lib.lib {inherit (i.parent) pkgs;};
      cilib = i.nix-gitlab-ci-lib.lib {inherit (i.parent) pkgs;};
      soonix = i.soonix-lib.lib {inherit (i.parent) pkgs;};
      doclib = i.nixmkdocs-lib.lib {inherit (i.parent) pkgs;};
      treefmt = import i.treefmt-nix;
      fenix = import i.fenix {inherit (i.parent) pkgs;};
    };
}
