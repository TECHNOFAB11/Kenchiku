use eyre::{Context as _, Result};
use kenchiku_common::{Context, IntoLuaErrDebug, minijinja_extras};
use minijinja::Environment;
use mlua::{ExternalResult, Lua};
use regex::Regex;
use std::fs;

use crate::fs::normalize_path;

pub struct LuaTmpl;

impl LuaTmpl {
    pub fn register(lua: &Lua, context: Context) -> Result<()> {
        let tmpl_table = lua.create_table()?;

        tmpl_table.set(
            "patch",
            lua.create_function(
                move |_lua,
                      (content, pattern, replacement, _opts): (
                    String,
                    String,
                    String,
                    Option<mlua::Table>,
                )| {
                    // TODO: options like replace all, replace first, etc.
                    let re = Regex::new(&pattern)
                        .wrap_err(format!("Invalid regex pattern '{}'", pattern))
                        .into_lua_err_debug()?;
                    let modified_content = re.replace_all(&content, replacement).to_string();
                    Ok(modified_content)
                },
            )?,
        )?;

        tmpl_table.set(
            "template",
            lua.create_function(move |_lua, (template, vars): (String, mlua::Table)| {
                let mut env = Environment::new();
                env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
                env = minijinja_extras::register(env);
                env.add_template("inline", &template).into_lua_err()?;
                let template = env.get_template("inline").into_lua_err()?;
                Ok(template.render(vars).into_lua_err()?)
            })?,
        )?;

        tmpl_table.set(
            "template_file",
            lua.create_function(move |_lua, (file, vars): (String, mlua::Table)| {
                let mut env = Environment::new();
                env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
                env = minijinja_extras::register(env);
                let context = context.clone();
                env.set_loader(move |path| {
                    let path = normalize_path(&context.scaffold_dir, path.to_string());
                    match fs::read_to_string(path) {
                        Ok(result) => Ok(Some(result)),
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
                        Err(err) => Err(minijinja::Error::new(
                            minijinja::ErrorKind::InvalidOperation,
                            "could not read template",
                        )
                        .with_source(err)),
                    }
                });
                let template = env.get_template(&file).into_lua_err()?;
                Ok(template.render(vars).into_lua_err()?)
            })?,
        )?;

        lua.globals().set("tmpl", tmpl_table)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_lua_tmpl_patch() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context {
            ..Default::default()
        };
        LuaTmpl::register(&lua, context)?;

        let execute_lua = |script: &str| -> eyre::Result<()> {
            lua.load(script).exec()?;
            Ok(())
        };

        let test_cases = vec![
            (
                "simple replacement",
                r#"
                    local result = tmpl.patch("hello world", "world", "universe")
                    print(result)
                    assert(result == "hello universe")
                "#,
            ),
            (
                "regex replacement",
                r#"
                    local result = tmpl.patch("hello world", "[[:word:]]+", "universe")
                    print(result)
                    assert(result == "universe universe")
                "#,
            ),
            (
                "no match",
                r#"
                    local result = tmpl.patch("hello world", "nomatch", "universe")
                    print(result)
                    assert(result == "hello world")
                "#,
            ),
            (
                "capture groups",
                r#"
                    local result = tmpl.patch("hello world", "h(ello) world", "H$1 WORLD")
                    print(result)
                    assert(result == "Hello WORLD")
                "#,
            ),
            (
                "unicode replacement",
                r#"
                    local result = tmpl.patch("你好世界", "世界", "世界你好")
                    print(result)
                    assert(result == "你好世界你好")
                "#,
            ),
        ];

        for (name, script) in test_cases {
            println!("Running test case: {}", name);
            execute_lua(script)?;
        }

        // Test error handling
        let error_cases = vec![(
            "invalid regex",
            r#"
                    tmpl.patch("hello world", "[", "universe")
                "#,
            "Invalid regex pattern '['",
        )];

        for (name, script, error_message) in error_cases {
            println!("Running error case: {}", name);
            let result = execute_lua(script);
            assert!(result.is_err());
            let err = result.unwrap_err();
            let err_string = format!("{:?}", err);
            assert!(
                err_string.contains(error_message),
                "Expected error message to contain '{}', but got '{}'",
                error_message,
                err_string
            );
        }

        Ok(())
    }

    #[test]
    fn test_lua_tmpl_template() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context {
            ..Default::default()
        };
        LuaTmpl::register(&lua, context)?;

        let execute_lua = |script: &str| -> eyre::Result<()> {
            lua.load(script).exec()?;
            Ok(())
        };

