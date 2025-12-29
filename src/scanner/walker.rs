//! File walking functionality with ignore support.
//!
//! Provides parallel file walking that respects .gitignore and other ignore patterns.

use ignore::{DirEntry, WalkBuilder, WalkState};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use crate::error::ScannerError;

/// File walker that respects .gitignore and provides parallel iteration
#[derive(Debug)]
pub struct FileWalker {
    root: PathBuf,
    additional_ignores: Vec<String>,
    parallel_jobs: usize,
}

impl FileWalker {
    /// Create a new file walker starting from the given root directory.
    ///
    /// Defaults to using all available CPU cores for parallel processing.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            additional_ignores: Vec::new(),
            parallel_jobs: num_cpus::get(),
        }
    }

    /// Add additional ignore patterns beyond .gitignore
    pub fn with_ignores(mut self, patterns: Vec<String>) -> Self {
        self.additional_ignores = patterns;
        self
    }

    /// Set the number of parallel jobs for file walking
    pub fn with_parallelism(mut self, jobs: usize) -> Self {
        self.parallel_jobs = jobs.max(1); // Ensure at least 1 job
        self
    }

    /// Walk all files, yielding WalkEntry for each valid file
    #[tracing::instrument(skip(self))]
    pub fn walk(&self) -> impl ParallelIterator<Item = crate::error::Result<WalkEntry>> {
        let (tx, rx) = mpsc::channel();

        // Build the walker
        let mut builder = WalkBuilder::new(&self.root);
        builder
            .hidden(true)           // Skip hidden files and directories
            .git_ignore(true)       // Respect .gitignore
            .git_global(true)       // Respect global gitignore
            .git_exclude(true)      // Respect .git/info/exclude
            .threads(self.parallel_jobs);

        // Add additional ignore patterns
        for pattern in &self.additional_ignores {
            builder.add_ignore(pattern.clone());
        }

        // Build and walk in a separate thread to avoid blocking
        let root = self.root.clone();
        std::thread::spawn(move || {
            builder.build_parallel().run(|| {
                Box::new(|entry| {
                    match entry {
                        Ok(dir_entry) => {
                            // Only process files
                            if let Some(file_type) = dir_entry.file_type() {
                                if file_type.is_file() {
                                    let walk_entry = WalkEntry::from_dir_entry(dir_entry, &root);
                                    let _ = tx.send(Ok(walk_entry));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = tx.send(Err(crate::error::LicenseCheckerError::Scanner(
                                ScannerError::WalkError {
                                    path: PathBuf::from("<unknown>"),
                                    source: err,
                                },
                            )));
                        }
                    }
                    WalkState::Continue
                })
            });
            drop(tx); // Close the channel
        });

        // Convert the receiver into a parallel iterator
        rx.into_iter().par_bridge()
    }
}

/// Entry representing a file found during walking
#[derive(Debug, Clone)]
pub struct WalkEntry {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Depth from the root directory
    pub depth: usize,
    /// File type information
    pub file_type: std::fs::FileType,
}

impl WalkEntry {
    /// Create a WalkEntry from an ignore::DirEntry
    fn from_dir_entry(entry: DirEntry, _root: &Path) -> Self {
        // file_type() should never return None for a file entry (we check is_file() first)
        // But to satisfy clippy's no-expect rule, we handle it explicitly
        let file_type = entry.file_type().unwrap_or_else(|| {
            // This should never happen, but if it does, we'll try to get it from metadata
            std::fs::metadata(entry.path())
                .ok()
                .and_then(|m| Some(m.file_type()))
                .expect("file_type() returned None for a file entry - this should never happen")
        });

        Self { path: entry.path().to_path_buf(), depth: entry.depth(), file_type }
    }

    /// Get the file extension as a string
    pub fn extension(&self) -> Option<&str> {
        self.path.extension()?.to_str()
    }

    /// Get the file name as a string
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    /// Check if this entry represents a regular file
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// Get the relative path from the root
    pub fn relative_path(&self, root: &Path) -> Option<PathBuf> {
        self.path.strip_prefix(root).ok().map(|p| p.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn file_walker_new() {
        let walker = FileWalker::new("/tmp");
        assert_eq!(walker.root, PathBuf::from("/tmp"));
        assert!(walker.additional_ignores.is_empty());
        assert_eq!(walker.parallel_jobs, num_cpus::get());
    }

    #[test]
    fn file_walker_with_ignores() {
        let walker =
            FileWalker::new("/tmp").with_ignores(vec!["*.tmp".to_string(), "target/".to_string()]);

        assert_eq!(walker.additional_ignores, vec!["*.tmp", "target/"]);
    }

    #[test]
    fn file_walker_with_parallelism() {
        let walker = FileWalker::new("/tmp").with_parallelism(4);
        assert_eq!(walker.parallel_jobs, 4);

        // Test minimum bound
        let walker = FileWalker::new("/tmp").with_parallelism(0);
        assert_eq!(walker.parallel_jobs, 1);
    }

    #[test]
    fn walk_entry_properties() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let walker = FileWalker::new(&temp_dir);
        let entries: Vec<_> = walker.walk().filter_map(|r| r.ok()).collect();

        assert!(!entries.is_empty());
        let entry = entries.into_iter().find(|e| e.path == test_file).unwrap();

        assert_eq!(entry.file_name(), Some("test.rs"));
        assert_eq!(entry.extension(), Some("rs"));
        assert!(entry.is_file());
        assert_eq!(entry.relative_path(temp_dir.path()).unwrap(), PathBuf::from("test.rs"));
    }

    #[test]
    fn walk_entry_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("src");
        fs::create_dir(&sub_dir).unwrap();
        let test_file = sub_dir.join("main.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let walker = FileWalker::new(&temp_dir);
        let entries: Vec<_> = walker.walk().filter_map(|r| r.ok()).collect();

        let entry = entries.into_iter().find(|e| e.path == test_file).unwrap();
        assert_eq!(entry.relative_path(temp_dir.path()).unwrap(), PathBuf::from("src/main.rs"));
    }
}
