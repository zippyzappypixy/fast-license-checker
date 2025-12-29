//! Error types for the fast-license-checker library.
//!
//! All errors use the `thiserror` crate for automatic `Display` and `Error` trait implementations.
//! Library errors are typed, while the CLI binary converts them to user-friendly messages.

use std::path::PathBuf;

/// Top-level error type for the license checker library
#[derive(Debug, thiserror::Error)]
pub enum LicenseCheckerError {
    /// Configuration-related errors (file loading, parsing, validation)
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// File scanning errors (walking directories, reading files)
    #[error("Scanner error: {0}")]
    Scanner(#[from] ScannerError),

    /// Header checking errors (detection, validation)
    #[error("Checker error: {0}")]
    Checker(#[from] CheckerError),

    /// Header fixing errors (insertion, file writing)
    #[error("Fixer error: {0}")]
    Fixer(#[from] FixerError),

    /// Validation errors for domain types (NewTypes)
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Generic string error for cases that don't fit other categories
    #[error("Generic error: {0}")]
    Generic(String),
}

/// Configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration file was not found at the specified path
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    /// Failed to parse configuration file (TOML/JSON syntax error)
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    /// Required configuration field is missing
    #[error("Missing required field: {field}")]
    MissingField {
        /// Name of the missing field
        field: &'static str,
    },

    /// Configuration field has an invalid value
    #[error("Invalid value for {field}: {message}")]
    InvalidValue {
        /// Name of the field with invalid value
        field: &'static str,
        /// Description of why the value is invalid
        message: String,
    },
}

/// File scanning errors
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    /// Failed to walk directory tree (permissions, I/O errors)
    #[error("Failed to walk directory {path}: {source}")]
    WalkError {
        /// Path where the walk error occurred
        path: PathBuf,
        /// Underlying error from the ignore crate
        #[source]
        source: ignore::Error,
    },

    /// I/O error while reading a file
    #[error("IO error reading {path}: {source}")]
    Io {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },
}

/// Header checking errors
#[derive(Debug, thiserror::Error)]
pub enum CheckerError {
    /// I/O error while reading file for header check
    #[error("IO error reading {path}: {source}")]
    Io {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// File contains binary data (NULL bytes detected)
    #[error("File appears to be binary: {0}")]
    BinaryFile(PathBuf),

    /// File encoding is not supported (non-UTF-8)
    #[error("Unsupported encoding in file: {0}")]
    UnsupportedEncoding(PathBuf),
}

/// Header fixing errors
#[derive(Debug, thiserror::Error)]
pub enum FixerError {
    /// Attempted to fix a binary file (not supported)
    #[error("Cannot fix binary file: {0}")]
    BinaryFile(PathBuf),

    /// Failed to write file during fix operation
    #[error("Failed to write {path}: {source}")]
    WriteError {
        /// Path to the file that couldn't be written
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Header already exists - prevents duplicate insertion
    #[error("Header already exists in {0} - refusing to duplicate")]
    IdempotencyViolation(PathBuf),

    /// Detected malformed header that requires manual review
    #[error(
        "Malformed header detected in {path} (similarity: {similarity}%) - manual review required"
    )]
    MalformedHeader {
        /// Path to the file with malformed header
        path: PathBuf,
        /// Similarity score (0-100) of the malformed header
        similarity: u8,
    },

    /// Failed to read file for fixing
    #[error("Failed to read {path}: {source}")]
    ReadError {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// File extension has no configured comment style
    #[error("Unsupported file extension '{extension}' for file: {path}")]
    UnsupportedExtension {
        /// The unsupported extension
        extension: String,
        /// Path to the file with unsupported extension
        path: PathBuf,
    },
}

/// Validation errors for NewTypes
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// License header text is empty or whitespace-only
    #[error("License header cannot be empty")]
    EmptyHeader,

    /// File extension string is empty
    #[error("File extension cannot be empty")]
    EmptyExtension,

    /// File extension contains invalid characters (not alphanumeric, underscore, +, or #)
    #[error("File extension contains invalid characters")]
    InvalidExtension,

    /// MaxHeaderBytes value is below the minimum (256 bytes)
    #[error("MaxHeaderBytes must be at least 256, got {0}")]
    HeaderBytesTooSmall(usize),

