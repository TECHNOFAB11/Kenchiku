---@type Scaffold
return {
	description = "Example!",
	construct = function()
		print("constructing scaffold")
		local exists = fs.exists("scaffold.lua")

		print("scaffold.lua exists? " .. tostring(exists))

		local stdout = exec.run("pwd")
		warn("PWD: " .. stdout)

		fs.write("hello.txt", "hello world!")

		local contents = fs.read("hello.txt")
		print("contents: " .. contents)
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
