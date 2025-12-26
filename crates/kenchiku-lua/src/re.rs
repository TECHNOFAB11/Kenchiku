use eyre::{Context as _, Result, eyre};
use kenchiku_common::{Context, IntoLuaErrDebug as _};
use mlua::{FromLua, Lua};
use regex::Regex;

pub struct LuaRe;

impl LuaRe {
    pub fn register(lua: &Lua, _context: Context) -> Result<()> {
        let re_table = lua.create_table()?;

        re_table.set(
            "match",
            lua.create_function(move |lua, (text, regex_str): (String, String)| {
                let regex = Regex::new(&regex_str)
                    .map_err(|e| mlua::Error::external(format!("Invalid regex: {}", e)))?;

                if let Some(captures) = regex.captures(&text) {
                    let matches = lua.create_table()?;
                    for (i, match_item) in captures.iter().enumerate() {
                        if let Some(m) = match_item {
                            // +1 so its 1 indexed for lua
                            matches.set(i + 1, m.as_str())?;
                        }
                    }

                    for name in regex.capture_names() {
                        if let Some(n) = name {
                            if let Some(m) = captures.name(n) {
                                matches.set(n, m.as_str())?;
                            }
                        }
                    }
                    Ok(Some(matches))
                } else {
                    Ok(None)
                }
            })?,
        )?;

        re_table.set(
            "replace",
            lua.create_function(
                move |_lua,
                      (content, pattern, replacement, opts): (
                    String,
                    String,
                    String,
                    LuaReReplaceOpts,
                )| {
                    let re = Regex::new(&pattern)
                        .wrap_err(format!("Invalid regex pattern '{}'", pattern))
                        .into_lua_err_debug()?;
                    let modified_content =
                        re.replacen(&content, opts.limit, replacement).to_string();
                    Ok(modified_content)
                },
            )?,
        )?;

        lua.globals().set("re", re_table)?;

        Ok(())
    }
}

#[derive(Default)]
struct LuaReReplaceOpts {
    limit: usize,
}

impl FromLua for LuaReReplaceOpts {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = match value {
            mlua::Value::Table(table) => table,
            // allow not passing any options table, then default to default
            mlua::Value::Nil => return Ok(Self::default()),
            other => {
                return Err(eyre!("Opts needs to be a table, received {:?}", other))
                    .into_lua_err_debug();
            }
        };
        Ok(Self {
            limit: table.get("limit").unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_re_match() -> Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaRe::register(&lua, context)?;

        let execute_lua = |script: &str| -> Result<()> {
            lua.load(script).exec()?;
            Ok(())
        };

        // Test basic match
        execute_lua(
            r#"
            local m = re.match("hello world", "hello")
            assert(m[1] == "hello")
            "#,
        )?;

        // Test capture groups
        execute_lua(
            r#"
            local m = re.match("hello world", "(\\w+) (\\w+)")
            assert(m[1] == "hello world")
            assert(m[2] == "hello")
            assert(m[3] == "world")
            "#,
        )?;

        // Test no match
        execute_lua(
            r#"
            local m = re.match("foo", "bar")
            assert(m == nil)
            "#,
        )?;

        // Test named capture groups
        execute_lua(
            r#"
            local m = re.match("2023-12-25", "(?P<y>\\d{4})-(?P<m>\\d{2})-(?P<d>\\d{2})")
            assert(m[1] == "2023-12-25")
            assert(m["y"] == "2023")
            assert(m["m"] == "12")
            assert(m["d"] == "25")
            "#,
        )?;

        // Test invalid regex
        let result = execute_lua(
            r#"
            re.match("bar", "(")
            "#,
        );
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_lua_re_replace() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaRe::register(&lua, context)?;

        let execute_lua = |script: &str| -> eyre::Result<()> {
            lua.load(script).exec()?;
            Ok(())
        };

        let test_cases = vec![
            (
                "simple replacement",
                r#"
                    local result = re.replace("hello world", "world", "universe")
                    print(result)
                    assert(result == "hello universe")
                "#,
            ),
            (
                "regex replacement",
                r#"
                    local result = re.replace("hello world", "[[:word:]]+", "universe")
                    print(result)
                    assert(result == "universe universe")
                "#,
            ),
            (
                "no match",
                r#"
                    local result = re.replace("hello world", "nomatch", "universe")
                    print(result)
                    assert(result == "hello world")
                "#,
            ),
            (
                "capture groups",
                r#"
                    local result = re.replace("hello world", "h(ello) world", "H$1 WORLD")
                    print(result)
                    assert(result == "Hello WORLD")
                "#,
            ),
            (
                "unicode replacement",
                r#"
                    local result = re.replace("你好世界", "世界", "世界你好")
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
                    re.replace("hello world", "[", "universe")
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
