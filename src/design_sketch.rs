//! Design sketch for Fast License Checker types and traits.
//!
//! This file contains all the core types and interfaces that will be implemented.
//! It serves as a design validation step before full implementation.
//!
//! Run `cargo check` to validate this design compiles correctly.

use std::path::PathBuf;
use std::time::Duration;

/// A validated file path wrapper.
///
/// Provides type safety and validation for file system paths.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePath(PathBuf);

impl FilePath {
    /// Creates a new FilePath from a PathBuf.
    ///
    /// # Errors
    /// Returns an error if the path is invalid or empty.
    pub fn new(path: PathBuf) -> Result<Self, LicenseCheckerError> {
        if path.as_os_str().is_empty() {
            return Err(LicenseCheckerError::Validation(
                "File path cannot be empty".to_string()
            ));
        }
        Ok(Self(path))
    }

    /// Returns the underlying PathBuf.
    pub fn as_path(&self) -> &PathBuf {
        &self.0
    }
}

/// A validated license header that is guaranteed non-empty.
///
/// Ensures license headers are properly formatted and non-empty at creation time.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LicenseHeader(String);

impl LicenseHeader {
    /// Creates a new LicenseHeader, validating that it's not empty.
    ///
    /// # Errors
    /// Returns `ValidationError` if the input is empty after trimming.
    pub fn new(s: impl Into<String>) -> Result<Self, LicenseCheckerError> {
        let s = s.into();
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(LicenseCheckerError::Validation(
                "License header cannot be empty".to_string()
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the header as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the header as a byte slice for efficient comparison.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Comment style configuration for different file types.
///
/// Defines how to format license headers for different programming languages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentStyle {
    /// The prefix for comment lines (e.g., "//", "#", "/*")
    pub prefix: String,
    /// Optional suffix for block comments (e.g., "*/")
    pub suffix: Option<String>,
}

impl CommentStyle {
    /// Creates a new comment style.
    pub fn new(prefix: String, suffix: Option<String>) -> Self {
        Self { prefix, suffix }
    }

    /// Returns a line comment style (e.g., "//" for Rust, "#" for Python).
    pub fn line_comment(prefix: &str) -> Self {
        Self::new(prefix.to_string(), None)
    }

    /// Returns a block comment style (e.g., "/*" and "*/" for CSS).
    pub fn block_comment(prefix: &str, suffix: &str) -> Self {
        Self::new(prefix.to_string(), Some(suffix.to_string()))
    }
}

/// A validated file extension (lowercase, no leading dot).
///
/// Ensures consistent handling of file extensions across the codebase.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileExtension(String);

impl FileExtension {
    /// Creates a new FileExtension from a string.
    ///
    /// Automatically converts to lowercase and removes leading dots.
    ///
    /// # Errors
    /// Returns an error if the extension is empty after processing.
    pub fn new(s: impl Into<String>) -> Result<Self, LicenseCheckerError> {
        let s = s.into();
        let ext = s.trim_start_matches('.').to_lowercase();
        if ext.is_empty() {
            return Err(LicenseCheckerError::Validation(
                "File extension cannot be empty".to_string()
            ));
        }
        Ok(Self(ext))
    }

    /// Returns the extension as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Maximum bytes to read from file start for header detection.
///
/// Limits memory usage and prevents reading entire large files.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaxHeaderBytes(usize);

impl MaxHeaderBytes {
    /// The minimum allowed value (256 bytes).
    pub const MIN_VALUE: usize = 256;

    /// Creates a new MaxHeaderBytes with validation.
    ///
    /// # Errors
    /// Returns an error if the value is less than the minimum.
    pub fn new(value: usize) -> Result<Self, LicenseCheckerError> {
        if value < Self::MIN_VALUE {
            return Err(LicenseCheckerError::Validation(
                format!("MaxHeaderBytes must be at least {}", Self::MIN_VALUE)
            ));
        }
        Ok(Self(value))
    }

    /// Returns the value as usize.
    pub fn value(&self) -> usize {
        self.0
    }
}

/// Similarity score for fuzzy header matching (0-100).
///
/// Represents how closely a file's header matches the expected header.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimilarityScore(u8);

impl SimilarityScore {
    /// Maximum similarity score (100%).
    pub const MAX: Self = Self(100);
    /// Minimum similarity score (0%).
    pub const MIN: Self = Self(0);

    /// Creates a new SimilarityScore, clamped to 0-100.
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// Returns the score as u8.
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Returns true if this represents an exact match.
    pub fn is_exact(&self) -> bool {
        self.0 == 100
    }
}

// Enums

/// The status of a file's license header check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    /// File has the correct license header.
    HasHeader,
    /// File is missing a license header.
    MissingHeader,
    /// File has a malformed header (with similarity score).
    MalformedHeader { similarity: SimilarityScore },
    /// File was skipped during scanning.
    Skipped { reason: SkipReason },
}

/// Reasons why a file might be skipped during scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// File contains binary data (detected by NULL bytes).
    Binary,
    /// File is empty (0 bytes).
    Empty,
    /// File is ignored by .gitignore rules.
    Gitignored,
    /// File is too large to check efficiently.
    TooLarge,
    /// File encoding is not supported.
    UnsupportedEncoding,
    /// No comment style configured for this file type.
    NoCommentStyle,
}

/// The mode of operation for the license checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    /// Only check files, don't modify them.
    Check,
    /// Check and fix files with missing headers.
    Fix,
}

/// The result of attempting to fix a file's license header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixAction {
    /// File was successfully fixed with a license header.
    Fixed,
    /// File already had the correct header.
    AlreadyHasHeader,
    /// File was skipped during fixing.
    Skipped { reason: SkipReason },
    /// Fixing failed with an error message.
    Failed { error: String },
}

