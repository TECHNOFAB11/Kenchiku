use eyre::Result;
use kenchiku_common::Context;
use mlua::Lua;
pub struct LuaLog;

impl LuaLog {
    pub fn register(lua: &Lua, _context: Context) -> Result<()> {
        let warn_fun = lua.create_function(|_, message: String| {
            tracing::warn!("Warning from scaffold: {}", message);
            // TODO: also write to writer of context, see below
            Ok(())
        })?;
        lua.globals().set("warn", warn_fun)?;

        let print_fun = lua.create_function(|_, message: String| {
            // TODO: add Writer to context which specifies where to write these messages to (for
            // mcp)
            println!("{}", message);
            Ok(())
        })?;
        lua.globals().set("print", print_fun)?;

        Ok(())
    }
}
