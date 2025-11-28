{inputs, ...}: let
  inherit (inputs) cilib fenix pkgs;
in
  cilib.mkCI {
    pipelines."default" = {
      stages = ["test"];
      jobs = {
        "test" = {
          stage = "test";
          nix.deps = with pkgs; [
            fenix.minimal.toolchain
            cargo-nextest
            gcc
          ];
          script = [
            "cargo nextest run --profile ci"
          ];
          allow_failure = true;
          artifacts = {
            when = "always";
            reports.junit = "target/nextest/ci/junit.xml";
          };
        };
      };
    };
  }
