use eyre::{Context as _, Result};
use kenchiku_common::Context;
use kenchiku_common::IntoLuaErrDebug;
use mlua::Lua;
use regex::Regex;

pub struct LuaTmpl;

impl LuaTmpl {
    pub fn register(lua: &Lua, _context: Context) -> Result<()> {
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

        lua.globals().set("tmpl", tmpl_table)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_lua_tmpl() -> eyre::Result<()> {
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
}
