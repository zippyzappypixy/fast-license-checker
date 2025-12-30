//! License header fixing functionality.
//!
//! Provides the main interface for adding license headers to files that are missing them,
//! with atomic writes and comprehensive error handling.

pub mod inserter;
pub mod writer;

use rayon::iter::ParallelIterator;
use std::path::Path;
use tracing::{debug, info};

use crate::{
    checker::HeaderChecker,
    config::Config,
    error::{FixerError, Result},
    scanner::walker::{FileWalker, WalkEntry},
    types::header_types::CommentStyle,
    types::{FilePath, FileStatus, ScanResult, ScanSummary, SkipReason},
};

/// Main interface for fixing license headers in files.
#[derive(Debug)]
pub struct HeaderFixer {
    walker: FileWalker,
    checker: HeaderChecker,
    config: Config,
}

impl HeaderFixer {
    /// Creates a new HeaderFixer with the given configuration.
    #[tracing::instrument(skip(config))]
    pub fn new(root: &Path, config: Config) -> Result<Self> {
        let walker = FileWalker::new(root)
            .with_ignores(config.ignore_patterns.clone())
            .with_parallelism(config.parallel_jobs.unwrap_or(1));
        let checker = HeaderChecker::new(&config)?;

        Ok(Self { walker, checker, config })
    }

    /// Fixes all files that are missing license headers.
    ///
    /// Returns a summary of the operation.
    #[tracing::instrument(skip(self))]
    #[allow(clippy::arithmetic_side_effects)] // Intentional counter increments
    pub fn fix_all(&self) -> Result<ScanSummary> {
        use std::time::Instant;

        info!("Starting fix operation");
        let start = Instant::now();

        // Get all files and their status
        let entries: Vec<WalkEntry> = self.walker.walk().collect::<Result<Vec<_>>>()?;

        let mut fixed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for entry in entries {
            // Check if file needs fixing
            let result = self.check_file(&entry)?;

            match result.status {
                FileStatus::MissingHeader => match self.fix_file(&result.path) {
                    Ok(_) => {
                        debug!(path = %result.path.as_path().display(), "Fixed file");
                        fixed += 1;
                    }
                    Err(e) => {
                        debug!(path = %result.path.as_path().display(), error = %e, "Failed to fix file");
                        failed += 1;
                    }
                },
                FileStatus::HasHeader => {
                    // Already has header, count as passed
                }
                FileStatus::Skipped { reason: _ } => {
                    skipped += 1;
                }
                FileStatus::MalformedHeader { .. } => {
                    failed += 1;
                }
            }
        }

        let total = fixed + failed + skipped;
        let duration = start.elapsed();
        let summary = ScanSummary::new(
            total, fixed, // Fixed files now pass
            failed, skipped, duration,
        );

        info!(
            total = summary.total,
            fixed,
            failed = summary.failed,
            skipped = summary.skipped,
            duration = ?duration,
            "Fix operation complete"
        );

        Ok(summary)
    }

    /// Checks a single file to determine its header status.
    #[tracing::instrument(skip(self))]
    fn check_file(&self, entry: &WalkEntry) -> Result<ScanResult> {
        use crate::scanner::filter::{is_binary, is_valid_utf8};

        let file_path = FilePath::new(entry.path.clone());

        // Read file content first
        let content = match std::fs::read(file_path.as_path()) {
            Ok(content) => content,
            Err(_e) => {
                // File read error - skip with appropriate reason
                return Ok(ScanResult {
                    path: file_path.clone(),
                    status: FileStatus::Skipped { reason: SkipReason::UnsupportedEncoding },
                });
            }
        };

        // Check if binary
        if is_binary(&content) {
            return Ok(ScanResult {
                path: file_path.clone(),
                status: FileStatus::Skipped { reason: SkipReason::Binary },
            });
        }

        // Check if valid UTF-8 for text processing
        if !is_valid_utf8(&content) {
            return Ok(ScanResult {
                path: file_path.clone(),
                status: FileStatus::Skipped { reason: SkipReason::UnsupportedEncoding },
            });
        }

        // Check if we should process this file
        let extension = file_path.extension().map(|ext| ext.as_str().to_string());
        if let Err(reason) = crate::scanner::filter::should_process_file(
            &content,
            extension.as_deref(),
            &self.config,
        ) {
            return Ok(ScanResult {
                path: file_path.clone(),
                status: FileStatus::Skipped { reason },
            });
        }

        // Check header
        match self.checker.check_file(file_path.as_path()) {
            Ok(status) => Ok(ScanResult { path: file_path.clone(), status }),
            Err(_) => Ok(ScanResult {
                path: file_path.clone(),
                status: FileStatus::Skipped { reason: SkipReason::UnsupportedEncoding },
            }),
        }
    }

    /// Fixes a single file by adding the license header.
    #[tracing::instrument(skip(self))]
    fn fix_file(&self, path: &FilePath) -> Result<()> {
        use crate::fixer::inserter::insert_header;
        use crate::fixer::writer::write_atomic;

        // Read the file content
        let content = std::fs::read(path.as_path()).map_err(|source| FixerError::ReadError {
            path: path.as_path().to_path_buf(),
            source,
        })?;

        // Get comment style for this file
        let extension =
            path.extension().map(|ext| ext.as_str().to_string()).unwrap_or_default();
        let style_config = self.config.comment_styles.get(&extension).ok_or_else(|| {
            FixerError::UnsupportedExtension {
                extension: extension.to_string(),
                path: path.as_path().to_path_buf(),
            }
        })?;
        let style = CommentStyle::new(style_config.prefix.clone(), style_config.suffix.clone());

        // Insert the header
        use crate::types::header_types::LicenseHeader;
        let license_header = LicenseHeader::new(self.config.license_header.clone())?;
        let new_content = insert_header(&content, &license_header, &style)?;

        // Write atomically
        write_atomic(path.as_path(), &new_content)?;

        Ok(())
    }
}
