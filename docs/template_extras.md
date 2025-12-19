# Template Extras

The template functions in Lua already have some builtin extras, like filters or functions (from minijinja's builtins).
But since they are quite limited, Kenchiku extends them with the extras below.

## Filters

### `timeformat`

Formats the unix timestamp it receives with the format you specify.

**Example**

```jinja
{{ now() | timeformat("%Y") }}
```

## Functions

### `now`

Returns the current unix timestamp (seconds since epoch).

**Example**

```jinja
{{ now() }}
```

### `panic`

Aborts the program with the specified error.

**Example**

```lua
{{ panic("some error here") }}
```
