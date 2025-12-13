use std::{collections::HashMap, path::PathBuf};

use crate::meta::ValueMeta;

pub mod meta;

#[derive(Clone)]
pub struct Context {
    pub working_dir: PathBuf,
    pub confirm_all: u8,
    pub confirm_fn: fn(message: String) -> eyre::Result<bool>,
    pub output: PathBuf,
    pub scaffold_dir: PathBuf,
    pub allow_overwrite: bool,
    pub values_meta: HashMap<String, ValueMeta>,
    pub values: HashMap<String, String>,
    pub prompt_value: fn(
        r#type: String,
        description: String,
        choices: Option<Vec<String>>,
    ) -> eyre::Result<String>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            confirm_fn: |_message| Ok(true),
            working_dir: Default::default(),
            confirm_all: 0,
            output: Default::default(),
            scaffold_dir: Default::default(),
            allow_overwrite: false,
            values_meta: Default::default(),
            values: Default::default(),
            prompt_value: |_, _, _| Ok("".to_string()),
        }
    }
}
