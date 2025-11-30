use inquire::Confirm;
use mlua::{Lua, Result};
use std::process::Command;

pub struct LuaExec;

impl LuaExec {
    pub fn register(lua: &Lua) -> Result<()> {
        let exec_table = lua.create_table()?;

        exec_table.set(
            "run",
            lua.create_function(|lua, command: String| {
                // TODO: dont use inquire here directly, maybe pass a callback in ctx or so, then
                // cli can use inquire
                let ans = Confirm::new(&format!("Execute command: '{}'?", command))
                    .with_default(false)
                    .prompt();

                match ans {
                    Ok(true) => {}
                    _ => return Ok(mlua::Value::Nil),
                }

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .map_err(mlua::Error::external)?;

                if output.status.success() {
                    Ok(mlua::Value::String(lua.create_string(
                        String::from_utf8_lossy(&output.stdout).to_string(),
                    )?))
                } else {
                    Err(mlua::Error::external(format!(
                        "Command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )))
                }
            })?,
        )?;

        lua.globals().set("exec", exec_table)?;

        Ok(())
    }
}
