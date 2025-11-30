local exists = fs.exists("scaffold.lua")

print("scaffold.lua exists? " .. tostring(exists))

local stdout = exec.run("echo hi!")
warn("Stdout: " .. stdout)

---@type Scaffold
return {
	description = "Example!",
	construct = function()
		print("constructing scaffold")
	end,
	patches = {
		example = {
			description = [[
        This patch does something.
      ]],
			run = function()
				print("running patch example")
			end,
		},
	},
}