// Result structs

/// The result of checking a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    /// The path to the file that was checked.
    pub path: FilePath,
    /// The status of the file's license header.
    pub status: FileStatus,
}

/// The result of attempting to fix a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixResult {
    /// The path to the file that was processed.
    pub path: FilePath,
    /// The action taken on the file.
    pub action: FixAction,
}

/// Summary of a complete scan operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanSummary {
    /// Total number of files processed.
    pub total: usize,
    /// Number of files that passed (had correct headers).
    pub passed: usize,
    /// Number of files that failed (missing or malformed headers).
    pub failed: usize,
    /// Number of files that were skipped.
    pub skipped: usize,
    /// How long the scan took.
    pub duration: Duration,
}

// Traits

/// Trait for checking license headers in file content.
pub trait HeaderChecker {
    /// Checks if the file content contains the expected license header.
    ///
    /// # Arguments
    /// * `content` - The file content as bytes
    /// * `expected` - The expected license header
    /// * `style` - The comment style for this file type
    ///
    /// # Returns
    /// The status of the file's license header
    fn check(&self, content: &[u8], expected: &LicenseHeader, style: &CommentStyle) -> FileStatus;
}

/// Trait for fixing missing license headers in file content.
pub trait HeaderFixer {
    /// Adds the license header to file content if missing.
    ///
    /// # Arguments
    /// * `content` - The original file content as bytes
    /// * `header` - The license header to add
    /// * `style` - The comment style for this file type
    ///
    /// # Returns
    /// The modified content with header added, or an error
    fn fix(&self, content: &[u8], header: &LicenseHeader, style: &CommentStyle) -> Result<Vec<u8>, FixerError>;
}

// Error types

/// Top-level errors that can occur during license checking operations.
#[derive(Debug, thiserror::Error)]
pub enum LicenseCheckerError {
    /// I/O error when reading or writing files.
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Configuration-related errors.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Validation errors for domain types.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Errors during header fixing operations.
    #[error("Fixer error: {0}")]
    Fixer(#[from] FixerError),
}

/// Errors related to configuration loading and parsing.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration file not found.
    #[error("Configuration file not found: {path}")]
    NotFound { path: PathBuf },

    /// Error parsing configuration file.
    #[error("Failed to parse config at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    /// Required configuration field is missing.
    #[error("Missing required config field: {field}")]
    MissingField { field: String },
}

/// Errors that can occur during header fixing operations.
#[derive(Debug, thiserror::Error)]
pub enum FixerError {
    /// Attempted to fix a binary file.
    #[error("Cannot fix binary file: {path}")]
    BinaryFile { path: PathBuf },

    /// Failed to write the fixed content to disk.
    #[error("Write error at {path}: {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Idempotency check failed (file changed unexpectedly).
    #[error("Idempotency violation: file changed during fix operation")]
    IdempotencyViolation,
}

// Placeholder implementations for compilation (to be replaced with real logic)

impl Default for CommentStyle {
    fn default() -> Self {
        Self::line_comment("//")
    }
}

impl Default for MaxHeaderBytes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Default for SimilarityScore {
    fn default() -> Self {
        Self::MIN
    }
}

impl ScanSummary {
    /// Creates a new scan summary.
    pub fn new(total: usize, passed: usize, failed: usize, skipped: usize, duration: Duration) -> Self {
        Self { total, passed, failed, skipped, duration }
    }
}

// Dummy implementations for traits (for compilation only)

pub struct DefaultHeaderChecker;
pub struct DefaultHeaderFixer;

impl HeaderChecker for DefaultHeaderChecker {
    fn check(&self, _content: &[u8], _expected: &LicenseHeader, _style: &CommentStyle) -> FileStatus {
        FileStatus::HasHeader // Placeholder
    }
}

impl HeaderFixer for DefaultHeaderFixer {
    fn fix(&self, content: &[u8], _header: &LicenseHeader, _style: &CommentStyle) -> Result<Vec<u8>, FixerError> {
        Ok(content.to_vec()) // Placeholder
    }
}
