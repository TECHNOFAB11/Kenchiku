use eyre::Context as _;
use kenchiku_common::{Context, IntoLuaErrDebug};
use mlua::{Lua, LuaSerdeExt, Result};

pub struct LuaJson;

impl LuaJson {
    pub fn register(lua: &Lua, _context: Context) -> Result<()> {
        let json_table = lua.create_table()?;

        json_table.set(
            "encode",
            lua.create_function(|_lua, data: mlua::Value| {
                Ok(serde_json::to_string(&data)
                    .wrap_err("failed to encode value to json")
                    .into_lua_err_debug()?)
            })?,
        )?;

        json_table.set(
            "decode",
            lua.create_function(|lua, data: String| -> Result<mlua::Value> {
                let value: serde_json::Value = serde_json::from_str(&data)
                    .wrap_err("failed to decode json")
                    .into_lua_err_debug()?;
                lua.to_value(&value)
            })?,
        )?;

        lua.globals().set("json", json_table)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenchiku_common::Context;
    use mlua::Lua;

    #[test]
    fn test_lua_json_encode() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaJson::register(&lua, context)?;

        // Test encoding a simple table
        lua.load(
            r#"
            local result = json.encode({foo = "bar", num = 42, bool = true})
            assert(result == '{"bool":true,"num":42,"foo":"bar"}')
        "#,
        )
        .exec()?;

        // Test encoding nested tables
        lua.load(
            r#"
            local result = json.encode({
                name = "test",
                data = {nested = "value", array = {1, 2, 3}}
            })
            assert(result == '{"name":"test","data":{"nested":"value","array":[1,2,3]}}')
        "#,
        )
        .exec()?;

        // Test encoding arrays
        lua.load(
            r#"
            local result = json.encode({"apple", "banana", "cherry"})
            assert(result == '["apple","banana","cherry"]')
        "#,
        )
        .exec()?;

        // Test encoding nil values
        lua.load(
            r#"
            local result = json.encode({present = "value", missing = nil})
            assert(result == '{"present":"value"}')
        "#,
        )
        .exec()?;

        // Test encoding numbers (integers and floats)
        lua.load(
            r#"
            local result = json.encode({int = 100, float = 3.14})
            assert(result == '{"int":100,"float":3.14}')
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_json_decode() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaJson::register(&lua, context)?;

        // Test decoding simple object
        lua.load(
            r#"
            local result = json.decode('{"name":"test","value":123}')
            assert(type(result) == "table")
            assert(result.name == "test")
            assert(result.value == 123)
        "#,
        )
        .exec()?;

        // Test decoding nested object
        lua.load(
            r#"
            local result = json.decode('{"data":{"nested":"value","count":5}}')
            assert(type(result) == "table")
            assert(type(result.data) == "table")
            assert(result.data.nested == "value")
            assert(result.data.count == 5)
        "#,
        )
        .exec()?;

        // Test decoding array
        lua.load(
            r#"
            local result = json.decode('["first","second","third"]')
            assert(type(result) == "table")
            assert(#result == 3)
            assert(result[1] == "first")
            assert(result[2] == "second")
            assert(result[3] == "third")
        "#,
        )
        .exec()?;

        // Test decoding mixed array/object
        lua.load(
            r#"
            local result = json.decode('{"items":[{"id":1},{"id":2}],"count":2}')
            assert(type(result) == "table")
            assert(type(result.items) == "table")
            assert(#result.items == 2)
            assert(result.items[1].id == 1)
            assert(result.items[2].id == 2)
            assert(result.count == 2)
        "#,
        )
        .exec()?;

        // Test decoding boolean and null
        lua.load(
            r#"
            local result = json.decode('{"active":true,"inactive":false,"nothing":null}')
            assert(result.active == true)
            assert(result.inactive == false)
            assert(result.nothing ~= nil)
            -- how mlua represents serde's Value::Null
            assert(type(result.nothing) == "userdata")
        "#,
        )
        .exec()?;

        // Test decoding numbers
        lua.load(
            r#"
            local result = json.decode('{"int":42,"float":3.14159,"negative":-10}')
            assert(result.int == 42)
            assert(result.float == 3.14159)
            assert(result.negative == -10)
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_json_encode_decode_roundtrip() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaJson::register(&lua, context)?;

        // Test roundtrip with complex data
        lua.load(
            r#"
            local original = {
                name = "test",
                numbers = {1, 2, 3, 4, 5},
                nested = {
                    flag = true,
                    value = nil,
                    items = {"a", "b", "c"}
                }
            }

            local encoded = json.encode(original)
            local decoded = json.decode(encoded)

            -- Verify structure
            assert(decoded.name == "test")
            assert(#decoded.numbers == 5)
            assert(decoded.numbers[1] == 1)
            assert(decoded.numbers[5] == 5)
            assert(decoded.nested.flag == true)
            assert(decoded.nested.value == nil)
            assert(#decoded.nested.items == 3)
            assert(decoded.nested.items[1] == "a")
            assert(decoded.nested.items[3] == "c")
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_json_error_handling() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaJson::register(&lua, context)?;

        // Test decode error with invalid JSON
        let result = lua
            .load(
                r#"
            json.decode('{invalid json}')
        "#,
            )
            .exec();
        assert!(result.is_err(), "Should fail on invalid JSON");

        // Test decode error with malformed JSON
        let result = lua
            .load(
                r#"
            json.decode('{"unclosed": "string}')
        "#,
            )
            .exec();
        assert!(result.is_err(), "Should fail on malformed JSON");

        Ok(())
    }

    #[test]
    fn test_lua_json_global_table() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaJson::register(&lua, context)?;

        // Verify json table exists and has expected functions
        lua.load(
            r#"
            assert(type(json) == "table")
            assert(type(json.encode) == "function")
            assert(type(json.decode) == "function")
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_json_empty_and_edge_cases() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = Context::default();
        LuaJson::register(&lua, context)?;

        // Test empty object
        lua.load(
            r#"
            local encoded = json.encode({})
            assert(encoded == '{}')
            local decoded = json.decode('{}')
            assert(type(decoded) == "table")
            assert(next(decoded) == nil) -- empty table
        "#,
        )
        .exec()?;

        // Test empty array
        lua.load(
            r#"
            local decoded = json.decode('[]')
            assert(type(decoded) == "table")
            assert(#decoded == 0)
        "#,
        )
        .exec()?;

        // Test with special characters in strings
        lua.load(
            r#"
            local encoded = json.encode({text = 'Line1\nLine2\tTab\\Backslash"Quote'})
            local decoded = json.decode(encoded)
            assert(decoded.text == 'Line1\nLine2\tTab\\Backslash"Quote')
        "#,
        )
        .exec()?;

        // Test with unicode characters
        lua.load(
            r#"
            local encoded = json.encode({unicode = "Hello 世界 🎉"})
            local decoded = json.decode(encoded)
            assert(decoded.unicode == "Hello 世界 🎉")
        "#,
        )
        .exec()?;

        Ok(())
    }
}
