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

		local contents = fs.read("hello.txt", { source = "workdir" })
		print("contents: " .. contents)

		local result = tmpl.patch("hello world", "hello", "konnichiwa")
		print("result: " .. result)
	end,
	values = {
		example = {
			description = "Some description",
			type = "string",
			default = "Example!",
		},
		test = {
			description = "Test",
			type = "enum",
			choices = { "option1", "option2" },
		},
	},
	patches = {
		example = {
			description = [[
        This patch does something.
      ]],
			run = function()
				print("running patch example")
			end,
			values = {
				test = {
					description = "some desc",
					type = "bool",
					default = false,
				},
			},
		},
	},
}
