//! Error types for the fast-license-checker library.
//!
//! All errors use the `thiserror` crate for automatic `Display` and `Error` trait implementations.
//! Library errors are typed, while the CLI binary converts them to user-friendly messages.

use std::path::PathBuf;

/// Top-level error type for the license checker library
#[derive(Debug, thiserror::Error)]
pub enum LicenseCheckerError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Scanner error: {0}")]
    Scanner(#[from] ScannerError),

    #[error("Checker error: {0}")]
    Checker(#[from] CheckerError),

    #[error("Fixer error: {0}")]
    Fixer(#[from] FixerError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}

/// Configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Missing required field: {field}")]
    MissingField { field: &'static str },

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: &'static str, message: String },
}

/// File scanning errors
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("Failed to walk directory {path}: {source}")]
    WalkError {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },

    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Header checking errors
#[derive(Debug, thiserror::Error)]
pub enum CheckerError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File appears to be binary: {0}")]
    BinaryFile(PathBuf),

    #[error("Unsupported encoding in file: {0}")]
    UnsupportedEncoding(PathBuf),
}

/// Header fixing errors
#[derive(Debug, thiserror::Error)]
pub enum FixerError {
    #[error("Cannot fix binary file: {0}")]
    BinaryFile(PathBuf),

    #[error("Failed to write {path}: {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Header already exists in {0} - refusing to duplicate")]
    IdempotencyViolation(PathBuf),

    #[error(
        "Malformed header detected in {path} (similarity: {similarity}%) - manual review required"
    )]
    MalformedHeader { path: PathBuf, similarity: u8 },

    #[error("Failed to read {path}: {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Unsupported file extension '{extension}' for file: {path}")]
    UnsupportedExtension { extension: String, path: PathBuf },
}

/// Validation errors for NewTypes
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("License header cannot be empty")]
    EmptyHeader,

    #[error("File extension cannot be empty")]
    EmptyExtension,

    #[error("File extension contains invalid characters")]
    InvalidExtension,

    #[error("MaxHeaderBytes must be at least 256, got {0}")]
    HeaderBytesTooSmall(usize),

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
        assert!(matches!(license_error, LicenseCheckerError::Validation(ValidationError::EmptyHeader)));
    }

    #[test]
    fn license_checker_error_from_string() {
        let error_str = "some validation error".to_string();
        let license_error: LicenseCheckerError = error_str.into();
        assert!(matches!(license_error, LicenseCheckerError::Validation(ValidationError::InvalidExtension)));
    }
}

impl From<String> for LicenseCheckerError {
    fn from(_err: String) -> Self {
        // Convert string errors to a generic validation error
        Self::Validation(ValidationError::InvalidExtension)
    }
}
