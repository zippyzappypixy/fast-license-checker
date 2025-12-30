//! License header validation and similarity calculation.
//!
//! Provides advanced algorithms for validating license headers,
//! including fuzzy matching for detecting malformed or incomplete headers.

use crate::types::{LicenseHeader, SimilarityScore};

/// Validate a detected header match and return appropriate file status
#[tracing::instrument]
pub fn validate_header_match(
    header_match: &crate::checker::detector::HeaderMatch,
    config_threshold: u8,
) -> crate::types::FileStatus {
    match header_match {
        crate::checker::detector::HeaderMatch::Exact => crate::types::FileStatus::HasHeader,
        crate::checker::detector::HeaderMatch::Fuzzy { similarity } => {
            if *similarity >= config_threshold {
                crate::types::FileStatus::HasHeader
            } else {
                crate::types::FileStatus::MalformedHeader {
                    similarity: SimilarityScore::new(*similarity),
                }
            }
        }
        crate::checker::detector::HeaderMatch::None => crate::types::FileStatus::MissingHeader,
    }
}

/// Calculate Levenshtein distance between two strings (more accurate similarity)
/// Uses a stack-efficient algorithm to avoid O(N*M) heap allocations.
#[tracing::instrument]
#[allow(clippy::arithmetic_side_effects)] // Allowed locally for this specific algorithm
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let b_len = b.chars().count();
    // Re-use a single row buffer to reduce memory overhead
    let mut cache: Vec<usize> = (0..=b_len).collect();

    for (i, a_char) in a.chars().enumerate() {
        let mut prev = i;
        cache[0] = i + 1; // Safe indexing (bounds checked)

        for (j, b_char) in b.chars().enumerate() {
            let current = cache[j + 1];
            let cost = if a_char == b_char { 0 } else { 1 };

            // Calculate minimum operation cost
            let min_cost = std::cmp::min(std::cmp::min(cache[j] + 1, current + 1), prev + cost);

            cache[j + 1] = min_cost;
            prev = current;
        }
    }
    cache[b_len]
}

/// Calculate similarity percentage using Levenshtein distance (0-100)
#[tracing::instrument]
#[allow(clippy::arithmetic_side_effects)]
pub fn levenshtein_similarity(a: &str, b: &str) -> u8 {
    let distance = levenshtein_distance(a, b);
    let max_len = a.len().max(b.len());

    if max_len == 0 {
        return 100;
    }

    let similarity = ((max_len - distance) * 100) / max_len;
    similarity.min(100) as u8
}

/// Advanced fuzzy matching using multiple algorithms
#[tracing::instrument(skip(content, expected))]
#[allow(clippy::arithmetic_side_effects)]
pub fn advanced_fuzzy_match(content: &[u8], expected: &str) -> Option<u8> {
    if content.is_empty() || expected.is_empty() {
        return None;
    }

    // Convert content to string for advanced matching
    let content_str = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // Take first few lines for comparison (limit to reasonable size)
    let content_lines: Vec<&str> = content_str.lines().take(10).collect();
    let expected_lines: Vec<&str> = expected.lines().take(10).collect();

    if content_lines.is_empty() || expected_lines.is_empty() {
        return None;
    }

    // Calculate similarity between corresponding lines
    let mut total_similarity = 0;
    let mut line_count = 0;

    for (content_line, expected_line) in content_lines.iter().zip(expected_lines.iter()) {
        if !content_line.trim().is_empty() || !expected_line.trim().is_empty() {
            let similarity = levenshtein_similarity(content_line, expected_line);
            total_similarity += similarity as u32;
            line_count += 1;
        }
    }

    if line_count == 0 {
        return None;
    }

    let average_similarity = (total_similarity / line_count) as u8;

    // Only return similarity if it's reasonably high
    if average_similarity >= 60 {
        Some(average_similarity)
    } else {
        None
    }
}

/// Validate that a license header conforms to expected format
#[tracing::instrument(skip(header))]
pub fn validate_header_format(header: &LicenseHeader) -> Result<(), String> {
    let text = header.as_str();

    // Basic validation rules
    if text.trim().is_empty() {
        return Err("Header is empty".to_string());
    }

    // Check reasonable length limits
    if text.len() > 5000 {
        return Err("Header is too long (>5KB)".to_string());
    }

    // Check for common license keywords
    let has_license_keyword = ["license", "copyright", "licensed", "permission", "redistribution"]
        .iter()
        .any(|keyword| text.to_lowercase().contains(keyword));

    if !has_license_keyword {
        return Err("Header does not appear to contain license text".to_string());
    }

    Ok(())
}

