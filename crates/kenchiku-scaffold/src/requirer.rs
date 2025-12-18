use mlua::prelude::*;
use mlua::{NavigateError, Require};
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

pub(crate) struct SimpleRequirer {
    /// Base directory where modules can be loaded from
    allowed_dir: PathBuf,
    /// Resolved module file (if found)
    module_file: Option<PathBuf>,
}

impl SimpleRequirer {
    pub fn new(allowed_dir: PathBuf) -> Self {
        Self {
            allowed_dir,
            module_file: None,
        }
    }

    /// Check if path is within the allowed directory
    fn is_allowed(&self, path: &Path) -> bool {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        canonical.starts_with(&self.allowed_dir)
    }

    /// Find module file
    fn find_module(&self, path: &Path) -> Option<PathBuf> {
        let file = path.with_extension("lua");
        if file.is_file() && self.is_allowed(&file) {
            return Some(file);
        }
        None
    }
}

impl Require for SimpleRequirer {
    #[tracing::instrument(level = "trace", skip(self), ret)]
    fn is_require_allowed(&self, chunk_name: &str) -> bool {
        self.is_allowed(Path::new(chunk_name))
    }

    #[tracing::instrument(level = "trace", skip(self), ret)]
    fn reset(&mut self, chunk_name: &str) -> Result<(), NavigateError> {
        if let Some(module) = self.find_module(Path::new(chunk_name)) {
            if self.is_allowed(&module) {
                self.module_file = Some(module);
                return Ok(());
            }
        }
        Err(NavigateError::NotFound)
    }

    #[tracing::instrument(level = "trace", skip(self), ret)]
    fn jump_to_alias(&mut self, _path: &str) -> Result<(), NavigateError> {
        Err(NavigateError::NotFound)
    }

    #[tracing::instrument(level = "trace", skip(self), fields(self.module_file), ret)]
    fn to_parent(&mut self) -> Result<(), NavigateError> {
        if let Some(path) = &self.module_file {
            let new_path = path.parent().ok_or(NavigateError::NotFound)?;
            if self.is_allowed(new_path) {
                self.module_file = Some(new_path.to_path_buf());
                return Ok(());
            }
        }
        Err(NavigateError::NotFound)
    }

    #[tracing::instrument(level = "trace", skip(self), ret)]
    fn to_child(&mut self, name: &str) -> Result<(), NavigateError> {
        if let Some(path) = &self.module_file {
            let new_path = &path.join(name).with_extension("lua");
            if self.is_allowed(new_path) {
                self.module_file = Some(new_path.to_path_buf());
                return Ok(());
            }
        }
        Err(NavigateError::NotFound)
    }

    #[tracing::instrument(level = "trace", skip(self), ret)]
    fn has_module(&self) -> bool {
        self.module_file.is_some()
    }

    fn cache_key(&self) -> String {
        self.module_file.as_ref().unwrap().display().to_string()
    }

    fn has_config(&self) -> bool {
        false
    }

