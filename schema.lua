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

---@class exec_global
---@field run fun(command: string): string Runs a command in the working dir.

---@type exec_global
exec = nil

---@class tmpl_global
---@field patch fun(content: string, pattern: string, replacement: string, opts: table?): string Patch a string, replacing the source with target.

---@type tmpl_global
tmpl = nil

---@param msg string Log a warning.
function warn(msg) end

---@class Patch
---@field description string Description of what the patch does.
---@field run fun() Function which executes the patch.

---@class Scaffold
---@field description string Description of what the scaffold does.
---@field construct fun() Function which executes the scaffold.
---@field patches table<string, Patch>? Patches this scaffold exposes.
