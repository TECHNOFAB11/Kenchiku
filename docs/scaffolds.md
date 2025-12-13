# Creating Scaffolds

Kenchiku scaffolds are directories containing a `scaffold.lua` file and any other resources (templates, files) needed to
generate or patch a project.

## Scaffold Structure

A scaffold directory could look like this:

```
my-scaffold/
├── scaffold.lua
├── templates/
│   ├── main.rs.j2
│   └── Cargo.toml.j2
└── static/
    └── .gitignore
```

The only important file we need is the `scaffold.lua`. It makes this directory a scaffold,
the rest you can structure as you like.

## `scaffold.lua`

The `scaffold.lua` file is the entry point. It must return a Lua table with the following structure:

```lua
---@type Scaffold (useful if you use luals)
return {
    description = "A brief description of the scaffold",
    -- Define values to prompt the user for
    values = {
        project_name = {
            description = "The name of the project",
            type = "string",
        },
        database = {
            description = "Database type",
            type = "enum",
            choices = { "postgres", "sqlite", "none" },
            default = "none",
        }
    },
    -- The main function called when constructing the scaffold
    construct = function()
        local name = values.get("project_name") ---@type string
        local db = values.get("database") ---@type string

        fs.mkdir(name)

        -- Render a template
        local cargo_toml = tmpl.template_file("templates/Cargo.toml.j2", {
            name = name,
            database = db
        })
        fs.write(name .. "/Cargo.toml", cargo_toml)

        print("Scaffold constructed successfully!")
    end,
    -- Optional patches for existing projects
    patches = {
        add_logging = {
            description = "Adds logging to the project",
            values = {}, -- Patches can also have their own values
            run = function()
                if fs.exists("Cargo.toml") then
                    local content = fs.read("Cargo.toml", { source = "workdir" })
                    -- Add tracing dependency if not present
                    if not content:find("tracing") then
                        -- Just an example, don't do this, you cannot know if [dependencies] is the last entry etc.
                        local new_content = content .. '\ntracing = "0.1"\n'
                        fs.write("Cargo.toml", new_content)
                    end
                else
                    warn("Cargo.toml not found!")
                end
            end
        }
    }
}
```

## Lua API

Kenchiku exposes several modules to the Lua environment to help you interact with the file system, handle user input,
and process data.

See [Lua APIs](apis.md) for the full reference documentation.