    fn config(&self) -> IoResult<Vec<u8>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config not supported",
        ))
    }

    #[tracing::instrument(level = "trace", skip_all, ret)]
    fn loader(&self, lua: &Lua) -> LuaResult<mlua::Function> {
        let module_path = self.module_file.clone().unwrap();
        let module_name = &module_path.display().to_string();

        lua.load(module_path).set_name(module_name).into_function()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        fs::write(base_path.join("simple.lua"), "return { value = 42 }").unwrap();
        fs::write(base_path.join("another.lua"), "return 'hello'").unwrap();

        fs::create_dir_all(base_path.join("nested/deep")).unwrap();
        fs::write(
            base_path.join("nested/module.lua"),
            "return { nested = true }",
        )
        .unwrap();
        fs::write(
            base_path.join("nested/deep/file.lua"),
            "return { deep = true }",
        )
        .unwrap();

        (temp_dir, base_path)
    }

    #[test]
    fn test_new_requirer() {
        let path = PathBuf::from("/tmp/test");
        let requirer = SimpleRequirer::new(path.clone());
        assert_eq!(requirer.allowed_dir, path);
        assert!(requirer.module_file.is_none());
    }

    #[test]
    fn test_is_allowed_inside_directory() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path.clone());

        let allowed_path = base_path.join("simple.lua");
        assert!(requirer.is_allowed(&allowed_path));
    }

    #[test]
    fn test_is_allowed_outside_directory() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path);

        let outside_path = PathBuf::from("/etc/passwd");
        assert!(!requirer.is_allowed(&outside_path));
    }

    #[test]
    fn test_is_allowed_nonexistent_path() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path.clone());

        let nonexistent = base_path.join("doesnt_exist.lua");
        assert!(!requirer.is_allowed(&nonexistent));
    }

    #[test]
    fn test_find_module_exists() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path.clone());

        let module_path = base_path.join("simple");
        let found = requirer.find_module(&module_path);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), base_path.join("simple.lua"));
    }

    #[test]
    fn test_find_module_not_exists() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path.clone());

        let module_path = base_path.join("nonexistent");
        let found = requirer.find_module(&module_path);
        assert!(found.is_none());
    }

    #[test]
    fn test_find_module_nested() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path.clone());

        let module_path = base_path.join("nested/module");
        let found = requirer.find_module(&module_path);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), base_path.join("nested/module.lua"));
    }

    #[test]
    fn test_is_require_allowed_valid_chunk() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path.clone());

        let binding = base_path.join("simple.lua");
        let chunk_name = binding.to_str().unwrap();
        assert!(requirer.is_require_allowed(chunk_name));
    }

    #[test]
    fn test_is_require_allowed_invalid_chunk() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path);

        assert!(!requirer.is_require_allowed("/etc/passwd"));
    }

    #[test]
    fn test_reset_simple_module() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        let binding = base_path.join("simple");
        let chunk_name = binding.to_str().unwrap();
        let result = requirer.reset(chunk_name);

        assert!(result.is_ok());
        assert!(requirer.module_file.is_some());
        assert_eq!(requirer.module_file.unwrap(), base_path.join("simple.lua"));
    }

    #[test]
    fn test_reset_nested_module() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        let binding = base_path.join("nested/module");
        let chunk_name = binding.to_str().unwrap();
        let result = requirer.reset(chunk_name);

        assert!(result.is_ok());
        assert_eq!(
            requirer.module_file.unwrap(),
            base_path.join("nested/module.lua")
        );
    }

    #[test]
    fn test_reset_nonexistent_module() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        let binding = base_path.join("nonexistent");
        let chunk_name = binding.to_str().unwrap();
        let result = requirer.reset(chunk_name);

        assert!(matches!(result, Err(NavigateError::NotFound)));
        assert!(requirer.module_file.is_none());
    }

    #[test]
    fn test_reset_outside_allowed_dir() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path);

        let result = requirer.reset("/etc/passwd");
        assert!(matches!(result, Err(NavigateError::NotFound)));
    }

    #[test]
    fn test_jump_to_alias_not_supported() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path);

        let result = requirer.jump_to_alias("some_alias");
        assert!(matches!(result, Err(NavigateError::NotFound)));
    }

    #[test]
    fn test_to_parent_success() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.join("nested/module.lua"));

        let result = requirer.to_parent();
        assert!(result.is_ok());
        assert_eq!(requirer.module_file.unwrap(), base_path.join("nested"));
    }

    #[test]
    fn test_to_parent_no_module_set() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path);

        let result = requirer.to_parent();
        assert!(matches!(result, Err(NavigateError::NotFound)));
    }

    #[test]
    fn test_to_parent_at_root() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.clone());

        let result = requirer.to_parent();
        // Should fail because parent of base_path might be outside allowed dir
        assert!(matches!(result, Err(NavigateError::NotFound)));
    }

    #[test]
    fn test_to_child_success() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.join("nested"));

        let result = requirer.to_child("module");
        assert!(result.is_ok());
        assert_eq!(
            requirer.module_file.unwrap(),
            base_path.join("nested/module.lua")
        );
    }

    #[test]
    fn test_to_child_no_module_set() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path);

        let result = requirer.to_child("module");
        assert!(matches!(result, Err(NavigateError::NotFound)));
    }

    #[test]
    fn test_to_child_nonexistent() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.clone());

        let result = requirer.to_child("nonexistent");
        assert!(matches!(result, Err(NavigateError::NotFound)));
    }

    #[test]
    fn test_has_module_true() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.join("simple.lua"));
        assert!(requirer.has_module());
    }

    #[test]
    fn test_has_module_false() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path);

        assert!(!requirer.has_module());
    }

    #[test]
    fn test_cache_key() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        let module_path = base_path.join("simple.lua");
        requirer.module_file = Some(module_path.clone());

        let cache_key = requirer.cache_key();
        assert_eq!(cache_key, module_path.display().to_string());
    }

    #[test]
    fn test_has_config_always_false() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path);

        assert!(!requirer.has_config());
    }

    #[test]
    fn test_config_not_supported() {
        let (_temp, base_path) = setup_test_env();
        let requirer = SimpleRequirer::new(base_path);

        let result = requirer.config();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_loader_creates_function() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.join("simple.lua"));

        let lua = Lua::new();
        let result = requirer.loader(&lua);

        assert!(result.is_ok());
        let function = result.unwrap();
        assert_eq!(function.info().name, None);
    }

    #[test]
    fn test_loader_executes_correctly() {
        let (_temp, base_path) = setup_test_env();
        let mut requirer = SimpleRequirer::new(base_path.clone());

        requirer.module_file = Some(base_path.join("simple.lua"));

        let lua = Lua::new();
        let function = requirer.loader(&lua).unwrap();

        let result: LuaResult<mlua::Table> = function.call(());
        assert!(result.is_ok());

        let table = result.unwrap();
        let value: i32 = table.get("value").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic]
    fn test_cache_key_panics_when_no_module() {
        let temp_dir = TempDir::new().unwrap();
        let requirer = SimpleRequirer::new(temp_dir.path().to_path_buf());

        // Should panic because module_file is None
        let _ = requirer.cache_key();
    }

    #[test]
    #[should_panic]
    fn test_loader_panics_when_no_module() {
        let temp_dir = TempDir::new().unwrap();
        let requirer = SimpleRequirer::new(temp_dir.path().to_path_buf());

        let lua = Lua::new();
        // Should panic because module_file is None
        let _ = requirer.loader(&lua);
    }
}