/// Check if content appears to have a malformed header that should be reported
#[tracing::instrument(skip(content))]
pub fn detect_malformed_header(content: &[u8]) -> Option<String> {
    let start_offset = crate::checker::prelude::effective_header_start(content);
    let search_region = content.get(start_offset..).unwrap_or(&[]);

    // Look for partial license indicators
    let content_str = match std::str::from_utf8(search_region) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let first_lines = content_str.lines().take(5).collect::<Vec<_>>().join("\n");

    // Check for partial matches that indicate a malformed header
    let partial_indicators = ["copyright", "license", "mit", "apache", "gpl", "bsd"];

    for indicator in &partial_indicators {
        if first_lines.to_lowercase().contains(indicator) {
            return Some(format!("Detected partial license text containing '{}'", indicator));
        }
    }

    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::checker::detector::HeaderMatch;

    #[test]
    fn validate_header_match_exact() {
        let header_match = HeaderMatch::Exact;
        let status = validate_header_match(&header_match, 70);
        assert!(matches!(status, crate::types::FileStatus::HasHeader));
    }

    #[test]
    fn validate_header_match_fuzzy_above_threshold() {
        let header_match = HeaderMatch::Fuzzy { similarity: 85 };
        let status = validate_header_match(&header_match, 70);
        assert!(matches!(status, crate::types::FileStatus::HasHeader));
    }

    #[test]
    fn validate_header_match_fuzzy_below_threshold() {
        let header_match = HeaderMatch::Fuzzy { similarity: 50 };
        let status = validate_header_match(&header_match, 70);
        assert!(matches!(status, crate::types::FileStatus::MalformedHeader { .. }));
    }

    #[test]
    fn validate_header_match_none() {
        let header_match = HeaderMatch::None;
        let status = validate_header_match(&header_match, 70);
        assert!(matches!(status, crate::types::FileStatus::MissingHeader));
    }

    #[test]
    fn levenshtein_distance_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn levenshtein_distance_empty() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("hello", ""), 5);
        assert_eq!(levenshtein_distance("", "world"), 5);
    }

    #[test]
    fn levenshtein_distance_substitution() {
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    }

    #[test]
    fn levenshtein_distance_insertion() {
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn levenshtein_distance_deletion() {
        assert_eq!(levenshtein_distance("hello", "ello"), 1);
    }

    #[test]
    fn levenshtein_similarity_perfect() {
        assert_eq!(levenshtein_similarity("hello", "hello"), 100);
    }

    #[test]
    fn levenshtein_similarity_half() {
        assert_eq!(levenshtein_similarity("hello", "helo"), 80); // 4 out of 5 chars match
    }

    #[test]
    fn levenshtein_similarity_none() {
        assert_eq!(levenshtein_similarity("hello", "world"), 20); // 1 out of 5 chars match
    }

    #[test]
    fn advanced_fuzzy_match_good_match() {
        let content = b"// MIT License\n// Copyright 2024";
        let expected = "// MIT License\n// Copyright 2024";

        let similarity = advanced_fuzzy_match(content, expected).unwrap();
        assert!(similarity >= 90);
    }

    #[test]
    fn advanced_fuzzy_match_partial_match() {
        let content = b"// MIT License\n// Wrong text";
        let expected = "// MIT License\n// Copyright 2024";

        let similarity = advanced_fuzzy_match(content, expected).unwrap();
        assert!(similarity >= 60 && similarity < 90);
    }

    #[test]
    fn advanced_fuzzy_match_no_match() {
        let content = b"fn main() {}";
        let expected = "// MIT License";

        assert_eq!(advanced_fuzzy_match(content, expected), None);
    }

    #[test]
    fn validate_header_format_valid() {
        let header = LicenseHeader::new("MIT License\nCopyright 2024".to_string()).unwrap();
        assert!(validate_header_format(&header).is_ok());
    }

    #[test]
    fn validate_header_format_no_keywords() {
        let header =
            LicenseHeader::new("just some random text without keywords".to_string()).unwrap();
        assert!(validate_header_format(&header).is_err());
    }

    #[test]
    fn validate_header_format_too_long() {
        let long_text = "license ".repeat(1000);
        let header = LicenseHeader::new(long_text).unwrap();
        assert!(validate_header_format(&header).is_err());
    }

    #[test]
    fn detect_malformed_header_copyright() {
        let content = b"// Copyright 2024\nfn main() {}";
        assert!(detect_malformed_header(content).is_some());
    }

    #[test]
    fn detect_malformed_header_license() {
        let content = b"/* MIT License */\nclass Test {}";
        assert!(detect_malformed_header(content).is_some());
    }

    #[test]
    fn detect_malformed_header_none() {
        let content = b"fn main() {\n    println!(\"hello\");\n}";
        assert!(detect_malformed_header(content).is_none());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn levenshtein_distance_never_panics(a in ".*", b in ".*") {
            let _ = levenshtein_distance(&a, &b);
        }

        #[test]
        fn levenshtein_distance_symmetric(a in "[a-zA-Z]{1,100}", b in "[a-zA-Z]{1,100}") {
            let d1 = levenshtein_distance(&a, &b);
            let d2 = levenshtein_distance(&b, &a);
            assert_eq!(d1, d2, "Distance should be symmetric");
        }

        #[test]
        fn levenshtein_distance_identity(s in "[a-zA-Z]{1,100}") {
            let d = levenshtein_distance(&s, &s);
            assert_eq!(d, 0, "Distance to self should be zero");
        }

        #[test]
        fn levenshtein_similarity_bounded(a in ".*", b in ".*") {
            let sim = levenshtein_similarity(&a, &b);
            assert!(sim <= 100, "Similarity must be 0-100");
        }

        #[test]
        fn advanced_fuzzy_match_never_panics(content in prop::collection::vec(0u8..255u8, 0..1000), expected in ".*") {
            let _ = advanced_fuzzy_match(&content, &expected);
        }
    }
}
