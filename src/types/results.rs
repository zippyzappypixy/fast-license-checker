//! Scan and fix result types.
//!
//! Types that represent the outcomes of scanning and fixing operations.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{FilePath, SimilarityScore};

/// The status of a file's license header check.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileStatus {
    /// File has the correct license header.
    HasHeader,
    /// File is missing a license header.
    MissingHeader,
    /// File has a malformed header (with similarity score).
    MalformedHeader {
        /// How similar the header is to the expected one (0-100).
        similarity: SimilarityScore,
    },
    /// File was skipped during scanning.
    Skipped {
        /// Why the file was skipped.
        reason: SkipReason,
    },
}

impl FileStatus {
    /// Returns true if the file has a valid header.
    pub fn has_valid_header(&self) -> bool {
        matches!(self, FileStatus::HasHeader)
    }

    /// Returns true if the file is missing a header.
    pub fn is_missing_header(&self) -> bool {
        matches!(self, FileStatus::MissingHeader)
    }

    /// Returns true if the file has a malformed header.
    pub fn is_malformed_header(&self) -> bool {
        matches!(self, FileStatus::MalformedHeader { .. })
    }

    /// Returns true if the file was skipped.
    pub fn is_skipped(&self) -> bool {
        matches!(self, FileStatus::Skipped { .. })
    }

    /// Returns the similarity score if this is a malformed header.
    pub fn similarity_score(&self) -> Option<SimilarityScore> {
        match self {
            FileStatus::MalformedHeader { similarity } => Some(*similarity),
            _ => None,
        }
    }

    /// Returns the skip reason if this file was skipped.
    pub fn skip_reason(&self) -> Option<&SkipReason> {
        match self {
            FileStatus::Skipped { reason } => Some(reason),
            _ => None,
        }
    }
}

impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileStatus::HasHeader => write!(f, "has header"),
            FileStatus::MissingHeader => write!(f, "missing header"),
            FileStatus::MalformedHeader { similarity } => {
                write!(f, "malformed header ({} similar)", similarity)
            }
            FileStatus::Skipped { reason } => write!(f, "skipped ({})", reason),
        }
    }
}

/// Reasons why a file might be skipped during scanning.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::Binary => write!(f, "binary file"),
            SkipReason::Empty => write!(f, "empty file"),
            SkipReason::Gitignored => write!(f, "gitignored"),
            SkipReason::TooLarge => write!(f, "too large"),
            SkipReason::UnsupportedEncoding => write!(f, "unsupported encoding"),
            SkipReason::NoCommentStyle => write!(f, "no comment style"),
        }
    }
}

/// The mode of operation for the license checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScanMode {
    /// Only check files, don't modify them.
    Check,
    /// Check and fix files with missing headers.
    Fix,
}

impl std::fmt::Display for ScanMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanMode::Check => write!(f, "check"),
            ScanMode::Fix => write!(f, "fix"),
        }
    }
}

/// The result of attempting to fix a file's license header.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FixAction {
    /// File was successfully fixed with a license header.
    Fixed,
    /// File already had the correct header.
    AlreadyHasHeader,
    /// File was skipped during fixing.
    Skipped {
        /// Why the file was skipped.
        reason: SkipReason,
    },
    /// File would be fixed (preview mode).
    WouldFix,
    /// Fixing failed with an error message.
    Failed {
        /// The error message describing what went wrong.
        error: String,
    },
}

impl FixAction {
    /// Returns true if the fix was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, FixAction::Fixed | FixAction::AlreadyHasHeader)
    }

    /// Returns true if the file was skipped.
    pub fn is_skipped(&self) -> bool {
        matches!(self, FixAction::Skipped { .. })
    }

    /// Returns true if the fix failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, FixAction::Failed { .. })
    }

    /// Returns the skip reason if this action was skipped.
    pub fn skip_reason(&self) -> Option<&SkipReason> {
        match self {
            FixAction::Skipped { reason } => Some(reason),
            _ => None,
        }
    }

    /// Returns the error message if this action failed.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            FixAction::Failed { error } => Some(error),
            _ => None,
        }
    }
}

