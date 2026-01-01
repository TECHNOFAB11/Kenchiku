{
  inputs,
  cell,
  ...
}: let
  inherit (inputs) soonix;
  inherit (cell) ci;
in
  (soonix.make {
    hooks = {
      ci = ci.soonix;
      renovate = {
        output = ".gitlab/renovate.json5";
        data = {
          extends = ["config:recommended"];
          postUpgradeTasks.commands = ["nix-portable nix run .#soonix:update"];
          lockFileMaintenance = {
            enabled = true;
            extends = ["schedule:monthly"];
          };
          nix.enabled = true;
          gitlabci.enabled = false;
          packageRules = [
            {
              description = "Only split Nix lockfile management into separate branches";
              matchManagers = ["nix"];
              matchUpdateTypes = ["lockFileMaintenance"];
              branchTopic = "lock-file-maintenance-{{packageFile}}";
              commitMessageExtra = "({{packageFile}})";
            }
          ];
        };
        hook = {
          mode = "copy";
          gitignore = false;
        };
        opts.format = "json";
      };
    };
  }).config
