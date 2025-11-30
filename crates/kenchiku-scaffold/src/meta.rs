use eyre::eyre;
use mlua::{ExternalError, FromLua, Lua};
use std::collections::HashMap;

fn get_and_check<'lua, T>(
    table: &mlua::Table,
    key: &str,
    type_name: &str,
    lua: &'lua Lua,
) -> mlua::Result<T>
where
    T: FromLua,
{
    match table.get(key)? {
        mlua::Value::Nil => Err(eyre!("'{}' field is missing in the table", key).into_lua_err()),
        value => match T::from_lua(value.clone(), lua) {
            Ok(typed_value) => Ok(typed_value),
            Err(_) => Err(
                eyre!("'{}' field must be a {}, got {:?}", key, type_name, value).into_lua_err(),
            ),
        },
    }
}

#[derive(Debug)]
pub struct PatchMeta {
    /// Description of what the patch does.
    pub description: String,
    /// Function which executes the patch.
    pub run: mlua::Function,
}

impl FromLua for PatchMeta {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let table = match value {
            mlua::Value::Table(table) => table,
            other => {
                return Err(eyre!(
                    "Scaffold needs to return a table for patches, received {:?}",
                    other
                )
                .into_lua_err());
            }
        };

        Ok(PatchMeta {
            description: get_and_check(&table, "description", "string", &lua)
                .map(|val: String| val.trim().to_string())?,
            run: get_and_check(&table, "run", "function", &lua)?,
        })
    }
}

#[derive(Debug)]
pub struct ScaffoldMeta {
    /// Description of what the scaffold does.
    pub description: String,
    /// Function which executes the scaffold.
    pub construct: mlua::Function,
    /// Patches this scaffold exposes.
    pub patches: HashMap<String, PatchMeta>,
}

impl FromLua for ScaffoldMeta {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let table = match value {
            mlua::Value::Table(table) => table,
            other => {
                return Err(eyre!(
                    "Scaffolds need to return lua tables, this one returned {:?}",
                    other
                )
                .into_lua_err());
            }
        };
        Ok(ScaffoldMeta {
            description: get_and_check(&table, "description", "string", &lua)
                .map(|val: String| val.trim().to_string())?,
            construct: get_and_check(&table, "construct", "function", &lua)?,
            patches: table.get("patches").unwrap_or_default(),
        })
    }
}