impl std::fmt::Display for FixAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixAction::Fixed => write!(f, "fixed"),
            FixAction::AlreadyHasHeader => write!(f, "already has header"),
            FixAction::Skipped { reason } => write!(f, "skipped ({})", reason),
            FixAction::WouldFix => write!(f, "would fix"),
            FixAction::Failed { error } => write!(f, "failed: {}", error),
        }
    }
}

/// The result of checking a single file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScanResult {
    /// The path to the file that was checked.
    pub path: FilePath,
    /// The status of the file's license header.
    pub status: FileStatus,
}

impl ScanResult {
    /// Creates a new scan result.
    pub fn new(path: FilePath, status: FileStatus) -> Self {
        Self { path, status }
    }

    /// Returns true if this result represents a successful check.
    pub fn is_success(&self) -> bool {
        self.status.has_valid_header()
    }

    /// Returns true if this result requires attention (missing or malformed header).
    pub fn needs_attention(&self) -> bool {
        self.status.is_missing_header() || self.status.is_malformed_header()
    }
}

impl std::fmt::Display for ScanResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.status)
    }
}

/// The result of attempting to fix a single file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixResult {
    /// The path to the file that was processed.
    pub path: FilePath,
    /// The action taken on the file.
    pub action: FixAction,
}

impl FixResult {
    /// Creates a new fix result.
    pub fn new(path: FilePath, action: FixAction) -> Self {
        Self { path, action }
    }

    /// Returns true if the fix was successful.
    pub fn is_success(&self) -> bool {
        self.action.is_success()
    }
}

impl std::fmt::Display for FixResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.action)
    }
}

/// Summary of a complete scan operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl ScanSummary {
    /// Creates a new scan summary.
    pub fn new(
        total: usize,
        passed: usize,
        failed: usize,
        skipped: usize,
        duration: Duration,
    ) -> Self {
        Self { total, passed, failed, skipped, duration }
    }

    /// Returns the number of files that need attention (failed + skipped).
    pub fn needs_attention(&self) -> usize {
        #[allow(clippy::arithmetic_side_effects)]
        {
            self.failed + self.skipped
        }
    }

    /// Returns true if all files passed (no failures or skips).
    pub fn is_clean(&self) -> bool {
        self.failed == 0 && self.skipped == 0
    }

    /// Returns the success rate as a percentage (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
}

impl Default for ScanSummary {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, Duration::default())
    }
}

