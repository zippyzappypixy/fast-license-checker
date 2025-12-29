//! File scanning functionality.
//!
//! Provides the main Scanner interface that coordinates file walking,
//! content filtering, and license header checking.

pub mod filter;
pub mod walker;

use std::path::Path;
use std::time::Instant;

use rayon::iter::ParallelIterator;

use crate::checker::HeaderChecker;
use crate::config::Config;
use crate::error::{Result, ScannerError};
use crate::types::{FilePath, ScanResult, ScanSummary};

use self::filter::should_process_file;
use self::walker::{FileWalker, WalkEntry};

/// Main scanner that coordinates walking and checking
#[derive(Debug)]
pub struct Scanner {
    walker: FileWalker,
    checker: HeaderChecker,
    config: Config,
}

impl Scanner {
    /// Create a new scanner for the given root directory
    #[tracing::instrument(skip(config))]
    pub fn new(root: impl AsRef<Path> + std::fmt::Debug, config: Config) -> Result<Self> {
        // Validate that the root directory exists and is readable
        let root_path = root.as_ref();
        if !root_path.exists() {
            return Err(crate::error::LicenseCheckerError::Scanner(ScannerError::Io {
                path: root_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Root directory does not exist",
                ),
            }));
        }

        if !root_path.is_dir() {
            return Err(crate::error::LicenseCheckerError::Scanner(ScannerError::Io {
                path: root_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "Root path is not a directory",
                ),
            }));
        }

        let walker = FileWalker::new(root_path)
            .with_ignores(config.ignore_patterns.clone())
            .with_parallelism(config.parallel_jobs.unwrap_or_else(|| num_cpus::get()));

        // Create header checker for actual header detection
        let checker = HeaderChecker::new(&config)?;

        Ok(Self { walker, checker, config })
    }

    /// Scan all files and return results
    #[tracing::instrument(skip(self))]
    pub fn scan(&self) -> Result<ScanSummary> {
        let start = Instant::now();

        // Walk files and process them in parallel
        let results: Vec<ScanResult> = self
            .walker
            .walk()
            .filter_map(|entry_result| {
                match entry_result {
                    Ok(entry) => Some(self.check_file(&entry)),
                    Err(e) => {
                        // Log the error but continue processing
                        tracing::warn!("Error walking directory entry: {}", e);
                        None
                    }
                }
            })
            .collect();

        let duration = start.elapsed();
        let summary = ScanSummary::new(
            results.len(),
            results.iter().filter(|r| r.status.has_valid_header()).count(),
            results.iter().filter(|r| r.status.is_missing_header()).count(),
            results.iter().filter(|r| r.status.is_skipped()).count(),
            duration,
        );

        tracing::info!("Scan completed: {} files in {:.2}s", summary.total, duration.as_secs_f64());

        Ok(summary)
    }

    /// Check a single file and return the result
    #[tracing::instrument(skip(self, entry))]
    fn check_file(&self, entry: &WalkEntry) -> ScanResult {
        let file_path = match FilePath::new_existing(entry.path.clone()) {
            Ok(fp) => fp,
            Err(_) => {
                // File no longer exists or is not accessible
                return ScanResult::new(
                    FilePath::new(entry.path.clone()),
                    crate::types::FileStatus::Skipped {
                        reason: crate::types::SkipReason::Gitignored, // Generic skip reason
                    },
                );
            }
        };

        // Read file content
        let content = match self.read_file_content(&entry.path) {
            Ok(content) => content,
            Err(_) => {
                // Could not read file
                return ScanResult::new(
                    file_path,
                    crate::types::FileStatus::Skipped {
                        reason: crate::types::SkipReason::Gitignored, // Generic skip reason
                    },
                );
            }
        };

        // Check if file should be processed
        let extension = entry.extension();
        match should_process_file(&content, extension, &self.config) {
            Ok(_) => {
                // File should be processed - check license header using HeaderChecker
                let status = self.checker.check_content(&content, extension);
                ScanResult::new(file_path, status)
            }
            Err(reason) => {
                // File should be skipped
                ScanResult::new(file_path, crate::types::FileStatus::Skipped { reason })
            }
        }
    }

    /// Read file content up to the configured maximum bytes
    #[tracing::instrument]
    fn read_file_content(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        use std::fs::File;
        use std::io::Read;

        let file = File::open(path)?;
        let mut buffer = Vec::new();

        // Read up to max_header_bytes + some buffer for safety
        // Use checked_add to prevent overflow (though unlikely in practice)
        let max_read =
            self.config.max_header_bytes.checked_add(1024).unwrap_or(self.config.max_header_bytes);
        file.take(max_read as u64).read_to_end(&mut buffer)?;

        // Truncate to the configured maximum
        if buffer.len() > self.config.max_header_bytes {
            buffer.truncate(self.config.max_header_bytes);
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Tests are allowed to use unwrap() for test setup
    #[allow(clippy::unwrap_used, clippy::expect_used, clippy::arithmetic_side_effects)]
    #[test]
    fn scanner_new_valid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        // HeaderChecker requires a valid license header
        config.license_header = "MIT License\nCopyright 2024".to_string();

        let scanner = Scanner::new(&temp_dir, config);
        assert!(scanner.is_ok());
    }

    #[test]
    fn scanner_new_nonexistent_directory() {
        let config = Config::default();
        let result = Scanner::new("/nonexistent/path", config);
        assert!(result.is_err());
    }

    #[test]
    fn scanner_new_file_instead_of_directory() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let config = Config::default();

        let result = Scanner::new(temp_file.path(), config);
        assert!(result.is_err());
    }

    #[test]
    fn scanner_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.license_header = "MIT License\nCopyright 2024".to_string();

        let scanner = Scanner::new(&temp_dir, config).unwrap();
        let summary = scanner.scan().unwrap();

        assert_eq!(summary.total, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.skipped, 0);
    }

    #[test]
    fn scanner_scan_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.license_header = "MIT License\nCopyright 2024".to_string();

        // Create a test file with content
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() { println!(\"Hello World\"); }").unwrap();

        // Verify the file was created
        assert!(test_file.exists());

        let scanner = Scanner::new(&temp_dir, config).unwrap();
        let summary = scanner.scan().unwrap();

        assert_eq!(summary.total, 1);
        // File without header should be marked as missing header
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn scanner_scan_with_license_header() {
        let mut config = Config::default();
        // Use raw header text (without comment markers) - HeaderChecker will format it
        config.license_header = "MIT License\n\nCopyright 2024".to_string();
        // Add comment style for .rs files
        use crate::config::CommentStyleConfig;
        config.comment_styles.insert(
            "rs".to_string(),
            CommentStyleConfig { prefix: "//".to_string(), suffix: None },
        );

        let temp_dir = TempDir::new().unwrap();

        // Create a test file with the formatted license header
        let test_file = temp_dir.path().join("test.rs");
        let content = "// MIT License\n\n// Copyright 2024\n\nfn main() {}\n";
        fs::write(&test_file, content).unwrap();

        let scanner = Scanner::new(&temp_dir, config).unwrap();
        let summary = scanner.scan().unwrap();

        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 1); // Should have valid header
    }

    #[test]
    fn scanner_skip_empty_files() {
        let mut config = Config::default();
        config.skip_empty_files = true;
        config.license_header = "MIT License\nCopyright 2024".to_string();

        let temp_dir = TempDir::new().unwrap();

        // Create an empty file
        let empty_file = temp_dir.path().join("empty.txt");
        fs::write(&empty_file, "").unwrap();

        let scanner = Scanner::new(&temp_dir, config).unwrap();
        let summary = scanner.scan().unwrap();

        assert_eq!(summary.total, 1);
        assert_eq!(summary.skipped, 1);
    }

    #[test]
    fn scanner_skip_binary_files() {
        let mut config = Config::default();
        config.license_header = "MIT License\nCopyright 2024".to_string();
        let temp_dir = TempDir::new().unwrap();

        // Create a binary file (with null bytes)
        let binary_file = temp_dir.path().join("binary.bin");
        fs::write(&binary_file, &[0x00, 0x01, 0x02, 0x00]).unwrap();

        let scanner = Scanner::new(&temp_dir, config).unwrap();
        let summary = scanner.scan().unwrap();

        assert_eq!(summary.total, 1);
        assert_eq!(summary.skipped, 1);
    }

    #[test]
    fn scanner_skip_unknown_extensions() {
        let mut config = Config::default();
        config.license_header = "MIT License\nCopyright 2024".to_string();
        let temp_dir = TempDir::new().unwrap();

        // Create a file with unknown extension
        let unknown_file = temp_dir.path().join("test.unknown");
        fs::write(&unknown_file, "some content").unwrap();

        let scanner = Scanner::new(&temp_dir, config).unwrap();
        let summary = scanner.scan().unwrap();

        assert_eq!(summary.total, 1);
        assert_eq!(summary.skipped, 1);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::scanner::filter::{is_binary, is_valid_utf8};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn binary_detection_never_panics(content in prop::collection::vec(0u8..255u8, 0..1000)) {
            let _ = is_binary(&content);
        }

        #[test]
        fn utf8_validation_never_panics(content in prop::collection::vec(0u8..255u8, 0..1000)) {
            let _ = is_valid_utf8(&content);
        }

        #[test]
        fn binary_files_always_skipped(
            content in prop::collection::vec(0u8..255u8, 100..1000)
        ) {
            // If content has null bytes, should be detected as binary
            if content.contains(&0u8) {
                assert!(is_binary(&content));
            }
        }
    }
}
