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

---@class Scaffold
---@field description string? An optional description of the scaffold.
