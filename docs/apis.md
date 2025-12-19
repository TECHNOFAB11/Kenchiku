# Lua APIs

Kenchiku provides a bunch of Lua globals to scaffolds to do anything useful.

!!! note

    These are only defined when running the `construct` or `run` functions, otherwise
    fetching the metadata of Scaffolds could

    1. allow code execution
    1. annoy everyone with prompts

See [LuaLS Setup](./luals_setup.md) for setting up the **lua**-**L**anguage-**S**erver.

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

### `tmpl.template(template_string, vars)`

Renders a [MiniJinja](https://github.com/mitsuhiko/minijinja) template string with the given variables.
See [Template Extras](./template_extras.md) for more filters & functions.

**Example**

```lua
tmpl.template("Hello {{ name }}!", { name = "World" })
```

### `tmpl.template_file(file_path, vars)`

Reads a file from the scaffold directory and renders it as a template.
See [Template Extras](./template_extras.md) for more filters & functions.

**Example**

```lua
tmpl.template_file("templates/main.rs.j2", { name = "my_project" })
```

## `values` Module

### `values.get(name)`

Retrieves the value for the given name. If the value wasn't provided via CLI flags, Kenchiku will interactively prompt the user based on the value definition in `scaffold.lua`.

**Example**

```lua
local name = values.get("project_name")
```

## `exec` Module

### `exec.run(command)`

_Confirmation Level_: **2**

Run a command using `sh -c`. Returns a table with:

- `stdout` (string): Standard output
- `stderr` (string): Standard error
- `exit_code` (integer): Exit code of the command

**Example**

```lua
local result = exec.run("pwd")
print(result.stdout)
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
