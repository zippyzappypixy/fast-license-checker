//! Configuration loading logic.
//!
//! Handles loading configuration from files, CLI arguments, and environment variables.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::types::Config;
use crate::error::{ConfigError, Result};

/// CLI argument overrides for configuration
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    /// Override license header text
    pub license_header: Option<String>,
    /// Load license header from file
    pub license_file: Option<PathBuf>,
    /// Override parallel jobs
    pub parallel_jobs: Option<usize>,
    /// Override max header bytes
    pub max_header_bytes: Option<usize>,
    /// Override similarity threshold
    pub similarity_threshold: Option<u8>,
}

/// Load configuration with the following priority (highest to lowest):
/// 1. CLI overrides
/// 2. Environment variables
/// 3. Configuration file
/// 4. Default values
#[tracing::instrument(skip(cli_overrides))]
pub fn load_config(config_path: Option<&Path>, cli_overrides: CliOverrides) -> Result<Config> {
    // Start with defaults
    let mut config = Config::default();

    // Load from configuration file if it exists
    if let Some(path) = config_path {
        if path.exists() {
            config = load_from_file(path)?;
        }
    } else {
        // Try default config file locations
        let default_paths = [
            PathBuf::from(".flc.toml"),
            PathBuf::from(".flc.json"),
            PathBuf::from("flc.toml"),
            PathBuf::from("flc.json"),
        ];

        for path in &default_paths {
            if path.exists() {
                config = load_from_file(path)?;
                break;
            }
        }
    }

    // Apply environment variable overrides
    config = apply_env_overrides(config)?;

    // Apply CLI overrides
    config = apply_cli_overrides(config, cli_overrides)?;

    // Validate final configuration
    validate_config(&config)?;

    Ok(config)
}

/// Load configuration from a TOML or JSON file
#[tracing::instrument]
fn load_from_file(path: &Path) -> Result<Config> {
    let content =
        fs::read_to_string(path).map_err(|_| ConfigError::NotFound(path.to_path_buf()))?;

    let config = if path.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&content).map_err(|e| ConfigError::InvalidValue {
            field: "config_file",
            message: format!("Invalid JSON format: {}", e),
        })
    } else {
        toml::from_str(&content).map_err(ConfigError::Parse)
    }?;

    Ok(config)
}

/// Apply environment variable overrides
#[tracing::instrument]
fn apply_env_overrides(mut config: Config) -> Result<Config> {
    // FLC_HEADER - license header text
    if let Ok(header) = env::var("FLC_HEADER") {
        if !header.trim().is_empty() {
            config.license_header = header;
        }
    }

    // FLC_MAX_BYTES - maximum header bytes
    if let Ok(bytes_str) = env::var("FLC_MAX_BYTES") {
        if let Ok(bytes) = bytes_str.parse::<usize>() {
            config.max_header_bytes = bytes;
        }
    }

    // FLC_SIMILARITY_THRESHOLD - similarity threshold
    if let Ok(threshold_str) = env::var("FLC_SIMILARITY_THRESHOLD") {
        if let Ok(threshold) = threshold_str.parse::<u8>() {
            config.similarity_threshold = threshold.min(100);
        }
    }

    // FLC_PARALLEL_JOBS - number of parallel jobs
    if let Ok(jobs_str) = env::var("FLC_PARALLEL_JOBS") {
        if let Ok(jobs) = jobs_str.parse::<usize>() {
            config.parallel_jobs = Some(jobs);
        }
    }

    Ok(config)
}

/// Apply CLI argument overrides
#[tracing::instrument(skip(cli_overrides))]
fn apply_cli_overrides(mut config: Config, cli_overrides: CliOverrides) -> Result<Config> {
    // License header from CLI
    if let Some(header) = cli_overrides.license_header {
        config.license_header = header;
    }

    // License header from file
    if let Some(license_file) = cli_overrides.license_file {
        let header_content = fs::read_to_string(&license_file).map_err(|e| {
            crate::error::LicenseCheckerError::Config(ConfigError::InvalidValue {
                field: "license_file",
                message: format!("Could not read license file: {}", e),
            })
        })?;
        config.license_header = header_content;
    }

    // Other overrides
    if let Some(jobs) = cli_overrides.parallel_jobs {
        config.parallel_jobs = Some(jobs);
    }

    if let Some(bytes) = cli_overrides.max_header_bytes {
        config.max_header_bytes = bytes;
    }

    if let Some(threshold) = cli_overrides.similarity_threshold {
        config.similarity_threshold = threshold.min(100);
    }

    Ok(config)
}

/// Validate the final configuration
#[tracing::instrument]
fn validate_config(config: &Config) -> Result<()> {
    // Validate max_header_bytes is reasonable
    if config.max_header_bytes < 256 {
        return Err(crate::error::LicenseCheckerError::Config(ConfigError::InvalidValue {
            field: "max_header_bytes",
            message: "must be at least 256 bytes".to_string(),
        }));
    }

    // Validate similarity_threshold is in range
    if config.similarity_threshold > 100 {
        return Err(crate::error::LicenseCheckerError::Config(ConfigError::InvalidValue {
            field: "similarity_threshold",
            message: "must be between 0 and 100".to_string(),
        }));
    }

    // Validate parallel_jobs is reasonable if set
    if let Some(jobs) = config.parallel_jobs {
        if jobs == 0 {
            return Err(crate::error::LicenseCheckerError::Config(ConfigError::InvalidValue {
                field: "parallel_jobs",
                message: "must be greater than 0".to_string(),
            }));
        }
    }

    Ok(())
}

