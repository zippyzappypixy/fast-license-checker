//! # Fast License Checker (flc)
//!
//! A blazing-fast CLI tool for license header verification and fixing.
//! Scans directories to find files missing license headers and can automatically
//! add them with proper comment styles for different file types.

use clap::Parser;
use std::path::PathBuf;

/// Fast License Checker - Blazing fast license header verification
#[derive(Parser, Debug)]
#[command(name = "flc")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Directory or file to scan
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Run in fix mode (add missing headers)
    #[arg(short, long)]
    pub fix: bool,

    /// Path to file containing license header text
    #[arg(short = 'l', long = "license")]
    pub license_file: Option<PathBuf>,

    /// License header text (alternative to --license)
    #[arg(long = "header", conflicts_with = "license_file")]
    pub header_text: Option<String>,

    /// Config file path
    #[arg(short, long, default_value = ".license-checker.toml")]
    pub config: PathBuf,

    /// Number of parallel jobs (default: number of CPUs)
    #[arg(short, long, env = "FLC_JOBS")]
    pub jobs: Option<usize>,

    /// Maximum bytes to read for header check
    #[arg(long, default_value = "8192", env = "FLC_MAX_BYTES")]
    pub max_bytes: usize,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    pub output: OutputFormat,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Don't use colors in output
    #[arg(long)]
    pub no_color: bool,
}

use anyhow::{Context, Result};
use tracing_subscriber::{fmt, EnvFilter};

mod cli {
    pub mod output;
}

use cli::output::OutputFormat;
use fast_license_checker::{
    config::Config,
    fixer::HeaderFixer,
    scanner::Scanner,
    types::ScanSummary,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on verbosity
    init_tracing(cli.verbose, cli.quiet)?;

    // Load configuration
    let config = load_config(&cli).context("Failed to load configuration")?;

    tracing::debug!(?config, "Loaded configuration");

    // Validate license header is provided
    if config.license_header.is_empty() {
        anyhow::bail!(
            "No license header provided. Use --license <file> or --header <text>, \
             or add 'license_header' to your config file."
        );
    }

    // Run scan or fix
    let summary =
        if cli.fix { run_fix_mode(&cli, &config)? } else { run_scan_mode(&cli, &config)? };

    // Print results
    cli::output::print_summary(&summary, cli.output, !cli.no_color);

    // Exit with error code if there were failures
    if summary.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn init_tracing(verbose: u8, quiet: bool) -> Result<()> {
    let level = if quiet {
        "error"
    } else {
        match verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    fmt().with_env_filter(filter).with_target(false).init();

    Ok(())
}

fn run_scan_mode(cli: &Cli, config: &Config) -> Result<ScanSummary> {
    let scanner =
        Scanner::new(cli.path.as_path(), config.clone()).context("Failed to create scanner")?;

    let summary = scanner.scan()?;

    Ok(summary)
}

fn run_fix_mode(cli: &Cli, config: &Config) -> Result<ScanSummary> {
    let fixer =
        HeaderFixer::new(cli.path.as_path(), config.clone()).context("Failed to create fixer")?;

    let summary = fixer.fix_all().context("Fix operation failed")?;

    Ok(summary)
}

fn load_config(cli: &Cli) -> Result<Config> {
    use fast_license_checker::config::{load_config, CliOverrides};
    use std::fs;

    // Load license header from file or text
    let license_header = if let Some(file_path) = &cli.license_file {
        Some(fs::read_to_string(file_path).context("Failed to read license file")?)
    } else {
        cli.header_text.clone()
    };

    let overrides = CliOverrides {
        license_header,
        license_file: cli.license_file.clone(),
        parallel_jobs: cli.jobs,
        max_header_bytes: Some(cli.max_bytes),
        similarity_threshold: None, // CLI doesn't override this yet
    };

    Ok(load_config(Some(cli.config.as_path()), overrides)?)
}
