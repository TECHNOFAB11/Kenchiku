{
  inputs = {
    devshell-lib.url = "gitlab:rensa-nix/devshell?dir=lib";
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
      treefmt = import i.treefmt-nix;
      fenix = import i.fenix {inherit (i.parent) pkgs;};
    };
}
