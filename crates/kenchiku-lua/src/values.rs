use eyre::{Context as _, ContextCompat, Result, eyre};
use kenchiku_common::{Context, IntoLuaErrDebug};
use mlua::{IntoLua, Lua};
use std::sync::Arc;
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
                    return Err(eyre!("No value named {} defined", id)).into_lua_err_debug();
                }
                let meta = meta.unwrap();

                let run_validation = |val: &str| -> Result<(), String> {
                    if let Some(validate) = &meta.validate {
                        let res: mlua::Value =
                            validate.call(val.to_string()).map_err(|e| e.to_string())?;
                        match res {
                            mlua::Value::Boolean(true) => Ok(()),
                            mlua::Value::Boolean(false) => {
                                Err("validate function rejected value".to_string())
                            }
                            mlua::Value::String(s) => Err(s.to_string_lossy()),
                            _ => Err("Invalid return value from validate fn".to_string()),
                        }
                    } else {
                        Ok(())
                    }
                };

                // 1. if value was already set
                if val.is_some() {
                    trace!(id, "Value was already set");
                    let val_str = val.unwrap();
                    if let Err(e) = run_validation(&val_str) {
                        return Err(eyre!("Value '{}' for '{}' is invalid: {}", val_str, id, e))
                            .into_lua_err_debug();
                    }
                    return string_to_value_of_type(
                        &lua,
                        meta.r#type.clone(),
                        val_str,
                        meta.choices.clone(),
                        id,
                    );
                }
                // 2. if value is unset, ask the user
                trace!(id, "Asking user for value...");

                let validator: Option<Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>> =
                    if meta.validate.is_some() {
                        let validate = meta.validate.clone().unwrap();
                        Some(Arc::new(move |input: &str| {
                            let res: mlua::Value = validate
                                .call(input.to_string())
                                .map_err(|e| e.to_string())?;
                            match res {
                                mlua::Value::Boolean(true) => Ok(()),
                                mlua::Value::String(s) => Err(s
                                    .to_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|_| "Invalid value".to_string())),
                                _ => Err("Invalid value".to_string()),
                            }
                        }))
                    } else {
                        None
                    };

                let answer = (context.prompt_value)(
                    id.clone(),
                    meta.r#type.clone(),
                    meta.description.clone(),
                    meta.choices.clone(),
                    meta.default.clone().map(|v| {
                        v.to_string()
                            .expect("lua default should be serializable to string")
                    }),
                    validator,
                )
                .into_lua_err_debug()?;

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
        .into_lua_err_debug()?;
    if !choices.contains(val) {
        return Err(eyre!("Invalid choice for enum: {}", val)).into_lua_err_debug();
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
            .into_lua_err_debug()?,
        "number" => val
            .parse::<usize>()
            .wrap_err("failed to parse value as a number")
            .into_lua_err_debug()?
            .into_lua(lua)?,
        "bool" => val
            .parse::<bool>()
            .wrap_err("failed to parse value as a bool")
            .into_lua_err_debug()?
            .into_lua(lua)?,
        _ => mlua::Value::Nil,
    };
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenchiku_common::{Context, meta::ValueMeta};
    use mlua::{FromLua as _, Lua};
    use std::{collections::HashMap, sync::Arc};

    fn create_test_context(
        values: HashMap<String, String>,
        values_meta: HashMap<String, ValueMeta>,
        prompt_response: Option<String>,
    ) -> Context {
        Context {
            values,
            values_meta,
            prompt_value: Arc::new(move |_name, _type, _desc, _choices, _default, _validator| {
                Ok(prompt_response.clone().unwrap_or_default())
            }),
            ..Default::default()
        }
    }

    fn execute_lua_with_context(lua: &Lua, script: &str, context: Context) -> eyre::Result<()> {
        LuaValues::register(lua, context)?;
        lua.load(script).exec()?;
        Ok(())
    }

    fn eval_lua_with_context(
        lua: &Lua,
        script: &str,
        context: Context,
    ) -> eyre::Result<mlua::Value> {
        LuaValues::register(lua, context)?;
        Ok(lua.load(script).eval()?)
    }

    #[test]
    fn test_get_existing_string_value() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("name".to_string(), "John".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "name".to_string(),
            ValueMeta {
                r#type: "string".to_string(),
                description: "User name".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        execute_lua_with_context(
            &lua,
            r#"
                local name = values.get("name")
                assert(type(name) == "string")
                assert(name == "John")
            "#,
            context,
        )
    }

    #[test]
    fn test_get_existing_number_value() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("age".to_string(), "25".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "age".to_string(),
            ValueMeta {
                r#type: "number".to_string(),
                description: "User age".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        execute_lua_with_context(
            &lua,
            r#"
                local age = values.get("age")
                assert(type(age) == "number")
                assert(age == 25)
            "#,
            context,
        )
    }

    #[test]
    fn test_get_existing_bool_value() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("enabled".to_string(), "true".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "enabled".to_string(),
            ValueMeta {
                r#type: "bool".to_string(),
                description: "Feature flag".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        execute_lua_with_context(
            &lua,
            r#"
                local enabled = values.get("enabled")
                assert(type(enabled) == "boolean")
                assert(enabled == true)
            "#,
            context,
        )
    }

    #[test]
    fn test_get_existing_enum_value() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("color".to_string(), "red".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "color".to_string(),
            ValueMeta {
                r#type: "enum".to_string(),
                description: "Color choice".to_string(),
                default: None,
                choices: Some(vec![
                    "red".to_string(),
                    "green".to_string(),
                    "blue".to_string(),
                ]),
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        execute_lua_with_context(
            &lua,
            r#"
                local color = values.get("color")
                assert(type(color) == "string")
                assert(color == "red")
            "#,
            context,
        )
    }

    #[test]
    fn test_get_enum_value_invalid_choice() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("color".to_string(), "yellow".to_string()); // Not in choices

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "color".to_string(),
            ValueMeta {
                r#type: "enum".to_string(),
                description: "Color choice".to_string(),
                default: None,
                choices: Some(vec![
                    "red".to_string(),
                    "green".to_string(),
                    "blue".to_string(),
                ]),
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("color")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid choice for enum"));
        assert!(err.contains("yellow"));

        Ok(())
    }

    #[test]
    fn test_get_default_enum_value_invalid() -> eyre::Result<()> {
        let lua = Lua::new();

        let values = HashMap::new(); // No value set

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "color".to_string(),
            ValueMeta {
                r#type: "enum".to_string(),
                description: "Color choice".to_string(),
                default: Some(mlua::Value::String(
                    lua.create_string("yellow".to_string())?,
                )),
                choices: Some(vec![
                    "red".to_string(),
                    "green".to_string(),
                    "blue".to_string(),
                ]),
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("color")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid choice for enum"));

        Ok(())
    }

    #[test]
    fn test_prompt_for_value() -> eyre::Result<()> {
        let lua = Lua::new();

        let values = HashMap::new(); // No value set
        let values_meta = HashMap::new(); // No meta either

        let context = Context {
            values,
            values_meta,
            prompt_value: Arc::new(|_name, _type, _desc, _choices, _default, _validator| {
                Ok("PromptedValue".to_string())
            }),
            ..Default::default()
        };

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("name")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No value named name defined"));

        Ok(())
    }

    #[test]
    fn test_get_nonexistent_value() -> eyre::Result<()> {
        let lua = Lua::new();

        let values = HashMap::new();
        let values_meta = HashMap::new(); // No metadata for "nonexistent"

        let context = create_test_context(values, values_meta, None);

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("nonexistent")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No value named nonexistent defined"));

        Ok(())
    }

    #[test]
    fn test_invalid_number_parsing() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("age".to_string(), "not-a-number".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "age".to_string(),
            ValueMeta {
                r#type: "number".to_string(),
                description: "User age".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("age")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("failed to parse value as a number"));

        Ok(())
    }

    #[test]
    fn test_invalid_bool_parsing() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("enabled".to_string(), "not-a-bool".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "enabled".to_string(),
            ValueMeta {
                r#type: "bool".to_string(),
                description: "Feature flag".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("enabled")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("failed to parse value as a bool"));

        Ok(())
    }

    #[test]
    fn test_enum_without_choices() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("choice".to_string(), "something".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "choice".to_string(),
            ValueMeta {
                r#type: "enum".to_string(),
                description: "Some choice".to_string(),
                default: None,
                choices: None, // No choices defined for enum
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("choice")
            "#,
            context,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no choices on enum type value"));

        Ok(())
    }

    #[test]
    fn test_unknown_type_returns_nil() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("custom".to_string(), "somevalue".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "custom".to_string(),
            ValueMeta {
                r#type: "custom_type".to_string(), // Unknown type
                description: "Custom type".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        let result = eval_lua_with_context(
            &lua,
            r#"
                return values.get("custom")
            "#,
            context,
        )?;

        assert!(matches!(result, mlua::Value::Nil));

        Ok(())
    }

    #[test]
    fn test_multiple_value_gets() -> eyre::Result<()> {
        let lua = Lua::new();

        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());
        values.insert("age".to_string(), "30".to_string());
        values.insert("active".to_string(), "true".to_string());

        let mut values_meta = HashMap::new();
        values_meta.insert(
            "name".to_string(),
            ValueMeta {
                r#type: "string".to_string(),
                description: "User name".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );
        values_meta.insert(
            "age".to_string(),
            ValueMeta {
                r#type: "number".to_string(),
                description: "User age".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );
        values_meta.insert(
            "active".to_string(),
            ValueMeta {
                r#type: "bool".to_string(),
                description: "Active status".to_string(),
                default: None,
                choices: None,
                validate: None,
            },
        );

        let context = create_test_context(values, values_meta, None);

        execute_lua_with_context(
            &lua,
            r#"
                local name = values.get("name")
                local age = values.get("age")
                local active = values.get("active")

                assert(type(name) == "string")
                assert(name == "Alice")

                assert(type(age) == "number")
                assert(age == 30)

                assert(type(active) == "boolean")
                assert(active == true)
            "#,
            context,
        )
    }

    #[test]
    fn test_values_table_registered() -> eyre::Result<()> {
        let lua = Lua::new();
        let context = create_test_context(HashMap::new(), HashMap::new(), None);

        LuaValues::register(&lua, context)?;

        let globals = lua.globals();
        let values: mlua::Table = globals.get("values")?;

        let get_func: mlua::Function = values.get("get")?;

        assert!(get_func.call::<mlua::Value>("test").is_err());

        Ok(())
    }

    #[test]
    fn test_validation_success_bool() -> eyre::Result<()> {
        let lua = Lua::new();
        let mut values = HashMap::new();
        values.insert("age".to_string(), "25".to_string());

        let meta_table: mlua::Table = lua
            .load(
                r#"
            {
                type = "number",
                description = "User age",
                validate = function(v) return true end
            }
        "#,
            )
            .eval()?;
        let meta = ValueMeta::from_lua(mlua::Value::Table(meta_table), &lua)?;

        let mut values_meta = HashMap::new();
        values_meta.insert("age".to_string(), meta);

        let context = create_test_context(values, values_meta, None);
        execute_lua_with_context(
            &lua,
            r#"
                local age = values.get("age")
                assert(age == 25)
            "#,
            context,
        )
    }

    #[test]
    fn test_validation_failure_bool() -> eyre::Result<()> {
        let lua = Lua::new();
        let mut values = HashMap::new();
        values.insert("age".to_string(), "25".to_string());

        let meta_table: mlua::Table = lua
            .load(
                r#"
            {
                type = "number",
                description = "User age",
                validate = function(v) return false end
            }
        "#,
            )
            .eval()?;
        let meta = ValueMeta::from_lua(mlua::Value::Table(meta_table), &lua)?;

        let mut values_meta = HashMap::new();
        values_meta.insert("age".to_string(), meta);

        let context = create_test_context(values, values_meta, None);
        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("age")
            "#,
            context,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("rejected value"));
        Ok(())
    }

    #[test]
    fn test_validation_failure_string() -> eyre::Result<()> {
        let lua = Lua::new();
        let mut values = HashMap::new();
        values.insert("age".to_string(), "25".to_string());

        let meta_table: mlua::Table = lua
            .load(
                r#"
            {
                type = "number",
                description = "User age",
                validate = function(v) return "Custom error" end
            }
        "#,
            )
            .eval()?;
        let meta = ValueMeta::from_lua(mlua::Value::Table(meta_table), &lua)?;

        let mut values_meta = HashMap::new();
        values_meta.insert("age".to_string(), meta);

        let context = create_test_context(values, values_meta, None);
        let result = execute_lua_with_context(
            &lua,
            r#"
                values.get("age")
            "#,
            context,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Custom error"));
        Ok(())
    }
}
