use eyre::{Context as _, Result, eyre};
use kenchiku_common::Context;
use kenchiku_lua::{exec::LuaExec, fs::LuaFS, log::LuaLog, tmpl::LuaTmpl};
use mlua::{FromLua, Lua};
use std::{fs::read_to_string, path::PathBuf};
use tracing::{debug, info, warn};

use crate::{meta::ScaffoldMeta, utils::move_files_to_destination};

pub mod discovery;
mod meta;
mod utils;

#[derive(Debug)]
pub struct Scaffold {
    #[allow(dead_code)]
    lua: Lua,
    pub name: String,
    pub path: PathBuf,
    pub meta: ScaffoldMeta,
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
        lua.sandbox(true)?;

        let file_content = read_to_string(&scaffold_lua_path)?;
        let scaffold_content: mlua::Value = lua
            .load(&file_content)
            .set_name(scaffold_lua_path.display().to_string())
            .eval()
            .wrap_err("failed to load scaffold.lua")?;

        let meta = ScaffoldMeta::from_lua(scaffold_content, &lua)?;
        let name: String = path
            .file_name()
            .expect("to get filename of path")
            .to_str()
            .expect("to get filename of path")
            .to_owned();

        Ok(Self {
            lua,
            name,
            path,
            meta,
        })
    }

    fn register_functions(&self, context: Context) -> Result<()> {
        LuaLog::register(&self.lua, context.clone())?;
        LuaFS::register(&self.lua, context.clone())?;
        LuaExec::register(&self.lua, context.clone())?;
        LuaTmpl::register(&self.lua, context)?;
        Ok(())
    }

    pub fn call_construct(self, context: Context) -> Result<()> {
        self.register_functions(context)?;
        self.meta
            .construct
            .call::<()>(())
            .wrap_err("failed to call construct function")
    }

    pub fn call_patch(self, name: &str, context: Context) -> Result<()> {
        self.register_functions(context)?;
        let patch_meta = self
            .meta
            .patches
            .iter()
            .find(|patch| patch.0 == name)
            .ok_or(eyre!("no patch with name '{}' found", name))?
            .1;
        patch_meta
            .run
            .call::<()>(())
            .wrap_err("failed to call patch function")
    }

    pub fn print(self) {
        println!("Name: {}", self.name);
        println!("Description: {}", self.meta.description);
        println!("Patches:");
        for patch in self.meta.patches {
            println!("  - Name: {}", patch.0);
            println!("    Description: {}", patch.1.description);
        }
    }

    pub fn construct(self, context: Context) -> Result<()> {
        debug!(dir = ?context.working_dir, "Constructing scaffold");
        self.call_construct(context.clone())?;
        let remaining = move_files_to_destination(
            &context.working_dir,
            &context.output,
            // TODO: make configurable?
            true,
            context.allow_overwrite,
        )?;
        if remaining.len() > 0 {
            let paths_pretty = remaining
                .iter()
                .map(|path| format!("- {}", path.display()))
                .collect::<Vec<_>>()
                .join("\n");
            warn!("Existing files are in the way, please manually copy these over:\n{paths_pretty}")
        } else {
            info!(to = ?context.output, "Scaffold files successfully copied over");
            std::fs::remove_dir_all(context.working_dir)?;
        }
        Ok(())
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
                construct = function() end,
            }
        "#;
        let scaffold_lua = tmp.path().join("scaffold.lua");
        fs::write(scaffold_lua, lua_content).unwrap();

        let res = Scaffold::load(tmp.path().to_path_buf()).expect("scaffold to load");

        assert_eq!(res.meta.description, "hello world!".to_string());
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
