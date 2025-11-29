use eyre::{Result, eyre};
use mlua::Lua;
use std::{fs::read_to_string, path::PathBuf};
use tracing::debug;

#[derive(Debug)]
pub struct Scaffold {
    #[allow(dead_code)]
    lua: Lua,
    pub path: PathBuf,
    pub description: String,
}

impl Scaffold {
    pub fn load(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(eyre!("Path does not exist"));
        }
        let scaffold_lua_path = path.join("scaffold.lua");
        if !scaffold_lua_path.exists() {
            return Err(eyre!(
                "Scaffold '{}' does not contain scaffold.lua file",
                path.display()
            ));
        }

        debug!(?path, "loading scaffold...");

        let lua = Lua::new();
        let file_content = read_to_string(scaffold_lua_path)?;
        let scaffold_content: mlua::Value = lua.load(&file_content).eval()?;

        let table = match scaffold_content {
            mlua::Value::Table(table) => table,
            other => {
                return Err(eyre!(
                    "Scaffolds need to return lua tables, this one returned {:?}",
                    other
                ));
            }
        };

        let description = table
            .get("description")
            .unwrap_or("no description".to_string());

        Ok(Self {
            lua,
            path,
            description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn test_load_valid_path() {
        let tmp = tempfile::tempdir().unwrap();
        let lua_content = r#"
            return {
                description = "hello world!",
            }
        "#;
        let scaffold_lua = tmp.path().join("scaffold.lua");
        fs::write(scaffold_lua, lua_content).unwrap();

        let res = Scaffold::load(tmp.path().to_path_buf()).expect("scaffold to load");

        assert_eq!(res.description, "hello world!".to_string());
    }
    #[test]
    fn test_load_invalid_paths() {
        let res = Scaffold::load(Path::new("bogus").to_path_buf());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Path does not exist".to_string()
        );

        let tmp = tempfile::tempdir().unwrap();
        let res = Scaffold::load(tmp.path().to_path_buf());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            format!(
                "Scaffold '{}' does not contain scaffold.lua file",
                tmp.path().display()
            )
            .to_string()
        );
    }
}
