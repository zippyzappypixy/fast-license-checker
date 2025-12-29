//! Content filtering and binary detection.
//!
//! Provides utilities for detecting binary files, validating encodings,
//! and determining if files should be skipped during scanning.

use crate::config::Config;
use crate::types::SkipReason;

/// Detect if content is likely binary (contains NULL bytes)
#[tracing::instrument(skip(content))]
pub fn is_binary(content: &[u8]) -> bool {
    // Use memchr for fast NULL byte search
    // Binary files typically contain NULL bytes in the first few KB
    memchr::memchr(0, content).is_some()
}

/// Detect if content is valid UTF-8
#[tracing::instrument(skip(content))]
pub fn is_valid_utf8(content: &[u8]) -> bool {
    std::str::from_utf8(content).is_ok()
}

/// Check if file should be skipped based on content and configuration
#[tracing::instrument(skip(content, config))]
pub fn should_skip(content: &[u8], config: &Config) -> Option<SkipReason> {
    // Check for empty files
    if content.is_empty() && config.skip_empty_files {
        return Some(SkipReason::Empty);
    }

    // Check for binary content
    if is_binary(content) {
        return Some(SkipReason::Binary);
    }

    // Check for valid UTF-8 encoding
    if !is_valid_utf8(content) {
        return Some(SkipReason::UnsupportedEncoding);
    }

    None
}

/// Check if a file extension has a configured comment style
#[tracing::instrument]
pub fn has_comment_style(config: &Config, extension: Option<&str>) -> bool {
    extension.and_then(|ext| config.comment_styles.get(ext)).is_some()
}

/// Determine skip reason for files without comment styles
#[tracing::instrument]
pub fn skip_reason_for_extension(config: &Config, extension: Option<&str>) -> Option<SkipReason> {
    if !has_comment_style(config, extension) {
        Some(SkipReason::NoCommentStyle)
    } else {
        None
    }
}

/// Comprehensive file filtering combining all checks
#[tracing::instrument(skip(content, config))]
pub fn should_process_file(
    content: &[u8],
    extension: Option<&str>,
    config: &Config,
) -> Result<(), SkipReason> {
    // First check content-based filters
    if let Some(reason) = should_skip(content, config) {
        return Err(reason);
    }

    // Then check extension-based filters
    if let Some(reason) = skip_reason_for_extension(config, extension) {
        return Err(reason);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn is_binary_with_null_bytes() {
        let content = b"Hello\x00World";
        assert!(is_binary(content));
    }

    #[test]
    fn is_binary_without_null_bytes() {
        let content = b"Hello World";
        assert!(!is_binary(content));
    }

    #[test]
    fn is_binary_empty_content() {
        let content = b"";
        assert!(!is_binary(content));
    }

    #[test]
    fn is_valid_utf8_ascii() {
        let content = b"Hello World";
        assert!(is_valid_utf8(content));
    }

    #[test]
    fn is_valid_utf8_unicode() {
        let content = "Hello 世界 🌍".as_bytes();
        assert!(is_valid_utf8(content));
    }

    #[test]
    fn is_valid_utf8_invalid() {
        let content = &[0xff, 0xfe, 0xfd]; // Invalid UTF-8 sequence
        assert!(!is_valid_utf8(content));
    }

    #[test]
    fn should_skip_empty_file_when_configured() {
        let mut config = Config::default();
        config.skip_empty_files = true;

        let content = b"";
        assert_eq!(should_skip(content, &config), Some(SkipReason::Empty));
    }

    #[test]
    fn should_skip_empty_file_when_not_configured() {
        let mut config = Config::default();
        config.skip_empty_files = false;

        let content = b"";
        assert_eq!(should_skip(content, &config), None);
    }

    #[test]
    fn should_skip_binary_content() {
        let config = Config::default();
        let content = b"Hello\x00World";
        assert_eq!(should_skip(content, &config), Some(SkipReason::Binary));
    }

    #[test]
    fn should_skip_invalid_utf8() {
        let config = Config::default();
        let content = &[0xff, 0xfe, 0xfd];
        assert_eq!(should_skip(content, &config), Some(SkipReason::UnsupportedEncoding));
    }

    #[test]
    fn should_not_skip_valid_content() {
        let config = Config::default();
        let content = b"fn main() { println!(\"Hello World\"); }";
        assert_eq!(should_skip(content, &config), None);
    }

    #[test]
    fn has_comment_style_known_extension() {
        let config = Config::default();
        assert!(has_comment_style(&config, Some("rs")));
        assert!(has_comment_style(&config, Some("py")));
        assert!(has_comment_style(&config, Some("html")));
    }

    #[test]
    fn has_comment_style_unknown_extension() {
        let config = Config::default();
        assert!(!has_comment_style(&config, Some("xyz")));
        assert!(!has_comment_style(&config, Some("")));
    }

    #[test]
    fn has_comment_style_no_extension() {
        let config = Config::default();
        assert!(!has_comment_style(&config, None));
    }

    #[test]
    fn skip_reason_for_extension_with_style() {
        let config = Config::default();
        assert_eq!(skip_reason_for_extension(&config, Some("rs")), None);
    }

    #[test]
    fn skip_reason_for_extension_without_style() {
        let config = Config::default();
        assert_eq!(
            skip_reason_for_extension(&config, Some("xyz")),
            Some(SkipReason::NoCommentStyle)
        );
    }

    #[test]
    fn should_process_file_valid() {
        let config = Config::default();
        let content = b"fn main() {}";
        let result = should_process_file(content, Some("rs"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn should_process_file_skip_empty() {
        let mut config = Config::default();
        config.skip_empty_files = true;

        let content = b"";
        let result = should_process_file(content, Some("rs"), &config);
        assert_eq!(result, Err(SkipReason::Empty));
    }

    #[test]
    fn should_process_file_skip_binary() {
        let config = Config::default();
        let content = b"Hello\x00World";
        let result = should_process_file(content, Some("rs"), &config);
        assert_eq!(result, Err(SkipReason::Binary));
    }

    #[test]
    fn should_process_file_skip_invalid_utf8() {
        let config = Config::default();
        let content = &[0xff, 0xfe, 0xfd];
        let result = should_process_file(content, Some("rs"), &config);
        assert_eq!(result, Err(SkipReason::UnsupportedEncoding));
    }

    #[test]
    fn should_process_file_skip_no_comment_style() {
        let config = Config::default();
        let content = b"some content";
        let result = should_process_file(content, Some("xyz"), &config);
        assert_eq!(result, Err(SkipReason::NoCommentStyle));
    }
}
