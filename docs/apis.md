# Lua APIs

Kenchiku provides a bunch of Lua globals to scaffolds to do anything useful.

!!! note

    These are only defined when running the `construct` or `run` functions, otherwise
    fetching the metadata of Scaffolds could

    1. allow code execution
    1. annoy everyone with prompts

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

## General

### `warn(message)`

Creates a warning log like this:

```
<date>  WARN kenchiku_lua::log: Warning from scaffold: <message>
```

### `print(message)`

Prints message to stdout.

## `fs` Module

### `fs.exists(path)`

Checks whether a file/path exists.

**Example**

```lua
fs.exists("example.txt")
```

### `fs.mkdir(path)`

Creates all directories up to and including `path`.

**Example**

```lua
fs.mkdir("example/directory/here")
```

### `fs.read(path, opts?)`

Reads the content of a file at `path`. Use `opts` to specify whether the file should
be read from the working directory or scaffold directory:

**Example**

```lua
fs.read("example.txt", { source = "workdir" })
fs.read("example.txt", { source = "scaffold" })
```

### `fs.write(path, content)`

Writes a file to `path` containing `content`.

**Example**

```lua
fs.write("example.txt", "hello world!")
```

## `tmpl` Module

### `tmpl.patch(content, pattern, replacement)`

Patch the content by replacing pattern with the replacement.
Basically a regex replace, so `pattern` can contain capture groups and
`replacement` can refer to them using `$1` for example.

**Example**

```lua
tmpl.patch("hello world", "w+", "konnichiwa")
```

## `exec` Module

### `exec.run(command)`

Run a command using `sh -c`. Returns stdout as a string.

**Example**

```lua
exec.run("pwd")
```

## `json` Module

### `json.encode(data any)`

Encodes `data` to a json string.

**Example**

```lua
json.encode({ hello = "world" })
```

### `json.decode(data string)`

Decodes `data` json string to a lua value.

**Example**

```lua
json.decode('{"hello": "world"}')
```
