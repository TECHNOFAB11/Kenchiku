use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::meta::ValueMeta;

pub mod meta;

#[derive(Clone)]
pub struct Context {
    pub working_dir: PathBuf,
    pub confirm_all: u8,
    pub confirm_fn: Arc<dyn Fn(String) -> eyre::Result<bool> + Send + Sync>,
    pub output: PathBuf,
    pub scaffold_dir: PathBuf,
    pub allow_overwrite: bool,
    pub values_meta: HashMap<String, ValueMeta>,
    pub values: HashMap<String, String>,
    pub prompt_value: Arc<
        dyn Fn(String, String, Option<Vec<String>>, Option<String>) -> eyre::Result<String>
            + Send
            + Sync,
    >,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            confirm_fn: Arc::new(|_message| Ok(true)),
            working_dir: Default::default(),
            confirm_all: 0,
            output: Default::default(),
            scaffold_dir: Default::default(),
            allow_overwrite: false,
            values_meta: Default::default(),
            values: Default::default(),
            prompt_value: Arc::new(|_, _, _, _| Ok("".to_string())),
        }
    }
}

pub trait IntoLuaErrDebug<T> {
    fn into_lua_err_debug(self) -> mlua::Result<T>;
}

impl<T> IntoLuaErrDebug<T> for eyre::Result<T> {
    fn into_lua_err_debug(self) -> mlua::Result<T> {
        self.map_err(|e| mlua::Error::external(format!("{:?}", e)))
    }
}
