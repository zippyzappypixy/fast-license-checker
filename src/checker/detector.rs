//! License header detection and matching.
//!
//! Provides algorithms for detecting license headers in source files,
//! including exact matching and fuzzy matching for malformed headers.

use crate::types::{CommentStyle, LicenseHeader};

/// Result of header detection attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderMatch {
    /// Header found with exact match
    Exact,
    /// Header found with fuzzy match (similarity percentage)
    Fuzzy {
        /// Similarity score (0-100) indicating how closely the detected header matches the expected one
        similarity: u8,
    },
    /// No header found
    None,
}

/// Check if the expected header is present in content
#[tracing::instrument(skip(content, expected))]
pub fn detect_header(
    content: &[u8],
    expected: &LicenseHeader,
    style: &CommentStyle,
) -> HeaderMatch {
    let start_offset = crate::checker::prelude::effective_header_start(content);
    let search_region = &content[start_offset..];

    // Format expected header with comment style
    let formatted_header = format_header_for_search(expected, style);

    // Exact match
    if search_region.starts_with(formatted_header.as_bytes()) {
        return HeaderMatch::Exact;
    }

    // Fuzzy match for malformed headers
    if let Some(similarity) = fuzzy_match(search_region, &formatted_header) {
        if similarity >= 70 {
            return HeaderMatch::Fuzzy { similarity };
        }
    }

    HeaderMatch::None
}

/// Format a license header for search using the given comment style
#[tracing::instrument(skip(header))]
pub fn format_header_for_search(header: &LicenseHeader, style: &CommentStyle) -> String {
    let header_text = header.as_str();

    if let Some(suffix) = &style.suffix {
        // Block comment style (/* */)
        let mut result = String::new();
        result.push_str(&style.prefix);
        result.push('\n');

        // Add each line with proper formatting
        for line in header_text.lines() {
            if !line.is_empty() {
                result.push_str(line);
                result.push('\n');
            } else {
                // Empty lines in header
                result.push('\n');
            }
        }

        result.push_str(suffix);
        result
    } else {
        // Line comment style (//, #, etc.)
        let mut result = String::new();
        for line in header_text.lines() {
            if !line.is_empty() {
                result.push_str(&style.prefix);
                result.push(' ');
                result.push_str(line);
                result.push('\n');
            } else {
                // Empty lines in header remain empty
                result.push('\n');
            }
        }
        result
    }
}

/// Perform fuzzy matching between content and expected header
#[tracing::instrument(skip(content, expected))]
pub fn fuzzy_match(content: &[u8], expected: &str) -> Option<u8> {
    if content.is_empty() || expected.is_empty() {
        return None;
    }

    // Convert expected to bytes for comparison
    let expected_bytes = expected.as_bytes();

    // Simple similarity calculation: compare first N bytes
    let min_len = content.len().min(expected_bytes.len()).min(256); // Limit to first 256 bytes

    if min_len < 10 {
        return None; // Too short to be meaningful
    }

    let content_prefix = &content[..min_len];
    let expected_prefix = &expected_bytes[..min_len];

    let similarity = calculate_byte_similarity(content_prefix, expected_prefix);

    if similarity >= 70 {
        Some(similarity)
    } else {
        None
    }
}

/// Calculate similarity between two byte slices (0-100)
#[tracing::instrument]
pub fn calculate_byte_similarity(a: &[u8], b: &[u8]) -> u8 {
    if a.is_empty() && b.is_empty() {
        return 100;
    }

    // Find length of common prefix
    let prefix_len = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();

    if prefix_len == 0 {
        return 0;
    }

    // Similarity is based on how much of the shorter string matches
    let shorter_len = a.len().min(b.len());
    ((prefix_len * 100) / shorter_len).min(100) as u8
}

