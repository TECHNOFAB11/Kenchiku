---@meta

---@class fs_global
---@field exists fun(path: string): boolean

---@type fs_global
fs = nil

---@class exec_global
---@field run fun(command: string): string | nil

---@type exec_global
exec = nil

---@param msg string
function warn(msg) end

---@class Patch
---@field description string Description of what the patch does.
---@field run fun() Function which executes the patch.

---@class Scaffold
---@field description string Description of what the scaffold does.
---@field construct fun() Function which executes the scaffold.
---@field patches table<string, Patch>? Patches this scaffold exposes.
