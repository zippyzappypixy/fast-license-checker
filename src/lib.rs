#![allow(clippy::arithmetic_side_effects)]

//! # Fast License Checker
//!
//! A blazing-fast library for checking and fixing license headers in source files.
//!
//! ## Features
//!
//! - **Fast**: Scans 100,000 files in under 1 second using parallel processing
//! - **Git-aware**: Automatically respects `.gitignore` files
//! - **Safe**: Never corrupts binary files
//! - **Smart**: Handles shebangs, XML declarations, and various comment styles
//!
//! ## Architecture
//!
//! The library is organized into the following modules:
//!
//! - `config` - Configuration loading and validation
//! - `types` - Domain types (NewTypes) with validation
//! - `scanner` - File walking with `.gitignore` support
//! - `checker` - License header detection and validation
//! - `fixer` - License header insertion with atomic writes
//! - `error` - Typed error definitions
//!
//! ## Example
//!
//! ```rust,ignore
//! use fast_license_checker::{Config, Scanner};
//!
//! let config = Config::default();
//! let scanner = Scanner::new(".", config)?;
//! let summary = scanner.scan();
//!
//! println!("Checked {} files", summary.total);
//! ```

// Note: Lints are configured in Cargo.toml [lints] section

// Module declarations will be added as we implement them
pub mod checker;
pub mod config;
pub mod error;
pub mod fixer;
pub mod scanner;
pub mod types;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
