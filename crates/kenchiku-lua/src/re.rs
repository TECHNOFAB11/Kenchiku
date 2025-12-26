use eyre::Result;
use kenchiku_common::Context;
use mlua::Lua;
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

        lua.globals().set("re", re_table)?;

        Ok(())
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
}