/// Create a basic configuration file template
pub fn create_config_template(path: &Path, format: &str) -> Result<()> {
    let template = match format {
        "toml" => {
            let mut template = String::from("# Fast License Checker Configuration\n");
            template.push_str("license_header = \"\"\"\n");
            template.push_str("Copyright 2024 Your Organization\n");
            template.push('\n');
            template.push_str("Licensed under the MIT License.\n");
            template.push_str("\"\"\"\n");
            template.push('\n');
            template.push_str("# Comment styles per file extension (defaults provided)\n");
            template.push_str("# [comment_styles]\n");
            template.push_str("# rs = { prefix = \"//\" }\n");
            template.push_str("# py = { prefix = \"#\" }\n");
            template.push_str("# css = { prefix = \"/*\", suffix = \"*/\" }\n");
            template.push('\n');
            template.push_str("# Additional ignore patterns (beyond .gitignore)\n");
            template.push_str("ignore_patterns = [\n");
            template.push_str("    \"*.tmp\",\n");
            template.push_str("    \"*.bak\",\n");
            template.push_str("    \"target/\",\n");
            template.push_str("    \"node_modules/\",\n");
            template.push_str("]\n");
            template.push('\n');
            template.push_str("# Maximum bytes to read from file start\n");
            template.push_str("max_header_bytes = 8192\n");
            template.push('\n');
            template.push_str("# Skip empty files\n");
            template.push_str("skip_empty_files = true\n");
            template.push('\n');
            template.push_str("# Number of parallel jobs (default: number of CPU cores)\n");
            template.push_str("# parallel_jobs = 4\n");
            template.push('\n');
            template.push_str("# Similarity threshold for malformed header detection (0-100)\n");
            template.push_str("similarity_threshold = 70\n");
            template
        }
        "json" => r#"{
  "license_header": "Copyright 2024 Your Organization\n\nLicensed under the MIT License.\n",
  "ignore_patterns": [
    "*.tmp",
    "*.bak",
    "target/",
    "node_modules/"
  ],
  "max_header_bytes": 8192,
  "skip_empty_files": true,
  "similarity_threshold": 70
}"#
        .to_string(),
        _ => {
            return Err(crate::error::LicenseCheckerError::Config(ConfigError::InvalidValue {
                field: "format",
                message: "must be 'toml' or 'json'".to_string(),
            }))
        }
    };

    fs::write(path, template).map_err(|e| ConfigError::InvalidValue {
        field: "config_path",
        message: format!("Could not write config file: {}", e),
    })?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_config_defaults() {
        let config = load_config(None, CliOverrides::default()).unwrap();
        assert_eq!(config.max_header_bytes, 8192);
        assert_eq!(config.similarity_threshold, 70);
        assert!(config.skip_empty_files);
    }

    #[test]
    fn load_config_with_cli_overrides() {
        let overrides = CliOverrides {
            license_header: Some("Test License".to_string()),
            max_header_bytes: Some(4096),
            similarity_threshold: Some(80),
            ..Default::default()
        };

        let config = load_config(None, overrides).unwrap();
        assert_eq!(config.license_header, "Test License");
        assert_eq!(config.max_header_bytes, 4096);
        assert_eq!(config.similarity_threshold, 80);
    }

    #[test]
    fn load_config_with_license_file() {
        let temp_dir = TempDir::new().unwrap();
        let license_file = temp_dir.path().join("LICENSE");
        fs::write(&license_file, "MIT License Content").unwrap();

        let overrides = CliOverrides { license_file: Some(license_file), ..Default::default() };

        let config = load_config(None, overrides).unwrap();
        assert_eq!(config.license_header, "MIT License Content");
    }

    #[test]
    fn validate_config_invalid_max_header_bytes() {
        let config = Config {
            max_header_bytes: 100, // Too small
            ..Default::default()
        };

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_config_invalid_similarity_threshold() {
        let config = Config {
            similarity_threshold: 150, // Too high
            ..Default::default()
        };

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn create_config_template_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        create_config_template(&config_path, "toml").unwrap();
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("license_header"));
        assert!(content.contains("max_header_bytes"));
    }

    #[test]
    fn create_config_template_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.json");

        create_config_template(&config_path, "json").unwrap();
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("license_header"));
        assert!(content.contains("max_header_bytes"));
    }

    #[test]
    fn test_apply_env_overrides() {
        // Set environment variables
        env::set_var("FLC_HEADER", "Env License");
        env::set_var("FLC_MAX_BYTES", "4096");
        env::set_var("FLC_SIMILARITY_THRESHOLD", "85");

        let config = apply_env_overrides(Config::default()).unwrap();
        assert_eq!(config.license_header, "Env License");
        assert_eq!(config.max_header_bytes, 4096);
        assert_eq!(config.similarity_threshold, 85);

        // Clean up
        env::remove_var("FLC_HEADER");
        env::remove_var("FLC_MAX_BYTES");
        env::remove_var("FLC_SIMILARITY_THRESHOLD");
    }

    #[test]
    fn load_from_file_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let toml_content = r#"
            license_header = "TOML License"
            max_header_bytes = 4096
            similarity_threshold = 75
        "#;

        fs::write(&config_path, toml_content).unwrap();

        let config = load_from_file(&config_path).unwrap();
        assert_eq!(config.license_header, "TOML License");
        assert_eq!(config.max_header_bytes, 4096);
        assert_eq!(config.similarity_threshold, 75);
    }

    #[test]
    fn load_from_file_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.json");

        let json_content = r#"{
            "license_header": "JSON License",
            "max_header_bytes": 2048,
            "similarity_threshold": 65
        }"#;

        fs::write(&config_path, json_content).unwrap();

        let config = load_from_file(&config_path).unwrap();
        assert_eq!(config.license_header, "JSON License");
        assert_eq!(config.max_header_bytes, 2048);
        assert_eq!(config.similarity_threshold, 65);
    }
}