impl std::fmt::Display for ScanSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Scanned {} files in {:.2}s: {} passed, {} failed, {} skipped ({:.1}% success)",
            self.total,
            self.duration.as_secs_f64(),
            self.passed,
            self.failed,
            self.skipped,
            self.success_rate() * 100.0
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn file_status_has_valid_header() {
        assert!(FileStatus::HasHeader.has_valid_header());
        assert!(!FileStatus::MissingHeader.has_valid_header());
        assert!(!FileStatus::MalformedHeader { similarity: SimilarityScore::new(50) }
            .has_valid_header());
    }

    #[test]
    fn file_status_similarity_score() {
        let status = FileStatus::MalformedHeader { similarity: SimilarityScore::new(75) };
        assert_eq!(status.similarity_score(), Some(SimilarityScore::new(75)));

        let status = FileStatus::HasHeader;
        assert_eq!(status.similarity_score(), None);
    }

    #[test]
    fn fix_action_is_success() {
        assert!(FixAction::Fixed.is_success());
        assert!(FixAction::AlreadyHasHeader.is_success());
        assert!(!FixAction::Skipped { reason: SkipReason::Binary }.is_success());
        assert!(!FixAction::Failed { error: "test".to_string() }.is_success());
    }

    #[test]
    fn scan_result_needs_attention() {
        let result = ScanResult::new(FilePath::new("test.txt".into()), FileStatus::MissingHeader);
        assert!(result.needs_attention());

        let result = ScanResult::new(FilePath::new("test.txt".into()), FileStatus::HasHeader);
        assert!(!result.needs_attention());
    }

    #[test]
    fn scan_summary_success_rate() {
        let summary = ScanSummary::new(100, 80, 15, 5, Duration::from_secs(1));
        assert_eq!(summary.success_rate(), 0.8);

        let empty_summary = ScanSummary::default();
        assert_eq!(empty_summary.success_rate(), 0.0);
    }

    #[test]
    fn scan_summary_is_clean() {
        let clean = ScanSummary::new(10, 10, 0, 0, Duration::from_secs(1));
        assert!(clean.is_clean());

        let dirty = ScanSummary::new(10, 8, 1, 1, Duration::from_secs(1));
        assert!(!dirty.is_clean());
    }

    // FileStatus additional tests
    #[test]
    fn file_status_is_missing_header() {
        assert!(FileStatus::MissingHeader.is_missing_header());
        assert!(!FileStatus::HasHeader.is_missing_header());
        assert!(!FileStatus::Skipped { reason: SkipReason::Binary }.is_missing_header());
    }

    #[test]
    fn file_status_is_malformed_header() {
        assert!(FileStatus::MalformedHeader { similarity: SimilarityScore::new(50) }
            .is_malformed_header());
        assert!(!FileStatus::HasHeader.is_malformed_header());
        assert!(!FileStatus::MissingHeader.is_malformed_header());
    }

    #[test]
    fn file_status_is_skipped() {
        assert!(FileStatus::Skipped { reason: SkipReason::Binary }.is_skipped());
        assert!(!FileStatus::HasHeader.is_skipped());
        assert!(!FileStatus::MissingHeader.is_skipped());
    }

    #[test]
    fn file_status_skip_reason() {
        let status = FileStatus::Skipped { reason: SkipReason::Binary };
        assert_eq!(status.skip_reason(), Some(&SkipReason::Binary));

        let status = FileStatus::HasHeader;
        assert_eq!(status.skip_reason(), None);
    }

    #[test]
    fn file_status_display() {
        assert_eq!(FileStatus::HasHeader.to_string(), "has header");
        assert_eq!(FileStatus::MissingHeader.to_string(), "missing header");
        assert_eq!(
            FileStatus::MalformedHeader { similarity: SimilarityScore::new(75) }.to_string(),
            "malformed header (75% similar)"
        );
        assert_eq!(
            FileStatus::Skipped { reason: SkipReason::Binary }.to_string(),
            "skipped (binary file)"
        );
    }

    // SkipReason tests
    #[test]
    fn skip_reason_display() {
        assert_eq!(SkipReason::Binary.to_string(), "binary file");
        assert_eq!(SkipReason::Empty.to_string(), "empty file");
        assert_eq!(SkipReason::Gitignored.to_string(), "gitignored");
        assert_eq!(SkipReason::TooLarge.to_string(), "too large");
        assert_eq!(SkipReason::UnsupportedEncoding.to_string(), "unsupported encoding");
        assert_eq!(SkipReason::NoCommentStyle.to_string(), "no comment style");
    }

    // ScanMode tests
    #[test]
    fn scan_mode_display() {
        assert_eq!(ScanMode::Check.to_string(), "check");
        assert_eq!(ScanMode::Fix.to_string(), "fix");
    }

    // FixAction additional tests
    #[test]
    fn fix_action_is_skipped() {
        assert!(FixAction::Skipped { reason: SkipReason::Binary }.is_skipped());
        assert!(!FixAction::Fixed.is_skipped());
        assert!(!FixAction::AlreadyHasHeader.is_skipped());
    }

    #[test]
    fn fix_action_is_failed() {
        assert!(FixAction::Failed { error: "test".to_string() }.is_failed());
        assert!(!FixAction::Fixed.is_failed());
        assert!(!FixAction::AlreadyHasHeader.is_failed());
    }

    #[test]
    fn fix_action_skip_reason() {
        let action = FixAction::Skipped { reason: SkipReason::Binary };
        assert_eq!(action.skip_reason(), Some(&SkipReason::Binary));

        let action = FixAction::Fixed;
        assert_eq!(action.skip_reason(), None);
    }

    #[test]
    fn fix_action_error_message() {
        let action = FixAction::Failed { error: "test error".to_string() };
        assert_eq!(action.error_message(), Some("test error"));

        let action = FixAction::Fixed;
        assert_eq!(action.error_message(), None);
    }

    #[test]
    fn fix_action_display() {
        assert_eq!(FixAction::Fixed.to_string(), "fixed");
        assert_eq!(FixAction::AlreadyHasHeader.to_string(), "already has header");
        assert_eq!(
            FixAction::Skipped { reason: SkipReason::Binary }.to_string(),
            "skipped (binary file)"
        );
        assert_eq!(
            FixAction::Failed { error: "test error".to_string() }.to_string(),
            "failed: test error"
        );
    }

    // ScanResult additional tests
    #[test]
    fn scan_result_is_success() {
        let result = ScanResult::new(FilePath::new("test.txt".into()), FileStatus::HasHeader);
        assert!(result.is_success());

        let result = ScanResult::new(FilePath::new("test.txt".into()), FileStatus::MissingHeader);
        assert!(!result.is_success());
    }

    #[test]
    fn scan_result_display() {
        let result = ScanResult::new(FilePath::new("test.txt".into()), FileStatus::HasHeader);
        assert_eq!(result.to_string(), "test.txt: has header");
    }

    // FixResult tests
    #[test]
    fn fix_result_is_success() {
        let result = FixResult::new(FilePath::new("test.txt".into()), FixAction::Fixed);
        assert!(result.is_success());

        let result = FixResult::new(
            FilePath::new("test.txt".into()),
            FixAction::Failed { error: "test".to_string() },
        );
        assert!(!result.is_success());
    }

    #[test]
    fn fix_result_display() {
        let result = FixResult::new(FilePath::new("test.txt".into()), FixAction::Fixed);
        assert_eq!(result.to_string(), "test.txt: fixed");
    }

    // ScanSummary additional tests
    #[test]
    fn scan_summary_new() {
        let summary = ScanSummary::new(100, 80, 15, 5, Duration::from_secs(2));
        assert_eq!(summary.total, 100);
        assert_eq!(summary.passed, 80);
        assert_eq!(summary.failed, 15);
        assert_eq!(summary.skipped, 5);
        assert_eq!(summary.duration, Duration::from_secs(2));
    }

    #[test]
    fn scan_summary_needs_attention() {
        let summary = ScanSummary::new(100, 80, 15, 5, Duration::from_secs(1));
        assert_eq!(summary.needs_attention(), 20); // failed + skipped

        let clean = ScanSummary::new(100, 100, 0, 0, Duration::from_secs(1));
        assert_eq!(clean.needs_attention(), 0);
    }

    #[test]
    fn scan_summary_display() {
        let summary = ScanSummary::new(100, 80, 15, 5, Duration::from_millis(2500));
        let display = summary.to_string();
        assert!(display.contains("100 files"));
        assert!(display.contains("2.50s"));
        assert!(display.contains("80 passed"));
        assert!(display.contains("15 failed"));
        assert!(display.contains("5 skipped"));
        assert!(display.contains("80.0% success"));
    }

    #[test]
    fn scan_summary_default() {
        let default = ScanSummary::default();
        assert_eq!(default.total, 0);
        assert_eq!(default.passed, 0);
        assert_eq!(default.failed, 0);
        assert_eq!(default.skipped, 0);
        assert_eq!(default.duration, Duration::default());
    }
}
