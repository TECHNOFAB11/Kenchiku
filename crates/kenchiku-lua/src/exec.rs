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

#[cfg(test)]
mod tests {
    use super::*;
    use kenchiku_common::Context;
    use mlua::Lua;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    fn create_test_context_with_confirm(
        auto_confirm: bool,
        working_dir: Option<PathBuf>,
    ) -> Context {
        let confirmed = Arc::new(Mutex::new(Vec::new()));
        let confirmed_clone = confirmed.clone();

        Context {
            working_dir: working_dir.unwrap_or_else(|| std::env::temp_dir()),
            confirm_all: if auto_confirm { 2 } else { 0 },
            confirm_fn: Arc::new(move |prompt: String| {
                confirmed_clone.lock().unwrap().push(prompt);
                Ok(auto_confirm)
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_lua_exec_simple_command() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result = exec.run("echo 'Hello World'")
            assert(type(result) == "string")
            assert(result:match("Hello World") ~= nil)
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_command_with_output() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result = exec.run("echo -n 'test123'")
            assert(result == "test123")
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_multiline_output() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result = exec.run("echo 'line1'; echo 'line2'; echo 'line3'")
            assert(result:match("line1") ~= nil)
            assert(result:match("line2") ~= nil)
            assert(result:match("line3") ~= nil)
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_working_directory() -> eyre::Result<()> {
        let lua = Lua::new();
        let temp_dir = std::env::temp_dir();
        let context = create_test_context_with_confirm(true, Some(temp_dir.clone()));
        LuaExec::register(&lua, context)?;

        let expected_dir = temp_dir.canonicalize()?;
        lua.load(&format!(
            r#"
            local result = exec.run("pwd")
            local trimmed = result:gsub("%s+$", "")
            assert(trimmed == "{}")
        "#,
            expected_dir.display()
        ))
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_command_failure() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        let result = lua
            .load(
                r#"
            exec.run("exit 1")
        "#,
            )
            .exec();

        assert!(result.is_err(), "Should fail on non-zero exit code");

        Ok(())
    }

    #[test]
    fn test_lua_exec_nonexistent_command() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        let result = lua
            .load(
                r#"
            exec.run("this_command_definitely_does_not_exist_12345")
        "#,
            )
            .exec();

        assert!(result.is_err(), "Should fail on nonexistent command");

        Ok(())
    }

    #[test]
    fn test_lua_exec_stderr_on_failure() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        let result = lua
            .load(
                r#"
            exec.run("echo 'error message' >&2; exit 1")
        "#,
            )
            .exec();

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("error message"),
            "Error should contain stderr output"
        );

        Ok(())
    }

    #[test]
    fn test_lua_exec_confirmation_prompt() -> eyre::Result<()> {
        let lua = Lua::new();
        let confirmed = Arc::new(Mutex::new(Vec::new()));
        let confirmed_clone = confirmed.clone();

        let context = Context {
            working_dir: std::env::temp_dir(),
            confirm_all: 0,
            confirm_fn: Arc::new(move |prompt: String| {
                confirmed_clone.lock().unwrap().push(prompt);
                Ok(true)
            }),
            ..Default::default()
        };

        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            exec.run("echo 'test'")
        "#,
        )
        .exec()?;

        let prompts = confirmed.lock().unwrap();
        assert_eq!(prompts.len(), 1, "Should have prompted once");
        assert!(
            prompts[0].contains("[sys] Execute command"),
            "Prompt should mention command execution"
        );
        assert!(
            prompts[0].contains("echo 'test'"),
            "Prompt should contain the command"
        );

        Ok(())
    }

    #[test]
    fn test_lua_exec_confirmation_denied() -> eyre::Result<()> {
        let lua = Lua::new();

        let context = Context {
            working_dir: std::env::temp_dir(),
            confirm_all: 0,
            confirm_fn: Arc::new(move |_prompt: String| Ok(false)),
            ..Default::default()
        };

        LuaExec::register(&lua, context)?;

        let result = lua
            .load(
                r#"
            exec.run("echo 'test'")
        "#,
            )
            .exec();

        assert!(result.is_err(), "Should fail when confirmation is denied");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("denied by user"),
            "Error should mention user denial"
        );

        Ok(())
    }

    #[test]
    fn test_lua_exec_auto_confirm() -> eyre::Result<()> {
        let lua = Lua::new();
        let confirmed = Arc::new(Mutex::new(Vec::new()));
        let confirmed_clone = confirmed.clone();

        let context = Context {
            working_dir: std::env::temp_dir(),
            confirm_all: 2, // Auto-confirm
            confirm_fn: Arc::new(move |prompt: String| {
                confirmed_clone.lock().unwrap().push(prompt);
                Ok(true)
            }),
            ..Default::default()
        };

        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            exec.run("echo 'test'")
        "#,
        )
        .exec()?;

        let prompts = confirmed.lock().unwrap();
        assert_eq!(prompts.len(), 0, "Should not prompt when confirm_all >= 2");

        Ok(())
    }

    #[test]
    fn test_lua_exec_global_table() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            assert(type(exec) == "table")
            assert(type(exec.run) == "function")
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_piped_commands() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result = exec.run("echo 'hello world' | tr 'a-z' 'A-Z'")
            assert(result:match("HELLO WORLD") ~= nil)
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_empty_command() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result = exec.run("")
            assert(type(result) == "string")
            assert(result == "")
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_command_with_environment_vars() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result = exec.run("TEST_VAR=hello; echo $TEST_VAR")
            assert(result:match("hello") ~= nil)
        "#,
        )
        .exec()?;

        Ok(())
    }

    #[test]
    fn test_lua_exec_multiple_commands() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context_with_confirm(true, None);
        LuaExec::register(&lua, context)?;

        lua.load(
            r#"
            local result1 = exec.run("echo 'first'")
            local result2 = exec.run("echo 'second'")
            local result3 = exec.run("echo 'third'")

            assert(result1:match("first") ~= nil)
            assert(result2:match("second") ~= nil)
            assert(result3:match("third") ~= nil)
        "#,
        )
        .exec()?;

        Ok(())
    }
}
