---@type Scaffold
return {
	description = "Example!",
	construct = function()
		print("constructing scaffold")
		local exists = fs.exists("scaffold.lua")

		print("scaffold.lua exists? " .. tostring(exists))

		local stdout = exec.run("pwd")
		warn("PWD: " .. stdout)
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
