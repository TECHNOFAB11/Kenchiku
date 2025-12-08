{inputs, ...}: let
  inherit (inputs) cilib fenix pkgs;
in
  cilib.mkCI {
    pipelines."default" = {
      stages = ["test" "build" "deploy"];
      jobs = {
        "test" = {
          stage = "test";
          nix.deps = with pkgs; [
            fenix.minimal.toolchain
            cargo-nextest
            gcc
          ];
          variables = {
            "LD_LIBRARY_PATH" = "${pkgs.stdenv.cc.cc.lib}/lib";
            "CARGO_HOME" = "\${CI_PROJECT_DIR}/.cargo";
          };
          cache = [
            {
              key = "rust-cache";
              paths = ["target/" ".cargo/bin" ".cargo/registry"];
            }
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
        "docs" = {
          stage = "build";
          script = [
            # sh
            ''
              nix build .#docs:default
              mkdir -p public
              cp -r result/. public/
            ''
          ];
          artifacts.paths = ["public"];
        };
        "pages" = {
          nix.enable = false;
          image = "alpine:latest";
          stage = "deploy";
          script = ["true"];
          artifacts.paths = ["public"];
          rules = [
            {
              "if" = "$CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH";
            }
          ];
        };
      };
    };
  }
