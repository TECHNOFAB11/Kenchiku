use std::path::{Path, PathBuf};

use eyre::{Result, eyre};
use kenchiku_common::{Context, IntoLuaErrDebug};
use mlua::{FromLua, Lua};
use tracing::debug;

pub struct LuaFS;

impl LuaFS {
    pub fn register(lua: &Lua, context: Context) -> Result<()> {
        let fs_table = lua.create_table()?;

        let working_dir = context.working_dir.clone();
        fs_table.set(
            "exists",
            lua.create_function(move |_, path: String| {
                let user_path = validate_path(&working_dir, path);
                Ok(!user_path.is_err() && user_path.unwrap().exists())
            })?,
        )?;

        let working_dir = context.working_dir.clone();
        let scaffold_dir = context.scaffold_dir.clone();
        fs_table.set(
            "read",
            lua.create_function(move |_, (path, opts): (String, LuaFsReadOpts)| {
                let path = match opts.source.as_ref() {
                    "workdir" => validate_path(&working_dir, path)?,
                    "scaffold" => validate_path(&scaffold_dir, path)?,
                    _ => {
                        return Err(eyre!(
                            "Invalid read source, must be one of workdir,scaffold"
                        ))
                        .into_lua_err_debug();
                    }
                };

                Ok(std::fs::read_to_string(&path)?)
            })?,
        )?;

        let working_dir = context.working_dir.clone();
        fs_table.set(
            "mkdir",
            lua.create_function(move |_, path: String| {
                let user_path = validate_path(&working_dir, path)?;
                Ok(std::fs::create_dir_all(&user_path)?)
            })?,
        )?;

        let working_dir = context.working_dir.clone();
        fs_table.set(
            "write",
            lua.create_function(move |_, (path, content): (String, String)| {
                let user_path = validate_path(&working_dir, path)?;
                debug!(?user_path, "Writing to file");
                Ok(std::fs::write(&user_path, content)?)
            })?,
        )?;

        lua.globals().set("fs", fs_table)?;

        Ok(())
    }
}

struct LuaFsReadOpts {
    source: String,
}

impl Default for LuaFsReadOpts {
    fn default() -> Self {
        Self {
            source: "scaffold".to_string(),
        }
    }
}

impl FromLua for LuaFsReadOpts {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = match value {
            mlua::Value::Table(table) => table,
            // allow not passing any options table, then default to default
            mlua::Value::Nil => return Ok(Self::default()),
            other => {
                return Err(eyre!("Opts needs to be a table, received {:?}", other))
                    .into_lua_err_debug();
            }
        };
        Ok(Self {
            source: table.get("source").unwrap_or_default(),
        })
    }
}

fn validate_path(working_dir: &Path, path: String) -> mlua::Result<PathBuf> {
    let specified_path = working_dir.join(&path);
    debug!(path, ?specified_path, "Validating path");
    let user_path = if specified_path.is_dir() {
        specified_path.clone()
    } else {
        specified_path
            .parent()
            .ok_or(mlua::Error::external("nope"))?
            .to_path_buf()
    };
    if !is_subpath(working_dir, &user_path)? {
        return Err(mlua::Error::external(format!(
            "scaffold is not allowed to access {}",
            user_path.display()
        )));
    }
    Ok(specified_path.to_path_buf())
}

fn is_subpath(base_path: &Path, user_path: &Path) -> Result<bool, std::io::Error> {
    let base_path_canonicalized = base_path.canonicalize()?;
    let user_path_canonicalized = user_path.canonicalize()?;

    Ok(user_path_canonicalized.starts_with(base_path_canonicalized))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn test_lua_fs() -> eyre::Result<()> {
        let temp_dir = tempdir()?;
        let working_dir = temp_dir.path().to_path_buf();

        let scaffold_temp_dir = tempdir()?;
        let scaffold_dir = scaffold_temp_dir.path().to_path_buf();
        fs::write(scaffold_dir.join("example.txt"), "hello world")?;

        let lua = Lua::new();
        let context = Context {
            working_dir: working_dir.clone(),
            scaffold_dir: scaffold_dir.clone(),
            ..Default::default()
        };
        LuaFS::register(&lua, context)?;

        let execute_lua = |script: &str| -> Result<()> {
            lua.load(script).exec()?;
            Ok(())
        };

        // Exists test
        execute_lua(&format!(
            r#"
                    local exists = fs.exists("{}")
                    assert(not exists)
                "#,
            working_dir.join("nonexistent.txt").display()
        ))?;

        // Mkdir test
        execute_lua(&format!(
            r#"
                    fs.mkdir("{}")
                "#,
            working_dir.join("new_dir").display()
        ))?;
        assert!(working_dir.join("new_dir").exists());

        // Write test
        execute_lua(
            r#"
                fs.write("test.txt", "hello world")
            "#,
        )?;
        assert_eq!(
            std::fs::read_to_string(working_dir.join("test.txt"))?,
            "hello world"
        );

        // Read test
        execute_lua(
            r#"
                local content = fs.read("test.txt", { source = "workdir" })
                assert(content == "hello world")
            "#,
        )?;

        // Read from scaffold test
        execute_lua(
            r#"
                local content = fs.read("example.txt")
                assert(content == "hello world")
            "#,
        )?;

        // Exists test 2
        execute_lua(
            r#"
                local exists = fs.exists("test.txt")
                assert(exists)
            "#,
        )?;
        Ok(())
    }

    #[test]
    fn test_is_subpath() -> eyre::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path();

        let sub_dir_path = base_path.join("subdir");
        fs::create_dir(&sub_dir_path)?;

        assert_eq!(is_subpath(base_path, &sub_dir_path)?, true);
        assert_eq!(is_subpath(base_path, base_path)?, true);

        let file_path = sub_dir_path.join("test_file.txt");
        fs::write(&file_path, "test content")?;
        assert_eq!(is_subpath(base_path, &file_path)?, true);

        let outside_path = PathBuf::from("/tmp"); // Assuming /tmp exists
        assert_eq!(is_subpath(base_path, &outside_path).unwrap_or(false), false);

        let relative_path = PathBuf::from("subdir/../another_dir");
        let absolute_relative_path = base_path.join(relative_path);
        let another_dir_path = base_path.join("another_dir");
        fs::create_dir(another_dir_path)?;
        assert_eq!(is_subpath(base_path, &absolute_relative_path)?, true);
        Ok(())
    }

    #[test]
    fn test_validate_path_valid() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let working_dir = temp_dir.path();

        // Valid path within the working directory
        let path = "foo/bar".to_string();
        fs::create_dir_all(working_dir.join(&path)).expect("directory should be created");
        let result = validate_path(working_dir, path.clone()).expect("path should be validated");
        assert_eq!(result, working_dir.join(path));
    }

    #[test]
    fn test_validate_path_working_dir() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let working_dir = temp_dir.path();

        // Path equal to the working directory
        let path = ".".to_string();
        let result = validate_path(working_dir, path).expect("path should be validated");
        assert_eq!(result, working_dir.join(".").to_path_buf());
    }

    #[test]
    fn test_validate_path_invalid() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let working_dir = temp_dir.path();

        // Invalid path (outside the working directory)
        let path = "../foo".to_string();
        let result = validate_path(working_dir, path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_file() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let working_dir = temp_dir.path();

        // Path that is a file within the working directory
        let path = "foo.txt".to_string();
        fs::write(working_dir.join(&path), "test").expect("file to be created");
        let result = validate_path(working_dir, path.clone()).expect("path should be validated");
        assert_eq!(result, working_dir.join(path));
    }
}