        let test_cases = vec![
            (
                "simple replacement",
                r#"
                    local result = tmpl.patch("hello world", "world", "universe")
                    print(result)
                    assert(result == "hello universe")
                "#,
            ),
            (
                "regex replacement",
                r#"
                    local result = tmpl.patch("hello world", "[[:word:]]+", "universe")
                    print(result)
                    assert(result == "universe universe")
                "#,
            ),
            (
                "no match",
                r#"
                    local result = tmpl.patch("hello world", "nomatch", "universe")
                    print(result)
                    assert(result == "hello world")
                "#,
            ),
            (
                "capture groups",
                r#"
                    local result = tmpl.patch("hello world", "h(ello) world", "H$1 WORLD")
                    print(result)
                    assert(result == "Hello WORLD")
                "#,
            ),
            (
                "unicode replacement",
                r#"
                    local result = tmpl.patch("你好世界", "世界", "世界你好")
                    print(result)
                    assert(result == "你好世界你好")
                "#,
            ),
            (
                "template simple variable",
                r#"
                    local result = tmpl.template("Hello {{ name }}!", { name = "World" })
                    print(result)
                    assert(result == "Hello World!")
                "#,
            ),
            (
                "template multiple variables",
                r#"
                    local result = tmpl.template("{{ greeting }} {{ name }}!", {
                        greeting = "Hello",
                        name = "World"
                    })
                    print(result)
                    assert(result == "Hello World!")
                "#,
            ),
            (
                "template with condition",
                r#"
                    local result = tmpl.template(
                        "{% if show %}Hello World!{% else %}Goodbye!{% endif %}",
                        { show = true }
                    )
                    print(result)
                    assert(result == "Hello World!")
                "#,
            ),
            (
                "template with loop",
                r#"
                    local result = tmpl.template(
                        "{% for item in items %}{{ item }}{% endfor %}",
                        { items = { "a", "b", "c" } }
                    )
                    print(result)
                    assert(result == "abc")
                "#,
            ),
            (
                "template with filters",
                r#"
                    local result = tmpl.template(
                        "{{ text | upper }}",
                        { text = "hello" }
                    )
                    print(result)
                    assert(result == "HELLO")
                "#,
            ),
        ];

        for (name, script) in test_cases {
            println!("Running test case: {}", name);
            execute_lua(script)?;
        }

        {
            let temp_dir = TempDir::new()?;
            let template_dir = temp_dir.path().join("templates");
            fs::create_dir_all(&template_dir)?;

            let template_content = r#"Hello {{ name }} from file!"#;
            fs::write(template_dir.join("simple.txt"), template_content)?;

            let conditional_template = r#"{% if show %}Visible{% else %}Hidden{% endif %}"#;
            fs::write(template_dir.join("conditional.txt"), conditional_template)?;

            let loop_template = r#"Items: {% for item in items %}{{ item }}{% if not loop.last %}, {% endif %}{% endfor %}"#;
            fs::write(template_dir.join("loop.txt"), loop_template)?;

            fs::create_dir_all(template_dir.join("nested"))?;
            let nested_template = r#"Nested: {{ value }}"#;
            fs::write(template_dir.join("nested/child.txt"), nested_template)?;

            let lua = Lua::new();
            let context = Context {
                scaffold_dir: template_dir,
                ..Default::default()
            };
            LuaTmpl::register(&lua, context)?;

            let test_cases = vec![
                (
                    "template_file simple",
                    r#"
                        local result = tmpl.template_file("simple.txt", { name = "World" })
                        print(result)
                        assert(result == "Hello World from file!")
                    "#,
                ),
                (
                    "template_file with condition true",
                    r#"
                        local result = tmpl.template_file("conditional.txt", { show = true })
                        print(result)
                        assert(result == "Visible")
                    "#,
                ),
                (
                    "template_file with condition false",
                    r#"
                        local result = tmpl.template_file("conditional.txt", { show = false })
                        print(result)
                        assert(result == "Hidden")
                    "#,
                ),
                (
                    "template_file with loop",
                    r#"
                        local result = tmpl.template_file("loop.txt", { items = { "apple", "banana", "cherry" } })
                        print(result)
                        assert(result == "Items: apple, banana, cherry")
                    "#,
                ),
                (
                    "template_file nested path",
                    r#"
                        local result = tmpl.template_file("nested/child.txt", { value = 42 })
                        print(result)
                        assert(result == "Nested: 42")
                    "#,
                ),
            ];

            for (name, script) in test_cases {
                println!("Running template_file test case: {}", name);
                lua.load(script).exec()?;
            }

            let error_cases = vec![(
                "template_file non-existent file",
                r#"
                        tmpl.template_file("nonexistent.txt", {})
                    "#,
                "template not found",
            )];

            for (name, script, error_message) in error_cases {
                println!("Running template_file error case: {}", name);
                let result = lua.load(script).exec();
                assert!(result.is_err());
                let err_string = result.unwrap_err().to_string();
                assert!(
                    err_string.contains(error_message),
                    "Expected error message to contain '{}', but got '{}'",
                    error_message,
                    err_string
                );
            }
        }

        let error_cases = vec![
            (
                "invalid regex",
                r#"
                    tmpl.patch("hello world", "[", "universe")
                "#,
                "Invalid regex pattern '['",
            ),
            (
                "template syntax error",
                r#"
                    tmpl.template("{% invalid syntax %}", {})
                "#,
                "syntax error: unknown statement invalid",
            ),
            (
                "template undefined variable",
                r#"
                    tmpl.template("{{ undefined_var }}", {})
                "#,
                "undefined value",
            ),
        ];

        for (name, script, error_message) in error_cases {
            println!("Running error case: {}", name);
            let result = execute_lua(script);
            assert!(result.is_err());
            let err = result.unwrap_err();
            let err_string = format!("{:?}", err);
            assert!(
                err_string.contains(error_message),
                "Expected error message to contain '{}', but got '{}'",
                error_message,
                err_string
            );
        }

        Ok(())
    }
}