/// Check if content contains any license header (heuristic)
#[tracing::instrument(skip(content))]
pub fn contains_any_license_header(content: &[u8]) -> bool {
    let start_offset = crate::checker::prelude::effective_header_start(content);
    let search_region = &content[start_offset..];

    // Look for common license keywords in first few lines
    let content_str = match std::str::from_utf8(search_region) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let first_lines = content_str.lines().take(10).collect::<Vec<_>>().join("\n");

    // Common license indicators (case-insensitive)
    let license_keywords = [
        "copyright",
        "license",
        "licensed under",
        "mit license",
        "apache license",
        "gpl",
        "lgpl",
        "bsd license",
        "mozilla public license",
        "isc license",
    ];

    for keyword in &license_keywords {
        if first_lines.to_lowercase().contains(keyword) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CommentStyle;

    fn create_test_header() -> LicenseHeader {
        LicenseHeader::new("MIT License\n\nCopyright 2024 Test".to_string()).unwrap()
    }

    fn create_line_style() -> CommentStyle {
        CommentStyle { prefix: "//".to_string(), suffix: None }
    }

    fn create_block_style() -> CommentStyle {
        CommentStyle { prefix: "/*".to_string(), suffix: Some("*/".to_string()) }
    }

    #[test]
    fn format_header_for_search_line_comments() {
        let header = create_test_header();
        let style = create_line_style();

        let formatted = format_header_for_search(&header, &style);
        let expected = "// MIT License\n\n// Copyright 2024 Test\n";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn format_header_for_search_block_comments() {
        let header = create_test_header();
        let style = create_block_style();

        let formatted = format_header_for_search(&header, &style);
        let expected = "/*\nMIT License\n\nCopyright 2024 Test\n*/";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn detect_header_exact_match() {
        let header = create_test_header();
        let style = create_line_style();

        let formatted = format_header_for_search(&header, &style);
        let content = format!("{}\nfn main() {{}}", formatted);

        let result = detect_header(content.as_bytes(), &header, &style);
        assert_eq!(result, HeaderMatch::Exact);
    }

    #[test]
    fn detect_header_after_shebang() {
        let header = create_test_header();
        let style = create_line_style();

        let formatted = format_header_for_search(&header, &style);
        let content = format!("#!/usr/bin/env python3\n{}", formatted);

        let result = detect_header(content.as_bytes(), &header, &style);
        assert_eq!(result, HeaderMatch::Exact);
    }

    #[test]
    fn detect_header_no_match() {
        let header = create_test_header();
        let style = create_line_style();

        let content = "fn main() {}".to_string();

        let result = detect_header(content.as_bytes(), &header, &style);
        assert_eq!(result, HeaderMatch::None);
    }

    #[test]
    fn calculate_byte_similarity_identical() {
        let a = b"hello world";
        let b = b"hello world";
        assert_eq!(calculate_byte_similarity(a, b), 100);
    }

    #[test]
    fn calculate_byte_similarity_different() {
        let a = b"hello";
        let b = b"world";
        // No common prefix
        assert_eq!(calculate_byte_similarity(a, b), 0);
    }

    #[test]
    fn calculate_byte_similarity_partial() {
        let a = b"hello world";
        let b = b"hello there";
        // Common prefix "hello " (6 bytes) out of shorter string length 11
        assert_eq!(calculate_byte_similarity(a, b), 54);
    }

    #[test]
    fn fuzzy_match_high_similarity() {
        let content = b"// MIT License";
        let expected = "// MIT License";

        assert_eq!(fuzzy_match(content, expected), Some(100));
    }

    #[test]
    fn fuzzy_match_low_similarity() {
        let content = b"fn main()";
        let expected = "// MIT License";

        assert_eq!(fuzzy_match(content, expected), None);
    }

    #[test]
    fn fuzzy_match_too_short() {
        let content = b"hi";
        let expected = "// MIT License";

        assert_eq!(fuzzy_match(content, expected), None);
    }

    #[test]
    fn contains_any_license_header_mit() {
        let content = b"// MIT License\nfn main() {}";
        assert!(contains_any_license_header(content));
    }

    #[test]
    fn contains_any_license_header_apache() {
        let content = b"/* Apache License */\nclass Test {}";
        assert!(contains_any_license_header(content));
    }

    #[test]
    fn contains_any_license_header_no_license() {
        let content = b"fn main() { println!(\"hello\"); }";
        assert!(!contains_any_license_header(content));
    }

    #[test]
    fn contains_any_license_header_after_shebang() {
        let content = b"#!/bin/bash\n# MIT License\necho hello";
        assert!(contains_any_license_header(content));
    }
}
