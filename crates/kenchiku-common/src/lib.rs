use std::path::PathBuf;

#[derive(Clone)]
pub struct Context {
    pub working_dir: PathBuf,
    pub confirm_all: u8,
    pub confirm_fn: fn(message: String) -> eyre::Result<bool>,
    pub output: PathBuf,
    pub allow_overwrite: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            confirm_fn: |_message| Ok(true),
            working_dir: Default::default(),
            confirm_all: 0,
            output: Default::default(),
            allow_overwrite: false,
        }
    }
}
