use eyre::Result;
use kenchiku_common::Context;
use mlua::Lua;

pub struct LuaFS;

impl LuaFS {
    pub fn register(lua: &Lua, context: Context) -> Result<()> {
        let fs_table = lua.create_table()?;

        fs_table.set(
            "exists",
            lua.create_function(move |_, path: String| {
                Ok(context.working_dir.join(&path).exists())
            })?,
        )?;

        lua.globals().set("fs", fs_table)?;

        Ok(())
    }
}
