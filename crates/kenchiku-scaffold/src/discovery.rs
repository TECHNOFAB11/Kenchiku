use std::{
    env,
    fs::{self, read_dir},
    path::{Path, PathBuf},
};

use tracing::{debug, info, warn};

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

fn find_directories_in_path(path_env: &str, name: &str) -> Vec<PathBuf> {
    let mut found_directories: Vec<PathBuf> = Vec::new();
    for path in path_env.split(':') {
        if path.is_empty() {
            continue;
        }
        let full_path = Path::new(path).join(name);
        if let Ok(metadata) = fs::metadata(&full_path) {
            if metadata.is_dir() {
                found_directories.push(full_path);
            }
        }
    }
    found_directories
}

pub fn find_all_scaffolds() -> Vec<PathBuf> {
    let kenchiku_path = env::var("KENCHIKU_PATH").unwrap_or("".to_string());
    find_scaffold_directories_in_path(&kenchiku_path)
}

fn find_scaffold_directories_in_path(path_env: &str) -> Vec<PathBuf> {
    debug!(path_env, "Finding all scaffolds in path");
    let mut found_directories: Vec<PathBuf> = Vec::new();
    for path in path_env.split(':') {
        if path.is_empty() {
            continue;
        }
        for scaffold_path in read_dir(path).expect("to read directory") {
            if let Ok(scaffold_path) = scaffold_path {
                let full_path = scaffold_path.path().join("scaffold.lua");
                if let Ok(metadata) = fs::metadata(&full_path) {
                    if metadata.is_file() {
                        found_directories.push(scaffold_path.path().to_path_buf());
                    }
                }
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
    use tempfile::tempdir;

    #[test]
    fn test_find_directories_in_path() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_string_lossy().to_string();

        let target_dir_name = "test_dir";
        let target_dir_path = Path::new(&temp_dir_path).join(target_dir_name);
        fs::create_dir_all(&target_dir_path).unwrap();

        let another_temp_dir = tempdir().unwrap();
        let another_temp_dir_path = another_temp_dir.path().to_string_lossy().to_string();
        let another_target_dir_path = Path::new(&another_temp_dir_path).join(target_dir_name);
        fs::create_dir_all(&another_target_dir_path).unwrap();

        let path_env = format!("{}:{}", temp_dir_path, another_temp_dir_path);
        env::set_var("KENCHIKU_PATH", &path_env);

        let found_directories = find_directories_in_path(&path_env, target_dir_name);
        assert_eq!(found_directories.len(), 2);
        assert!(found_directories.contains(&target_dir_path));
        assert!(found_directories.contains(&another_target_dir_path));
    }

    #[test]
    fn test_find_directories_in_path_empty() {
        let found_directories = find_directories_in_path("", "test_dir");
        assert_eq!(found_directories.len(), 0);
    }

    #[test]
    fn test_find_directories_in_path_no_match() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_string_lossy().to_string();

        env::set_var("KENCHIKU_PATH", &temp_dir_path);

        let found_directories = find_directories_in_path(&temp_dir_path, "non_existent_dir");
        assert_eq!(found_directories.len(), 0);
    }

    #[test]
    fn test_find_scaffold_directories_in_path_found() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create a scaffold.lua file in the temporary directory
        let scaffold_file_path = Path::new(&temp_dir_path).join("test/scaffold.lua");
        fs::create_dir_all(scaffold_file_path.parent().unwrap()).unwrap();
        fs::File::create(&scaffold_file_path).unwrap();

        env::set_var("KENCHIKU_PATH", &temp_dir_path);

        let found_directories = find_scaffold_directories_in_path(&temp_dir_path);
        assert_eq!(found_directories.len(), 1);
        assert_eq!(found_directories[0], format!("{}/test", temp_dir_path));
    }

    #[test]
    fn test_find_scaffold_directories_in_path_not_found() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_string_lossy().to_string();

        env::set_var("KENCHIKU_PATH", &temp_dir_path);

        let found_directories = find_scaffold_directories_in_path(&temp_dir_path);
        assert_eq!(found_directories.len(), 0);
    }

    #[test]
    fn test_find_scaffold_directories_in_path_multiple_paths() {
        let temp_dir1 = tempdir().unwrap();
        let temp_dir_path1 = temp_dir1.path().to_string_lossy().to_string();
        let temp_dir2 = tempdir().unwrap();
        let temp_dir_path2 = temp_dir2.path().to_string_lossy().to_string();

        let scaffold_file_path1 = Path::new(&temp_dir_path1).join("test/scaffold.lua");
        fs::create_dir_all(scaffold_file_path1.parent().unwrap()).unwrap();
        fs::File::create(&scaffold_file_path1).unwrap();

        let path_env = format!("{}:{}", temp_dir_path1, temp_dir_path2);
        env::set_var("KENCHIKU_PATH", &path_env);

        let found_directories = find_scaffold_directories_in_path(&path_env);
        assert_eq!(found_directories.len(), 1);
        assert_eq!(found_directories[0], format!("{}/test", temp_dir_path1));
    }

    #[test]
    fn test_find_scaffold_directories_in_path_empty_path() {
        let found_directories = find_scaffold_directories_in_path("");
        assert_eq!(found_directories.len(), 0);
    }

    #[test]
    fn test_find_scaffold_directories_in_path_multiple_scaffolds() {
        let temp_dir1 = tempdir().unwrap();
        let temp_dir_path1 = temp_dir1.path().to_string_lossy().to_string();
        let temp_dir2 = tempdir().unwrap();
        let temp_dir_path2 = temp_dir2.path().to_string_lossy().to_string();

        let scaffold_file_path1 = Path::new(&temp_dir_path1).join("test/scaffold.lua");
        fs::create_dir_all(scaffold_file_path1.parent().unwrap()).unwrap();
        fs::File::create(&scaffold_file_path1).unwrap();
        let scaffold_file_path2 = Path::new(&temp_dir_path2).join("other/scaffold.lua");
        fs::create_dir_all(scaffold_file_path2.parent().unwrap()).unwrap();
        fs::File::create(&scaffold_file_path2).unwrap();

        let path_env = format!("{}:{}", temp_dir_path1, temp_dir_path2);
        env::set_var("KENCHIKU_PATH", &path_env);

        let found_directories = find_scaffold_directories_in_path(&path_env);
        assert_eq!(found_directories.len(), 2);
        assert!(found_directories.contains(&temp_dir1.path().join("test").to_path_buf()));
        assert!(found_directories.contains(&temp_dir2.path().join("other").to_path_buf()));
    }
}
