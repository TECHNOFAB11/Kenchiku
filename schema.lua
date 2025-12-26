---@meta

---@class FsReadOpts
---@field source "scaffold"|"workdir" Where to read the file/path from.

---@class fs_global
---@field exists fun(path: string): boolean Checks if a file exists.
---@field read fun(path: string, opts?: FsReadOpts): string Reads the contents of a file.
---@field write fun(path: string, content: string) Writes content to a file.
---@field mkdir fun(path: string) Creates all directories up to path.

---@type fs_global
fs = nil

---@class ExecRunResult
---@field stdout string Stdout of the program.
---@field stderr string Stderr of the program.
---@field status_code integer Statuscode of the program.

---@class exec_global
---@field run fun(command: string): ExecRunResult Runs a command in the working dir.

---@type exec_global
exec = nil

---@class tmpl_global
---@field template fun(content: string, vars: table): string
---@field template_file fun(file: string, vars: table): string

---@type tmpl_global
tmpl = nil

---@class ReReplaceOpts
---@field limit number Limit to replace, default is 0.
---
---@class re_global
---@field match fun(content: string, pattern: string): table Regex matches a string, returning the captured groups.
---@field replace fun(content: string, pattern: string, replacement: string, opts: ReReplaceOpts?): string Regex replace in a string.

---@type re_global
re = nil

---@class values_global
---@field get fun(id: string): any Get a value, prompting the user if it isn't set.

---@type values_global
values = nil

---@param msg string Log a warning.
function warn(msg) end

---@class Value
---@field description string Description of value.
---@field type "string"|"number"|"enum"|"bool" Type of the value.
---@field default? string|number|boolean|nil Default value.
---@field choices? table<string> Enum values to select from.

---@class Patch
---@field description string Description of what the patch does.
---@field run fun() Function which executes the patch.
---@field values table<string, Value>? Values this patch requires.

---@class Scaffold
---@field description string Description of what the scaffold does.
---@field construct fun() Function which executes the scaffold.
---@field patches table<string, Patch>? Patches this scaffold exposes.
---@field values table<string, Value>? Values this scaffold requires.
