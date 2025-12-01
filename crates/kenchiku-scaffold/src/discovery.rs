use std::{
    env, fs,
    path::{Path, PathBuf},
};

use tracing::{info, warn};

pub fn discover_scaffold(path_or_name: String) -> Option<PathBuf> {
    if path_or_name.starts_with(".") || path_or_name.starts_with("/") {
        return Some(path_or_name.into());
    }

    let kenchiku_path = env::var("KENCHIKU_PATH").unwrap_or("".to_string());

    let results = find_directories_in_path(&kenchiku_path, &path_or_name);

    if results.len() > 1 {
        warn!(
            name = path_or_name,
            "More than one scaffold found with name"
        );
    }
    if results.len() > 0 {
        let path = results.first().unwrap().into();
        info!(scaffold = path_or_name, ?path, "Found scaffold");
        return Some(path);
    }
    None
}

fn find_directories_in_path(path_env: &str, name: &str) -> Vec<String> {
    let mut found_directories: Vec<String> = Vec::new();
    for path in path_env.split(':') {
        if path.is_empty() {
            continue;
        }
        let full_path = Path::new(path).join(name);
        if let Ok(metadata) = fs::metadata(&full_path) {
            if metadata.is_dir() {
                found_directories.push(full_path.to_string_lossy().to_string());
            }
        }
    }
    found_directories
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_find_directories_in_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_string_lossy().to_string();

        let target_dir_name = "test_dir";
        let target_dir_path = Path::new(&temp_dir_path).join(target_dir_name);
        fs::create_dir_all(&target_dir_path).unwrap();

        let another_temp_dir = tempfile::tempdir().unwrap();
        let another_temp_dir_path = another_temp_dir.path().to_string_lossy().to_string();
        let another_target_dir_path = Path::new(&another_temp_dir_path).join(target_dir_name);
        fs::create_dir_all(&another_target_dir_path).unwrap();

        let path_env = format!("{}:{}", temp_dir_path, another_temp_dir_path);
        env::set_var("KENCHIKU_PATH", &path_env);

        let found_directories = find_directories_in_path(&path_env, target_dir_name);
        assert_eq!(found_directories.len(), 2);
        assert!(found_directories.contains(&target_dir_path.to_string_lossy().to_string()));
        assert!(found_directories.contains(&another_target_dir_path.to_string_lossy().to_string()));
    }

    #[test]
    fn test_find_directories_in_path_empty() {
        let found_directories = find_directories_in_path("", "test_dir");
        assert_eq!(found_directories.len(), 0);
    }

    #[test]
    fn test_find_directories_in_path_no_match() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_string_lossy().to_string();

        env::set_var("KENCHIKU_PATH", &temp_dir_path);

        let found_directories = find_directories_in_path(&temp_dir_path, "non_existent_dir");
        assert_eq!(found_directories.len(), 0);
    }
}