    /// Similarity score is outside valid range (0-100)
    #[error("Similarity score must be 0-100, got {0}")]
    InvalidSimilarity(u8),
}

// Re-export common error types for convenience
pub use LicenseCheckerError as Error;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_checker_error_display() {
        let error =
            LicenseCheckerError::Config(ConfigError::NotFound(PathBuf::from("/tmp/test.toml")));
        assert_eq!(error.to_string(), "Configuration error: Config file not found: /tmp/test.toml");
    }

    #[test]
    fn config_error_display() {
        let error = ConfigError::NotFound(PathBuf::from("/tmp/test.toml"));
        assert_eq!(error.to_string(), "Config file not found: /tmp/test.toml");

        let error = ConfigError::MissingField { field: "license_header" };
        assert_eq!(error.to_string(), "Missing required field: license_header");

        let error = ConfigError::InvalidValue {
            field: "max_header_bytes",
            message: "must be positive".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid value for max_header_bytes: must be positive");
    }

    #[test]
    fn scanner_error_display() {
        let error = ScannerError::Io {
            path: PathBuf::from("/tmp/test.txt"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        };
        let error_str = error.to_string();
        assert!(error_str.contains("IO error reading /tmp/test.txt"));
        assert!(error_str.contains("file not found"));
    }

    #[test]
    fn checker_error_display() {
        let error = CheckerError::BinaryFile(PathBuf::from("/tmp/binary.exe"));
        assert_eq!(error.to_string(), "File appears to be binary: /tmp/binary.exe");

        let error = CheckerError::UnsupportedEncoding(PathBuf::from("/tmp/utf16.txt"));
        assert_eq!(error.to_string(), "Unsupported encoding in file: /tmp/utf16.txt");
    }

    #[test]
    fn fixer_error_display() {
        let error = FixerError::BinaryFile(PathBuf::from("/tmp/binary.exe"));
        assert_eq!(error.to_string(), "Cannot fix binary file: /tmp/binary.exe");

        let error = FixerError::IdempotencyViolation(PathBuf::from("/tmp/file.txt"));
        assert_eq!(
            error.to_string(),
            "Header already exists in /tmp/file.txt - refusing to duplicate"
        );

        let error =
            FixerError::MalformedHeader { path: PathBuf::from("/tmp/file.txt"), similarity: 85 };
        assert_eq!(
            error.to_string(),
            "Malformed header detected in /tmp/file.txt (similarity: 85%) - manual review required"
        );
    }

    #[test]
    fn validation_error_display() {
        let error = ValidationError::EmptyHeader;
        assert_eq!(error.to_string(), "License header cannot be empty");

        let error = ValidationError::HeaderBytesTooSmall(100);
        assert_eq!(error.to_string(), "MaxHeaderBytes must be at least 256, got 100");

        let error = ValidationError::InvalidSimilarity(150);
        assert_eq!(error.to_string(), "Similarity score must be 0-100, got 150");
    }

    #[test]
    fn error_conversion() {
        // Test that errors can be converted to LicenseCheckerError
        let config_error = ConfigError::NotFound(PathBuf::from("/tmp/test.toml"));
        let license_error: LicenseCheckerError = config_error.into();
        assert!(matches!(license_error, LicenseCheckerError::Config(_)));

        let fixer_error = FixerError::BinaryFile(PathBuf::from("/tmp/binary.exe"));
        let license_error: LicenseCheckerError = fixer_error.into();
        assert!(matches!(license_error, LicenseCheckerError::Fixer(_)));
    }

    #[test]
    fn license_checker_error_from_validation_error() {
        let validation_error = ValidationError::EmptyHeader;
        let license_error: LicenseCheckerError = validation_error.into();
        assert!(matches!(
            license_error,
            LicenseCheckerError::Validation(ValidationError::EmptyHeader)
        ));
    }

    #[test]
    fn license_checker_error_from_string() {
        let error_str = "some validation error".to_string();
        let license_error: LicenseCheckerError = error_str.clone().into();
        // String errors convert to Generic variant, not Validation
        assert!(matches!(
            license_error,
            LicenseCheckerError::Generic(ref s) if s == &error_str
        ));
    }
}

impl From<String> for LicenseCheckerError {
    fn from(err: String) -> Self {
        // Convert string errors to a generic error variant
        Self::Generic(err)
    }
}
