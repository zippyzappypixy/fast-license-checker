//! Atomic file writing operations.
//!
//! Provides safe file writing using temporary files and atomic rename
//! to prevent corruption if the process is interrupted during writing.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::error::{FixerError, Result};

/// Write content to file atomically (write temp, then rename)
#[tracing::instrument(skip(content))]
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| FixerError::WriteError {
        path: path.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path has no parent directory",
        ),
    })?;

    // Create temp file in same directory (for atomic rename)
    let temp_path = parent
        .join(format!(".{}.tmp", path.file_name().and_then(|n| n.to_str()).unwrap_or("file")));

    // Write to temp file
    {
        let mut file = File::create(&temp_path)
            .map_err(|e| FixerError::WriteError { path: temp_path.clone(), source: e })?;

        file.write_all(content)
            .map_err(|e| FixerError::WriteError { path: temp_path.clone(), source: e })?;

        file.sync_all()
            .map_err(|e| FixerError::WriteError { path: temp_path.clone(), source: e })?;
    }

    // Atomic rename
    fs::rename(&temp_path, path)
        .map_err(|e| FixerError::WriteError { path: path.to_path_buf(), source: e })?;

    tracing::info!(path = %path.display(), "Fixed file");

    Ok(())
}

/// Write content to file with backup (create .bak file)
#[tracing::instrument(skip(content))]
pub fn write_with_backup(path: &Path, content: &[u8]) -> Result<()> {
    let backup_path = path
        .with_extension(format!("{}.bak", path.extension().and_then(|e| e.to_str()).unwrap_or("")));

    // Create backup of original file
    if path.exists() {
        fs::copy(path, &backup_path)
            .map_err(|e| FixerError::WriteError { path: backup_path.clone(), source: e })?;
    }

    // Write new content
    write_atomic(path, content)?;

    // Remove backup on success
    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path); // Ignore errors on cleanup
    }

    Ok(())
}

/// Check if file is writable
#[tracing::instrument]
pub fn is_writable(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        !metadata.permissions().readonly()
    } else {
        // If file doesn't exist, check if parent directory is writable
        if let Some(parent) = path.parent() {
            fs::metadata(parent).map(|m| !m.permissions().readonly()).unwrap_or(false)
        } else {
            false
        }
    }
}

/// Get file size before writing (for undo operations)
#[tracing::instrument]
pub fn get_file_size(path: &Path) -> Result<u64> {
    fs::metadata(path).map(|m| m.len()).map_err(|e| {
        crate::error::LicenseCheckerError::Fixer(FixerError::WriteError {
            path: path.to_path_buf(),
            source: e,
        })
    })
}

/// Validate that content can be safely written
#[tracing::instrument(skip(content))]
pub fn validate_content(content: &[u8]) -> Result<()> {
    // Basic validation - ensure content is valid UTF-8 if it appears to be text
    if content.len() > 0 && content.len() <= 1024 {
        // For small files, validate UTF-8
        if std::str::from_utf8(content).is_err() {
            return Err(crate::error::LicenseCheckerError::Fixer(FixerError::WriteError {
                path: Path::new("<content>").to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Content is not valid UTF-8",
                ),
            }));
        }
    }

    // Check for reasonable file size (prevent accidental huge files)
    if content.len() > 100 * 1024 * 1024 {
        // 100MB limit
        return Err(crate::error::LicenseCheckerError::Fixer(FixerError::WriteError {
            path: Path::new("<content>").to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Content is too large (>100MB)",
            ),
        }));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn write_atomic_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = b"Hello, World!";

        write_atomic(&file_path, content).unwrap();

        let read_content = fs::read(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn write_atomic_creates_temp_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = b"test content";

        write_atomic(&file_path, content).unwrap();

        // Temp file should not exist after successful write
        let temp_path = temp_dir.path().join(".test.txt.tmp");
        assert!(!temp_path.exists());

        // Final file should exist
        assert!(file_path.exists());
    }

    #[test]
    fn write_with_backup_creates_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create original file
        let original_content = b"original content";
        fs::write(&file_path, original_content).unwrap();

        // Write with backup
        let new_content = b"new content";
        write_with_backup(&file_path, new_content).unwrap();

        // File should have new content
        let read_content = fs::read(&file_path).unwrap();
        assert_eq!(read_content, new_content);

        // Backup should be cleaned up
        let backup_path = file_path.with_extension("txt.bak");
        assert!(!backup_path.exists());
    }

    #[test]
    fn write_atomic_no_parent_directory() {
        let result = write_atomic(Path::new("nonexistent/file.txt"), b"content");
        assert!(result.is_err());
    }

    #[test]
    fn is_writable_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "content").unwrap();

        // In most test environments, files should be writable
        assert!(is_writable(&file_path));
    }

    #[test]
    fn is_writable_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        // Should be writable since parent directory exists
        assert!(is_writable(&file_path));
    }

    #[test]
    fn get_file_size_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = b"Hello, World!"; // 13 bytes
        fs::write(&file_path, content).unwrap();

        let size = get_file_size(&file_path).unwrap();
        assert_eq!(size, 13);
    }

    #[test]
    fn get_file_size_nonexistent() {
        let result = get_file_size(Path::new("nonexistent.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn validate_content_valid_utf8() {
        let content = "Hello, 世界 🌍".as_bytes();
        assert!(validate_content(content).is_ok());
    }

    #[test]
    fn validate_content_invalid_utf8() {
        let content = &[0xff, 0xfe, 0xfd]; // Invalid UTF-8
        let result = validate_content(content);
        // Small files are validated for UTF-8, so this should fail
        assert!(result.is_err());
    }

    #[test]
    fn validate_content_too_large() {
        let content = vec![0u8; 101 * 1024 * 1024]; // 101MB
        let result = validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn validate_content_empty() {
        let content = b"";
        assert!(validate_content(content).is_ok());
    }
}
