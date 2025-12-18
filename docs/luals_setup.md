# LuaLS Setup

## Manually

If you use `lua-language-server`, download `schema.lua` from the repo and create
a `.luarc.json` like this to benefit from autocompletion. Unfortunately there is no easier
solution since `luals` doesn't support loading libraries from URLs.

```json title=".luarc.json"
{
  "$schema": "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json",
  "workspace": {
    "library": [
      "/path/to/schema.lua"
    ]
  }
}
```

## Using Soonix

Alternatively, if you use [Soonix](https://soonix.projects.tf), you can do the following to automatically get the correct `schema.lua` added to your `.luarc.json`:

```nix
soonix.hooks.luarc = {
  output = ".luarc.json";
  data = {
    "$schema" = "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json";
    workspace.library = [
      "${kenchiku.packages.default}/share/kenchiku/schema.lua"
    ];
  };
  hook.mode = "copy";
  opts.format = "json";
};
```

Now when running Soonix (manually or using the [devshell](https://devshell.rensa.projects.tf) integration), it will generate a local `.luarc.json`, which is gitignored (since the Nix store path might not be the same across platforms).
