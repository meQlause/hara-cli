use std::fs;
use std::path::Path;

/// Writes a string content to a file, with an optional overwrite flag.
pub fn write_file(path: &str, content: &str, overwrite: bool) -> Result<(), String> {
    let p = Path::new(path);

    if p.exists() {
        if overwrite {
            tracing::warn!("Overwriting : {}", path);
        } else {
            tracing::info!("  –  Skipping    : {} (already exists)", path);
            return Ok(());
        }
    }

    fs::write(p, content)
        .map_err(|e| format!("Failed to write '{}': {}", path, e))?;

    tracing::info!("Created     : {}", path);
    Ok(())
}

/// Helper function to write a file only if it does not already exist.
pub fn write_if_missing(path: &str, content: &str) -> Result<(), String> {
    write_file(path, content, false)
}

/// Ensures that a list of directory paths exist, creating them if necessary.
pub fn ensure_dirs(dirs: &[&str]) -> Result<(), String> {
    for dir in dirs {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory '{}': {}", dir, e))?;
    }
    Ok(())
}

/// Resets (deletes) a list of directories if they exist.
pub fn reset_dirs(dirs: &[&str]) -> Result<(), String> {
    for dir in dirs {
        let p = Path::new(dir);
        if p.exists() && p.is_dir() {
            fs::remove_dir_all(p)
                .map_err(|e| format!("Failed to reset directory '{}': {}", dir, e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let path_str = file_path.to_str().unwrap();

        write_file(path_str, "hello", false).unwrap();
        assert_eq!(fs::read_to_string(path_str).unwrap(), "hello");

        write_file(path_str, "world", false).unwrap();
        assert_eq!(fs::read_to_string(path_str).unwrap(), "hello");

        write_file(path_str, "world", true).unwrap();
        assert_eq!(fs::read_to_string(path_str).unwrap(), "world");
    }

    #[test]
    fn test_ensure_dirs() {
        let dir = tempdir().unwrap();
        let sub1 = dir.path().join("src");
        let sub2 = dir.path().join("test/sub");
        
        let dirs = vec![
            sub1.to_str().unwrap(),
            sub2.to_str().unwrap()
        ];

        ensure_dirs(&dirs).unwrap();
        assert!(sub1.exists() && sub1.is_dir());
        assert!(sub2.exists() && sub2.is_dir());
    }

    #[test]
    fn test_reset_dirs() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("file.sol"), "content").unwrap();

        let src_str = src.to_str().unwrap();
        reset_dirs(&[src_str]).unwrap();

        assert!(!src.exists()); 
    }
}
