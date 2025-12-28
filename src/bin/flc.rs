//! # Fast License Checker CLI
//!
//! Command-line interface for the fast-license-checker library.
//!
//! ## Usage
//!
//! ```bash
//! # Scan current directory
//! flc .
//!
//! # Scan with specific header
//! flc --header "MIT License" src/
//!
//! # Fix missing headers
//! flc --fix --license LICENSE.txt .
//!
//! # JSON output for CI
//! flc --output json .
//! ```

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    // TODO: Implement CLI
    // 1. Parse arguments with clap
    // 2. Load configuration
    // 3. Run scan or fix mode
    // 4. Report results and exit

    info!("Fast License Checker v{}", fast_license_checker::VERSION);
    info!("CLI implementation coming soon...");

    Ok(())
}
