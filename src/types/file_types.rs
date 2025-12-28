//! File-related domain types.
//!
//! Types that represent file system concepts with validation and type safety.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::ValidationError;

/// A validated file path wrapper.
///
/// Provides type safety and optional validation for file system paths.
/// In scan mode, paths may not exist yet. In fix mode, we validate existence.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FilePath(PathBuf);

impl FilePath {
    /// Creates a new FilePath from a PathBuf without validation.
    ///
    /// Use this for scan operations where the file might not exist yet.
    /// For fix operations, use `new_existing()` instead.
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Creates a new FilePath, validating that the path exists and is a file.
    ///
    /// # Errors
    /// Returns an error if the path does not exist, is not a file, or is not accessible.
    pub fn new_existing(path: PathBuf) -> Result<Self, FilePathError> {
        if !path.exists() {
            return Err(FilePathError::NotFound(path));
        }
        if !path.is_file() {
            return Err(FilePathError::NotAFile(path));
        }
        Ok(Self(path))
    }

    /// Returns a reference to the underlying Path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Returns the file name as an Option<&str>.
    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name()?.to_str()
    }

    /// Returns the file extension as a FileExtension if present.
    pub fn extension(&self) -> Option<FileExtension> {
        self.0
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| FileExtension::new(ext.to_string()).ok())
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<FilePath> for PathBuf {
    fn from(fp: FilePath) -> PathBuf {
        fp.0
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// Errors that can occur when working with file paths.
#[derive(Debug, thiserror::Error)]
pub enum FilePathError {
    /// The specified path does not exist.
    #[error("Path does not exist: {0}")]
    NotFound(PathBuf),
    /// The specified path exists but is not a file.
    #[error("Path exists but is not a file: {0}")]
    NotAFile(PathBuf),
}

/// A validated file extension (lowercase, no leading dot).
///
/// Ensures consistent handling of file extensions across the codebase.
/// Extensions are normalized to lowercase and never contain a leading dot.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileExtension(String);

impl FileExtension {
    /// Creates a new FileExtension from a string.
    ///
    /// Automatically converts to lowercase and removes leading dots.
    ///
    /// # Errors
    /// Returns an error if the extension is empty after processing.
    pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
        let s = s.into();
        let ext = s.trim_start_matches('.').trim().to_lowercase();
        if ext.is_empty() {
            return Err(ValidationError::EmptyExtension);
        }
        if ext.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != '+' && c != '#') {
            return Err(ValidationError::InvalidExtension);
        }
        Ok(Self(ext))
    }

    /// Returns the extension as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FileExtension {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FileExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Maximum bytes to read from file start for header detection.
///
/// Limits memory usage and prevents reading entire large files.
/// The minimum value ensures we can detect reasonable license headers.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MaxHeaderBytes(usize);

impl MaxHeaderBytes {
    /// The minimum allowed value (256 bytes).
    pub const MIN_VALUE: usize = 256;

    /// The default value (8KB).
    pub const DEFAULT: Self = Self(8192);

    /// Creates a new MaxHeaderBytes with validation.
    ///
    /// # Errors
    /// Returns an error if the value is less than the minimum.
    pub fn new(value: usize) -> Result<Self, ValidationError> {
        if value < Self::MIN_VALUE {
            return Err(ValidationError::HeaderBytesTooSmall(value));
        }
        Ok(Self(value))
    }

    /// Returns the value as usize.
    pub fn value(&self) -> usize {
        self.0
    }
}

impl Default for MaxHeaderBytes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<MaxHeaderBytes> for usize {
    fn from(mhb: MaxHeaderBytes) -> usize {
        mhb.0
    }
}

impl std::fmt::Display for MaxHeaderBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // FilePath tests
    #[test]
    fn file_path_new() {
        let path = PathBuf::from("/tmp/test.txt");
        let fp = FilePath::new(path.clone());
        assert_eq!(fp.as_path(), &path);
    }

    #[test]
    fn file_path_new_existing_success() {
        // Create a temporary file for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let result = FilePath::new_existing(test_file.clone());
        assert!(result.is_ok());
        let fp = result.unwrap();
        assert_eq!(fp.as_path(), &test_file);
    }

    #[test]
    fn file_path_new_existing_not_found() {
        let nonexistent = PathBuf::from("/this/path/does/not/exist.txt");
        let result = FilePath::new_existing(nonexistent.clone());
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), FilePathError::NotFound(path) if path == nonexistent)
        );
    }

    #[test]
    fn file_path_new_existing_directory() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let result = FilePath::new_existing(temp_dir.path().to_path_buf());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FilePathError::NotAFile(_)));
    }

    #[test]
    fn file_path_file_name() {
        let fp = FilePath::new(PathBuf::from("/tmp/test.txt"));
        assert_eq!(fp.file_name(), Some("test.txt"));

        let fp_no_name = FilePath::new(PathBuf::from("/"));
        assert_eq!(fp_no_name.file_name(), None);
    }

    #[test]
    fn file_path_extension() {
        let fp = FilePath::new(PathBuf::from("test.txt"));
        let ext = fp.extension().unwrap();
        assert_eq!(ext.as_str(), "txt");

        let fp_no_ext = FilePath::new(PathBuf::from("test"));
        assert!(fp_no_ext.extension().is_none());

        let fp_upper = FilePath::new(PathBuf::from("test.RS"));
        let ext_upper = fp_upper.extension().unwrap();
        assert_eq!(ext_upper.as_str(), "rs"); // Should be normalized to lowercase
    }

    #[test]
    fn file_path_display() {
        let fp = FilePath::new(PathBuf::from("/tmp/test.txt"));
        assert_eq!(format!("{}", fp), "/tmp/test.txt");
    }

    #[test]
    fn file_path_as_ref_path() {
        let path = PathBuf::from("/tmp/test.txt");
        let fp = FilePath::new(path.clone());
        let path_ref: &Path = fp.as_ref();
        assert_eq!(path_ref, path.as_path());
    }

    #[test]
    fn file_path_from_pathbuf() {
        let path = PathBuf::from("/tmp/test.txt");
        let fp = FilePath::new(path.clone());
        let back_to_pathbuf: PathBuf = fp.into();
        assert_eq!(back_to_pathbuf, path);
    }

    // FileExtension tests
    #[test]
    fn file_extension_new() {
        let ext = FileExtension::new("RS").unwrap();
        assert_eq!(ext.as_str(), "rs");

        let ext_dot = FileExtension::new(".txt").unwrap();
        assert_eq!(ext_dot.as_str(), "txt");
    }

    #[test]
    fn file_extension_new_with_underscore() {
        let ext = FileExtension::new("test_file").unwrap();
        assert_eq!(ext.as_str(), "test_file");
    }

    #[test]
    fn file_extension_empty_error() {
        assert!(matches!(FileExtension::new("").unwrap_err(), ValidationError::EmptyExtension));
        assert!(matches!(FileExtension::new(".").unwrap_err(), ValidationError::EmptyExtension));
        assert!(matches!(FileExtension::new("   ").unwrap_err(), ValidationError::EmptyExtension));
    }

    #[test]
    fn file_extension_invalid_character_error() {
        // Test various invalid characters (excluding now-allowed + and #)
        let invalid_cases = vec!["file$", "test-file", "file.ext!", "test@ext", "file%ext"];

        for invalid in invalid_cases {
            let result = FileExtension::new(invalid);
            assert!(result.is_err(), "Expected error for: {}", invalid);
            assert!(matches!(result.unwrap_err(), ValidationError::InvalidExtension));
        }
    }

    #[test]
    fn file_extension_valid_cases() {
        let valid_cases = vec![
            "txt", "rs", "py", "js", "html", "css", "md", "toml", "yaml", "123", "a", "test123",
            "my_ext", "file_123",
        ];

        for valid in valid_cases {
            let result = FileExtension::new(valid);
            assert!(result.is_ok(), "Expected success for: {}", valid);
        }
    }

    #[test]
    fn file_extension_display() {
        let ext = FileExtension::new("txt").unwrap();
        assert_eq!(format!("{}", ext), "txt");
    }

    #[test]
    fn file_extension_as_ref_str() {
        let ext = FileExtension::new("txt").unwrap();
        let str_ref: &str = ext.as_ref();
        assert_eq!(str_ref, "txt");
    }

    #[test]
    fn file_extension_symbols() {
        // Test extensions with common programming symbols
        let symbol_cases = vec![
            ("c++", true),        // C++ extension
            ("c#", true),         // C# extension (though typically .cs)
            ("h++", true),        // C++ header variant
            ("file-name", false), // Hyphens should fail
            ("file$ext", false),  // Dollar signs should fail
            ("file!ext", false),  // Exclamation marks should fail
        ];

        for (ext, should_succeed) in symbol_cases {
            let result = FileExtension::new(ext);
            if should_succeed {
                assert!(result.is_ok(), "Expected success for symbol extension: {}", ext);
            } else {
                assert!(result.is_err(), "Expected failure for invalid extension: {}", ext);
                assert!(matches!(result.unwrap_err(), ValidationError::InvalidExtension));
            }
        }
    }

    #[test]
    fn file_path_invalid_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // Create a path with invalid UTF-8 bytes (Unix-specific)
        #[cfg(unix)]
        {
            let invalid_bytes = b"\xFF\xFE\xFD";
            let os_str = OsStr::from_bytes(invalid_bytes);
            let path = PathBuf::from(os_str);

            let fp = FilePath::new(path);
            // file_name() should return None for invalid UTF-8
            assert_eq!(fp.file_name(), None);
        }

        // On non-Unix systems, we can't easily create invalid UTF-8 paths
        // so we just ensure the method doesn't panic
        #[cfg(not(unix))]
        {
            let fp = FilePath::new(PathBuf::from("/tmp/test.txt"));
            let _ = fp.file_name(); // Should not panic
        }
    }

    // MaxHeaderBytes tests
    #[test]
    fn max_header_bytes_new() {
        let mhb = MaxHeaderBytes::new(4096).unwrap();
        assert_eq!(mhb.value(), 4096);
    }

    #[test]
    fn max_header_bytes_too_small() {
        let result = MaxHeaderBytes::new(100);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::HeaderBytesTooSmall(100)));
    }

    #[test]
    fn max_header_bytes_minimum() {
        let mhb = MaxHeaderBytes::new(256).unwrap();
        assert_eq!(mhb.value(), 256);
    }

    #[test]
    fn max_header_bytes_large_value() {
        let mhb = MaxHeaderBytes::new(1048576).unwrap(); // 1MB
        assert_eq!(mhb.value(), 1048576);
    }

    #[test]
    fn max_header_bytes_default() {
        let default = MaxHeaderBytes::default();
        assert_eq!(default.value(), 8192);
    }

    #[test]
    fn max_header_bytes_display() {
        let mhb = MaxHeaderBytes::new(4096).unwrap();
        assert_eq!(format!("{}", mhb), "4096 bytes");
    }

    #[test]
    fn max_header_bytes_from_usize() {
        let mhb = MaxHeaderBytes::new(8192).unwrap();
        let value: usize = mhb.into();
        assert_eq!(value, 8192);
    }
}
