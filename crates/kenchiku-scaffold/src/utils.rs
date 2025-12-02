use std::{fs, io::ErrorKind, path::PathBuf};

use eyre::Context;

pub(crate) fn move_files_to_destination(
    source_dir: &PathBuf,
    dest_dir: &PathBuf,
    merge_directories: bool,
    overwrite: bool,
) -> eyre::Result<Vec<PathBuf>> {
    let mut skipped_paths = Vec::new();

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest_dir.join(file_name);

        if source_path.is_dir() && dest_path.exists() && dest_path.is_dir() && merge_directories {
            let skipped =
                move_files_to_destination(&source_path, &dest_path, merge_directories, overwrite)?;
            skipped_paths.extend(skipped);
        } else if dest_path.exists() && !overwrite {
            skipped_paths.push(source_path);
        } else {
            // If overwrite is true and destination exists, remove destination first
            if dest_path.exists() && overwrite {
                if dest_path.is_dir() {
                    fs::remove_dir_all(&dest_path)?;
                } else {
                    fs::remove_file(&dest_path)?;
                }
            }
            fs::rename(&source_path, &dest_path)
                .or_else(|e| {
                    if e.kind() == ErrorKind::CrossesDevices {
                        fs::copy(&source_path, &dest_path)?;
                        fs::remove_file(&source_path)
                    } else {
                        Err(e)
                    }
                })
                .wrap_err(format!(
                    "failed to move file {source_path:?} to {dest_path:?}"
                ))?;
        }
    }

    Ok(skipped_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    fn create_dummy_file(path: &Path, content: &str) -> std::io::Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn test_basic_file_move() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let source_file_path = source_dir.path().join("test.txt");
        create_dummy_file(&source_file_path, "test content")?;

        let dest_file_path = dest_dir.path().join("test.txt");

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            false,
            false,
        )?;
        assert_eq!(skipped.len(), 0);
        assert!(dest_file_path.exists());
        assert!(!source_file_path.exists());
        assert_eq!(fs::read_to_string(dest_file_path)?, "test content");

        Ok(())
    }

    #[test]
    fn test_file_overwrite_skip() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let source_file_path = source_dir.path().join("test.txt");
        create_dummy_file(&source_file_path, "source content")?;

        let dest_file_path = dest_dir.path().join("test.txt");
        create_dummy_file(&dest_file_path, "destination content")?;

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            false,
            false,
        )?;
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0], source_file_path);
        assert!(source_file_path.exists());
        assert_eq!(fs::read_to_string(dest_file_path)?, "destination content");

        Ok(())
    }

    #[test]
    fn test_directory_merge() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let source_subdir = source_dir.path().join("subdir");
        fs::create_dir(&source_subdir)?;
        create_dummy_file(&source_subdir.join("source.txt"), "source file")?;

        let dest_subdir = dest_dir.path().join("subdir");
        fs::create_dir(&dest_subdir)?;
        create_dummy_file(&dest_subdir.join("dest.txt"), "dest file")?;

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            true,
            false,
        )?;
        assert_eq!(skipped.len(), 0);

        assert!(dest_subdir.join("source.txt").exists());
        assert!(dest_subdir.join("dest.txt").exists());
        assert!(!source_subdir.join("source.txt").exists());

        Ok(())
    }

    #[test]
    fn test_move_files_to_destination_empty_source_dir() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            false,
            false,
        )?;

        assert_eq!(skipped.len(), 0);
        Ok(())
    }

    #[test]
    fn test_move_files_to_destination_skips_existing_dir_when_not_merging() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let existing_dir_name = "existing_dir";
        let source_dir_path = source_dir.path().join(existing_dir_name);
        fs::create_dir(&source_dir_path)?;

        let dest_dir_path = dest_dir.path().join(existing_dir_name);
        fs::create_dir(&dest_dir_path)?;

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            false,
            false,
        )?;

        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0], source_dir_path);
        assert!(source_dir_path.exists());
        assert!(dest_dir_path.exists());

        Ok(())
    }

    #[test]
    fn test_file_overwrite_enabled() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let source_file_path = source_dir.path().join("test.txt");
        create_dummy_file(&source_file_path, "source content")?;

        let dest_file_path = dest_dir.path().join("test.txt");
        create_dummy_file(&dest_file_path, "destination content")?;

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            false,
            true,
        )?;
        assert_eq!(skipped.len(), 0);
        assert!(dest_file_path.exists());
        assert!(!source_file_path.exists());
        assert_eq!(fs::read_to_string(dest_file_path)?, "source content");

        Ok(())
    }

    #[test]
    fn test_directory_merge_with_overwrite() -> eyre::Result<()> {
        let source_dir = tempfile::tempdir()?;
        let dest_dir = tempfile::tempdir()?;

        let source_subdir = source_dir.path().join("subdir");
        fs::create_dir(&source_subdir)?;
        create_dummy_file(&source_subdir.join("source.txt"), "source file")?;

        let dest_subdir = dest_dir.path().join("subdir");
        fs::create_dir(&dest_subdir)?;
        create_dummy_file(&dest_subdir.join("dest.txt"), "dest file")?;

        let skipped = move_files_to_destination(
            &source_dir.path().to_path_buf(),
            &dest_dir.path().to_path_buf(),
            true,
            true,
        )?;
        assert_eq!(skipped.len(), 0);

        assert!(dest_subdir.join("source.txt").exists());
        assert!(dest_subdir.join("dest.txt").exists());
        assert!(!source_subdir.join("source.txt").exists());

        Ok(())
    }
}
