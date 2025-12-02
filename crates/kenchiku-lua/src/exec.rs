use kenchiku_common::Context;
use mlua::{Lua, Result};
use std::process::Command;

pub struct LuaExec;

impl LuaExec {
    pub fn register(lua: &Lua, context: Context) -> Result<()> {
        let exec_table = lua.create_table()?;

        exec_table.set(
            "run",
            lua.create_function(move |lua, command: String| {
                if context.confirm_all < 2 {
                    let ans = (context.confirm_fn)(format!(
                        "[sys] Execute command '{}' in {}?",
                        command,
                        context.working_dir.display()
                    ));

                    match ans {
                        Ok(true) => {}
                        _ => return Err(mlua::Error::external("command denied by user")),
                    }
                }

                let output = Command::new("sh")
                    .current_dir(&context.working_dir)
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
