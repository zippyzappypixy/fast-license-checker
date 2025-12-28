//! License header and comment-related domain types.
//!
//! Types that represent license headers, comment styles, and similarity scoring.

use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

/// A validated license header that is guaranteed non-empty.
///
/// Ensures license headers are properly formatted and non-empty at creation time.
/// Headers are trimmed and validated to prevent empty or whitespace-only headers.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LicenseHeader(String);

impl LicenseHeader {
    /// Creates a new LicenseHeader, validating that it's not empty.
    ///
    /// # Errors
    /// Returns `ValidationError::EmptyHeader` if the input is empty after trimming.
    pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
        let s = s.into();
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyHeader);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the header as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the header as a byte slice for efficient comparison.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for LicenseHeader {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for LicenseHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Comment style configuration for different file types.
///
/// Defines how to format license headers for different programming languages.
/// Supports both line comments (e.g., `//`, `#`) and block comments (e.g., `/* */`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentStyle {
    /// The prefix for comment lines (e.g., "//", "#", "/*")
    pub prefix: String,
    /// Optional suffix for block comments (e.g., "*/")
    pub suffix: Option<String>,
}

impl CommentStyle {
    /// Creates a new comment style.
    pub fn new(prefix: String, suffix: Option<String>) -> Self {
        Self { prefix, suffix }
    }

    /// Returns a line comment style (e.g., "//" for Rust, "#" for Python).
    pub fn line_comment(prefix: &str) -> Self {
        Self::new(prefix.to_string(), None)
    }

    /// Returns a block comment style (e.g., "/*" and "*/" for CSS).
    pub fn block_comment(prefix: &str, suffix: &str) -> Self {
        Self::new(prefix.to_string(), Some(suffix.to_string()))
    }

    /// Rust-style line comments (`//`).
    pub fn rust_style() -> Self {
        Self::line_comment("//")
    }

    /// Python-style line comments (`#`).
    pub fn python_style() -> Self {
        Self::line_comment("#")
    }

    /// JavaScript-style line comments (`//`).
    pub fn javascript_style() -> Self {
        Self::line_comment("//")
    }

    /// HTML/XML-style block comments (`<!-- -->`).
    pub fn html_style() -> Self {
        Self::block_comment("<!--", "-->")
    }

    /// CSS-style block comments (`/* */`).
    pub fn css_style() -> Self {
        Self::block_comment("/*", "*/")
    }

    /// C/C++-style block comments (`/* */`).
    pub fn c_style() -> Self {
        Self::block_comment("/*", "*/")
    }

    /// Shell-style line comments (`#`).
    pub fn shell_style() -> Self {
        Self::line_comment("#")
    }

    /// YAML-style line comments (`#`).
    pub fn yaml_style() -> Self {
        Self::line_comment("#")
    }

    /// TOML-style line comments (`#`).
    pub fn toml_style() -> Self {
        Self::line_comment("#")
    }

    /// Go-style line comments (`//`).
    pub fn go_style() -> Self {
        Self::line_comment("//")
    }

    /// Returns true if this is a block comment style.
    pub fn is_block_comment(&self) -> bool {
        self.suffix.is_some()
    }

    /// Formats a license header line according to this style.
    ///
    /// For line comments: `prefix header_line`
    /// For block comments: `prefix header_line suffix`
    pub fn format_line(&self, header_line: &str) -> String {
        match &self.suffix {
            Some(suffix) => format!("{}{}{}", self.prefix, header_line, suffix),
            None => format!("{}{}", self.prefix, header_line),
        }
    }
}

impl Default for CommentStyle {
    fn default() -> Self {
        Self::rust_style()
    }
}

impl std::fmt::Display for CommentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.suffix {
            Some(suffix) => write!(f, "{}...{}", self.prefix, suffix),
            None => write!(f, "{}", self.prefix),
        }
    }
}

/// Similarity score for fuzzy header matching (0-100).
///
/// Represents how closely a file's header matches the expected header.
/// 100 = exact match, 0 = no similarity.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimilarityScore(u8);

