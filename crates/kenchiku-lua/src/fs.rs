use std::path::Path;

use eyre::Result;
use mlua::Lua;

pub struct LuaFS;

impl LuaFS {
    pub fn register(lua: &Lua) -> Result<()> {
        let fs_table = lua.create_table()?;

        fs_table.set(
            "exists",
            lua.create_function(|_, path: String| Ok(Path::new(&path).exists()))?,
        )?;

        lua.globals().set("fs", fs_table)?;

        Ok(())
    }
}
