# Usage

Kenchiku finds scaffolds using the `KENCHIKU_PATH` env variable. Like normal `$PATH`, this looks like this:

```
/some/dir:/some/other/dir:/etc
```

So if you put your scaffolds at `~/.local/share/kenchiku/scaffolds`, add that to `KENCHIKU_PATH`.
Your scaffolds should then show up when running `kenchiku list`.

**Example**:

```sh
~/.local/share/kenchiku/scaffolds/
|- react-app/
|  |- scaffold.lua
|  |- package.json, etc.
```

## Construction 🚧

To construct a scaffold, simply run `kenchiku construct <scaffold>`.
The scaffold can ask you for values, but you can also specify them beforehand: `kenchiku construct <scaffold> -s a=b --set c=d`.

By default, many actions (like executing arbitrary commands) require your confirmation.
To disable (if you know your scaffolds/wrote them yourself), simply specify `-c` multiple times (every time will decrease the "severity" of confirmations).
`exec.run` for example requires confirmation level 2, so to allow this without any prompt, pass `-cc`.
If you completely don't care, just create an alias with a bunch of `c`'s ;)

## Patching ✏️

To run a patch, run the `patch` subcommand: `kenchiku patch <scaffold:patch>`.
Here, the scaffold name is followed by a `:`, then the name of the patch you want to run.

Values work the same, either pass them with `-s/--set` or get asked interactively.

## `scaffold.lua` Schema

```lua
---@type Scaffold
return {
  description = "A description of what this scaffold/the construct function does",
  construct = function() end,
  values = {
    value_name = {
      description = "Description",
      type = "string", -- or bool, enum, number
      default = "default", -- optional
      choices = {}, -- optional, for enum
    }
  },
  patches = { -- optional
    patch_name = {
      description = "A description of what this patch does",
      run = function() end,
      values = {}, -- Patches can also have values
    },
  },
}
```