impl SimilarityScore {
    /// Maximum similarity score (100%).
    pub const MAX: Self = Self(100);
    /// Minimum similarity score (0%).
    pub const MIN: Self = Self(0);

    /// Creates a new SimilarityScore, clamped to 0-100.
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// Creates a SimilarityScore from a percentage (0.0 to 1.0).
    pub fn from_percentage(percentage: f64) -> Self {
        Self::new((percentage * 100.0) as u8)
    }

    /// Returns the score as u8.
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Returns the score as a percentage (0.0 to 1.0).
    pub fn as_percentage(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    /// Returns true if this represents an exact match.
    pub fn is_exact(&self) -> bool {
        self.0 == 100
    }

    /// Returns true if this represents a close match (80% or higher).
    pub fn is_close(&self) -> bool {
        self.0 >= 80
    }
}

impl From<SimilarityScore> for u8 {
    fn from(score: SimilarityScore) -> u8 {
        score.0
    }
}

impl From<SimilarityScore> for f64 {
    fn from(score: SimilarityScore) -> f64 {
        score.as_percentage()
    }
}

impl std::fmt::Display for SimilarityScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_header_new() {
        let header = LicenseHeader::new("MIT License").unwrap();
        assert_eq!(header.as_str(), "MIT License");
    }

    #[test]
    fn license_header_trims_whitespace() {
        let header = LicenseHeader::new("  MIT License  ").unwrap();
        assert_eq!(header.as_str(), "MIT License");
    }

    #[test]
    fn license_header_empty_error() {
        assert!(matches!(LicenseHeader::new("").unwrap_err(), ValidationError::EmptyHeader));
        assert!(matches!(LicenseHeader::new("   ").unwrap_err(), ValidationError::EmptyHeader));
    }

    #[test]
    fn comment_style_line_comment() {
        let style = CommentStyle::line_comment("//");
        assert_eq!(style.prefix, "//");
        assert_eq!(style.suffix, None);
        assert!(!style.is_block_comment());
    }

    #[test]
    fn comment_style_block_comment() {
        let style = CommentStyle::block_comment("/*", "*/");
        assert_eq!(style.prefix, "/*");
        assert_eq!(style.suffix.as_deref(), Some("*/"));
        assert!(style.is_block_comment());
    }

    #[test]
    fn comment_style_format_line() {
        let line_style = CommentStyle::line_comment("//");
        assert_eq!(line_style.format_line(" Copyright 2024"), "// Copyright 2024");

        let block_style = CommentStyle::block_comment("/*", "*/");
        assert_eq!(block_style.format_line(" Copyright 2024 "), "/* Copyright 2024 */");
    }

    #[test]
    fn comment_style_presets() {
        assert_eq!(CommentStyle::rust_style(), CommentStyle::line_comment("//"));
        assert_eq!(CommentStyle::python_style(), CommentStyle::line_comment("#"));
        assert_eq!(CommentStyle::css_style(), CommentStyle::block_comment("/*", "*/"));
    }

    #[test]
    fn similarity_score_new() {
        let score = SimilarityScore::new(85);
        assert_eq!(score.value(), 85);
        assert_eq!(score.as_percentage(), 0.85);
    }

    #[test]
    fn similarity_score_clamped() {
        let score = SimilarityScore::new(150);
        assert_eq!(score.value(), 100);
    }

    #[test]
    fn similarity_score_exact() {
        assert!(SimilarityScore::MAX.is_exact());
        assert!(!SimilarityScore::MIN.is_exact());
    }

    #[test]
    fn similarity_score_close() {
        assert!(SimilarityScore::new(80).is_close());
        assert!(SimilarityScore::new(95).is_close());
        assert!(!SimilarityScore::new(70).is_close());
    }

    #[test]
    fn license_header_as_ref_str() {
        let header = LicenseHeader::new("MIT License").unwrap();
        let str_ref: &str = header.as_ref();
        assert_eq!(str_ref, "MIT License");
    }

    #[test]
    fn license_header_display() {
        let header = LicenseHeader::new("MIT License").unwrap();
        assert_eq!(format!("{}", header), "MIT License");
    }

