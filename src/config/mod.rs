//! Configuration management for fast-license-checker.
//!
//! This module handles loading and managing configuration from multiple sources
//! with proper precedence: CLI arguments > environment variables > config files > defaults.
//!
//! ## Configuration Sources (in priority order)
//!
//! 1. **CLI Arguments**: Direct overrides for specific settings
//! 2. **Environment Variables**: `FLC_*` prefixed variables for automation
//! 3. **Configuration Files**: TOML or JSON files (`.flc.toml`, `.flc.json`, etc.)
//! 4. **Defaults**: Sensible defaults for all settings
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use fast_license_checker::config::{load_config, CliOverrides};
//!
//! // Load with defaults
//! let config = load_config(None, CliOverrides::default())?;
//!
//! // Load with CLI overrides
//! let overrides = CliOverrides {
//!     license_header: Some("MIT License".to_string()),
//!     max_header_bytes: Some(4096),
//!     ..Default::default()
//! };
//! let config = load_config(Some(Path::new(".flc.toml")), overrides)?;
//! ```
//!
//! ## Configuration File Format
//!
//! ### TOML Example
//! ```toml
//! license_header = """
//! Copyright 2024 Your Organization
//!
//! Licensed under the MIT License.
//! """
//!
//! [comment_styles]
//! rs = { prefix = "//" }
//! py = { prefix = "#" }
//!
//! ignore_patterns = ["*.tmp", "target/"]
//! max_header_bytes = 8192
//! similarity_threshold = 70
//! ```
//!
//! ### JSON Example
//! ```json
//! {
//!   "license_header": "Copyright 2024 Your Organization\n\nLicensed under the MIT License.\n",
//!   "ignore_patterns": ["*.tmp", "target/"],
//!   "max_header_bytes": 8192,
//!   "similarity_threshold": 70
//! }
//! ```

pub mod loader;
pub mod types;

// Re-export main types and functions for convenience
pub use loader::{create_config_template, load_config, CliOverrides};
pub use types::{CommentStyleConfig, Config};
