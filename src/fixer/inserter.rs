//! Header insertion and formatting logic.
//!
//! Provides functions to format license headers with appropriate comment styles
//! and insert them into source files at the correct positions.

use crate::checker::prelude::header_start_offset;
use crate::error::Result;
use crate::types::{CommentStyle, LicenseHeader};

/// Format license header with appropriate comment style
#[tracing::instrument(skip(header))]
pub fn format_header(
    header: &LicenseHeader,
    style: &CommentStyle,
) -> String {
    let mut result = String::new();

    for line in header.as_str().lines() {
        result.push_str(&style.prefix);
        if !line.is_empty() {
            result.push(' ');
            result.push_str(line);
        }
        if let Some(ref suffix) = style.suffix {
            result.push(' ');
            result.push_str(suffix);
        }
        result.push('\n');
    }

    // Add blank line after header
    result.push('\n');

    result
}

/// Insert header into content at the correct position
#[tracing::instrument(skip(content, header))]
pub fn insert_header(
    content: &[u8],
    header: &LicenseHeader,
    style: &CommentStyle,
) -> Result<Vec<u8>> {
    let insert_offset = header_start_offset(content);
    let formatted = format_header(header, style);

    let mut result = Vec::with_capacity(content.len() + formatted.len());

    // Copy content before insertion point (shebang/xml)
    result.extend_from_slice(&content[..insert_offset]);

    // Insert header
    result.extend_from_slice(formatted.as_bytes());

    // Copy rest of content
    result.extend_from_slice(&content[insert_offset..]);

    Ok(result)
}

/// Check if content already contains a license header
#[tracing::instrument(skip(content, header))]
pub fn contains_header(
    content: &[u8],
    header: &LicenseHeader,
    style: &CommentStyle,
) -> bool {
    let formatted = format_header(header, style);
    let formatted_bytes = formatted.as_bytes();

    // Look for the formatted header in the content
    // Start from the header insertion point
    let start_offset = header_start_offset(content);

    if start_offset + formatted_bytes.len() <= content.len() {
        content[start_offset..start_offset + formatted_bytes.len()] == *formatted_bytes
    } else {
        false
    }
}

/// Remove existing header from content (for replacement)
#[tracing::instrument(skip(content, header))]
pub fn remove_header(
    content: &[u8],
    header: &LicenseHeader,
    style: &CommentStyle,
) -> Result<Vec<u8>> {
    let formatted = format_header(header, style);
    let formatted_bytes = formatted.as_bytes();

    let start_offset = header_start_offset(content);

    // Check if header exists at expected location
    if start_offset + formatted_bytes.len() <= content.len()
        && content[start_offset..start_offset + formatted_bytes.len()] == *formatted_bytes
    {
        // Remove the header
        let mut result = Vec::with_capacity(content.len() - formatted_bytes.len());
        result.extend_from_slice(&content[..start_offset]);
        result.extend_from_slice(&content[start_offset + formatted_bytes.len()..]);
        Ok(result)
    } else {
        // Header not found, return original content
        Ok(content.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CommentStyle;

    fn create_test_header() -> LicenseHeader {
        let header_text = vec!["MIT License", "", "Copyright 2024 Test"].join("\n");
        LicenseHeader::new(header_text).unwrap()
    }

    fn create_line_style() -> CommentStyle {
        CommentStyle {
            prefix: "//".to_string(),
            suffix: None,
        }
    }

    fn create_block_style() -> CommentStyle {
        CommentStyle {
            prefix: "/*".to_string(),
            suffix: Some("*/".to_string()),
        }
    }

    #[test]
    fn format_header_line_comments() {
        let header = create_test_header();
        let style = create_line_style();

        let formatted = format_header(&header, &style);
        let expected = "// MIT License\n//\n// Copyright 2024 Test\n\n";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn format_header_block_comments() {
        let header = create_test_header();
        let style = create_block_style();

        let formatted = format_header(&header, &style);
        let expected = "/* MIT License */\n/* */\n/* Copyright 2024 Test */\n\n";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn insert_header_at_start() {
        let header = create_test_header();
        let style = create_line_style();

        let content = b"fn main() {}\n";
        let result = insert_header(content, &header, &style).unwrap();

        let expected_start = b"// MIT License\n//\n// Copyright 2024 Test\n\nfn main() {}\n";
        assert_eq!(&result[..expected_start.len()], expected_start);
    }

    #[test]
    fn insert_header_after_shebang() {
        let header = create_test_header();
        let style = create_line_style();

        let content = b"#!/bin/bash\necho hello\n";
        let result = insert_header(content, &header, &style).unwrap();

        let expected = b"#!/bin/bash\n// MIT License\n//\n// Copyright 2024 Test\n\necho hello\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn contains_header_present() {
        let header = create_test_header();
        let style = create_line_style();

        let formatted = format_header(&header, &style);
        let content = format!("{}\nfn main() {{}}", formatted);

        assert!(contains_header(content.as_bytes(), &header, &style));
    }

    #[test]
    fn contains_header_absent() {
        let header = create_test_header();
        let style = create_line_style();

        let content = "fn main() {}\n".to_string();

        assert!(!contains_header(content.as_bytes(), &header, &style));
    }

    #[test]
    fn contains_header_wrong_position() {
        let header = create_test_header();
        let style = create_line_style();

        let content = "fn main() {}\n// MIT License\n".to_string();

        assert!(!contains_header(content.as_bytes(), &header, &style));
    }

    #[test]
    fn remove_header_present() {
        let header = create_test_header();
        let style = create_line_style();

        let formatted = format_header(&header, &style);
        let original_content = "fn main() {}\n".to_string();
        let content_with_header = format!("{}{}", formatted, original_content);

        let result = remove_header(content_with_header.as_bytes(), &header, &style).unwrap();

        assert_eq!(std::str::from_utf8(&result).unwrap(), original_content);
    }

    #[test]
    fn remove_header_absent() {
        let header = create_test_header();
        let style = create_line_style();

        let content = b"fn main() {}\n";
        let result = remove_header(content, &header, &style).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn insert_header_preserves_content() {
        let header = create_test_header();
        let style = create_line_style();

        let original = "package main\n\nfunc main() {\n\tfmt.Println(\"hello\")\n}\n";
        let result = insert_header(original.as_bytes(), &header, &style).unwrap();

        // Should contain original content after header
        let result_str = std::str::from_utf8(&result).unwrap();
        assert!(result_str.ends_with(original));
    }
}