    #[test]
    fn license_header_from_string() {
        let header = LicenseHeader::new(String::from("MIT License")).unwrap();
        assert_eq!(header.as_str(), "MIT License");
    }

    #[test]
    fn license_header_from_str() {
        let header = LicenseHeader::new("MIT License").unwrap();
        assert_eq!(header.as_str(), "MIT License");
    }

    #[test]
    fn comment_style_as_ref_str() {
        let style = CommentStyle::line_comment("//");
        let str_ref: &str = style.prefix.as_ref();
        assert_eq!(str_ref, "//");
    }

    #[test]
    fn comment_style_display() {
        let line_style = CommentStyle::line_comment("//");
        assert_eq!(format!("{}", line_style), "//");

        let block_style = CommentStyle::block_comment("/*", "*/");
        assert_eq!(format!("{}", block_style), "/*...*/");
    }

    #[test]
    fn comment_style_from_parts() {
        let style = CommentStyle::new("//".to_string(), None);
        assert_eq!(style.prefix, "//");
        assert_eq!(style.suffix, None);
        assert!(!style.is_block_comment());
    }

    #[test]
    fn comment_style_block_format_complex() {
        let style = CommentStyle::block_comment("/*", "*/");
        assert_eq!(style.format_line("/* nested */ comment"), "/*/* nested */ comment*/");
    }

    #[test]
    fn similarity_score_from_percentage() {
        let score = SimilarityScore::from_percentage(0.85);
        assert_eq!(score.value(), 85);

        let score_zero = SimilarityScore::from_percentage(0.0);
        assert_eq!(score_zero.value(), 0);

        let score_one = SimilarityScore::from_percentage(1.0);
        assert_eq!(score_one.value(), 100);
    }

    #[test]
    fn similarity_score_from_u8() {
        let score = SimilarityScore::new(75);
        let value: u8 = score.into();
        assert_eq!(value, 75);
    }

    #[test]
    fn similarity_score_from_f64() {
        let score = SimilarityScore::new(90);
        let percentage: f64 = score.into();
        assert_eq!(percentage, 0.9);
    }

    #[test]
    fn similarity_score_display() {
        let score = SimilarityScore::new(85);
        assert_eq!(format!("{}", score), "85%");

        let max_score = SimilarityScore::MAX;
        assert_eq!(format!("{}", max_score), "100%");
    }

    #[test]
    fn comment_style_all_presets() {
        // Test all preset methods return expected styles
        assert_eq!(CommentStyle::rust_style(), CommentStyle::line_comment("//"));
        assert_eq!(CommentStyle::python_style(), CommentStyle::line_comment("#"));
        assert_eq!(CommentStyle::javascript_style(), CommentStyle::line_comment("//"));
        assert_eq!(CommentStyle::shell_style(), CommentStyle::line_comment("#"));
        assert_eq!(CommentStyle::yaml_style(), CommentStyle::line_comment("#"));
        assert_eq!(CommentStyle::toml_style(), CommentStyle::line_comment("#"));
        assert_eq!(CommentStyle::go_style(), CommentStyle::line_comment("//"));

        // Block comment presets
        assert_eq!(CommentStyle::html_style(), CommentStyle::block_comment("<!--", "-->"));
        assert_eq!(CommentStyle::css_style(), CommentStyle::block_comment("/*", "*/"));
        assert_eq!(CommentStyle::c_style(), CommentStyle::block_comment("/*", "*/"));
    }

    #[test]
    fn license_header_as_bytes() {
        let header = LicenseHeader::new("MIT License").unwrap();
        assert_eq!(header.as_bytes(), header.as_str().as_bytes());
        assert_eq!(header.as_bytes(), b"MIT License");
    }

    #[test]
    fn comment_style_default() {
        let default_style = CommentStyle::default();
        let rust_style = CommentStyle::rust_style();
        assert_eq!(default_style, rust_style);
        assert_eq!(default_style.prefix, "//");
        assert_eq!(default_style.suffix, None);
    }
}
