use eyre::Result;
use kenchiku_common::Context;
use mlua::Lua;
pub struct LuaLog;

impl LuaLog {
    pub fn register(lua: &Lua, _context: Context) -> Result<()> {
        let warn_fun = lua.create_function(|_, message: String| {
            tracing::warn!("Warning from scaffold: {}", message);
            Ok(())
        })?;
        lua.globals().set("warn", warn_fun)?;

        Ok(())
    }
}
