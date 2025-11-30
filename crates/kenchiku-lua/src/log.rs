use eyre::Result;
use mlua::Lua;

pub struct LuaLog;

impl LuaLog {
    pub fn register(lua: &Lua) -> Result<()> {
        let warn_fun = lua.create_function(|_, message: String| {
            tracing::warn!("Warning from scaffold: {}", message);
            Ok(())
        })?;
        lua.globals().set("warn", warn_fun)?;

        Ok(())
    }
}
