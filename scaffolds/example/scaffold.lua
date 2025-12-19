local module = require("./module")

---@type Scaffold
return {
	description = "Example!",
	construct = function()
		print("constructing scaffold")
		print("module says hello " .. module.hello)
		local exists = fs.exists("scaffold.lua")

		print("scaffold.lua exists? " .. tostring(exists))

		local res = exec.run("pwd")
		warn("PWD: " .. res.stdout)

		fs.write("hello.txt", "hello world!")

		local contents = fs.read("hello.txt", { source = "workdir" })
		print("contents: " .. contents)

		local result = tmpl.patch("hello world", "hello", "konnichiwa")
		print("result: " .. result)

		local value = values.get("example") ---@type string
		print("Value is: " .. tostring(value))

		local boolean = values.get("boolean") ---@type boolean
		print("Bool is: " .. tostring(boolean))

		local enum = values.get("enum") ---@type string
		print("Enum is: " .. tostring(enum))

		local template = [[
Hello world! {{ a }}
]]
		local templated = tmpl.template(template, { a = "b" })
		print("Templated: " .. templated)

		local templated_file = tmpl.template_file("example.txt.tmpl", { greeting = "Hello" })
		print("Templated file: " .. templated_file)

		-- local invalid = values.get("invalid")
		-- print("Should not reach this")
	end,
	values = {
		example = {
			description = "Some description",
			type = "string",
			default = "Example!",
		},
		enum = {
			description = "Test",
			type = "enum",
			choices = { "option1", "option2" },
			default = "option2",
		},
		boolean = {
			description = "Some bool",
			type = "bool",
			default = true,
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
