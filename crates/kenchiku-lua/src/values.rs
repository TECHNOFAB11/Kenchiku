use eyre::{Context as _, ContextCompat, Result, eyre};
use kenchiku_common::Context;
use mlua::{ExternalResult, IntoLua, Lua};
use tracing::{debug, trace};

pub struct LuaValues;

impl LuaValues {
    pub fn register(lua: &Lua, context: Context) -> Result<()> {
        let values_table = lua.create_table()?;

        values_table.set(
            "get",
            lua.create_function(move |lua, id: String| {
                let meta = context.values_meta.get(&id);
                let val = context.values.get(&id);
                debug!(id, ?val, ?meta, "Getting value");
                if meta.is_none() {
                    return Err(eyre!("No value named {} defined", id)).into_lua_err();
                }
                let meta = meta.unwrap();
                // 1. if value was already set
                if val.is_some() {
                    trace!(id, "Value was already set");
                    return string_to_value_of_type(
                        &lua,
                        meta.r#type.clone(),
                        val.unwrap(),
                        meta.choices.clone(),
                        id,
                    );
                }
                // 2. if default exists
                if meta.default.is_some() {
                    trace!(id, "Default exists");
                    return match meta.r#type.as_str() {
                        "enum" => validate_enum_contains(
                            &lua,
                            meta.choices.clone(),
                            &meta.default.clone().unwrap().to_string()?,
                        ),
                        _ => Ok(meta.default.clone().unwrap()),
                    };
                }
                // 3. if we have no value, ask the user
                trace!(id, "Asking user for value...");
                let answer = (context.prompt_value)(
                    meta.r#type.clone(),
                    meta.description.clone(),
                    meta.choices.clone(),
                )
                .into_lua_err()?;
                return string_to_value_of_type(
                    &lua,
                    meta.r#type.clone(),
                    &answer,
                    meta.choices.clone(),
                    id,
                );
            })?,
        )?;

        lua.globals().set("values", values_table)?;

        Ok(())
    }
}

fn validate_enum_contains(
    lua: &mlua::Lua,
    choices: Option<Vec<String>>,
    val: &String,
) -> Result<mlua::Value, mlua::Error> {
    let choices = choices
        .wrap_err("no choices on enum type value")
        .into_lua_err()?;
    if !choices.contains(val) {
        return Err(eyre!("Invalid choice for enum: {}", val)).into_lua_err();
    }
    Ok(val.clone().into_lua(lua)?)
}

fn string_to_value_of_type(
    lua: &mlua::Lua,
    val_type: String,
    val: &String,
    choices: Option<Vec<String>>,
    id: String,
) -> Result<mlua::Value, mlua::Error> {
    let value = match val_type.as_str() {
        "string" => val.clone().into_lua(lua)?,
        "enum" => validate_enum_contains(&lua, choices.clone(), val)
            .wrap_err(format!("on value {id}"))
            .into_lua_err()?,
        "number" => val
            .parse::<usize>()
            .wrap_err("failed to parse value as a number")
            .into_lua_err()?
            .into_lua(lua)?,
        "bool" => val
            .parse::<bool>()
            .wrap_err("failed to parse value as a bool")
            .into_lua_err()?
            .into_lua(lua)?,
        _ => mlua::Value::Nil,
    };
    Ok(value)
}
