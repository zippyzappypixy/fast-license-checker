//! Preliminary content analysis for license header detection.
//!
//! Handles detection of shebang lines, XML declarations, and other content
//! that may precede license headers in source files.

/// Detect shebang line and return byte offset after it
#[tracing::instrument]
pub fn detect_shebang(content: &[u8]) -> Option<usize> {
    if content.starts_with(b"#!") {
        // Find end of first line
        memchr::memchr(b'\n', content).and_then(|pos| pos.checked_add(1))
    } else {
        None
    }
}

/// Detect XML declaration and return byte offset after it
#[tracing::instrument]
pub fn detect_xml_declaration(content: &[u8]) -> Option<usize> {
    if content.starts_with(b"<?xml") {
        // Find closing ?>
        content.windows(2).position(|w| w == b"?>").and_then(|pos| {
            // Skip past ?> and any following newline
            let end = pos.checked_add(2)?;
            match content.get(end) {
                Some(&b'\n') => end.checked_add(1),
                _ => Some(end),
            }
        })
    } else {
        None
    }
}

/// Get the byte offset where header should start (after shebang/xml)
#[tracing::instrument]
pub fn header_start_offset(content: &[u8]) -> usize {
    detect_shebang(content).or_else(|| detect_xml_declaration(content)).unwrap_or(0)
}

/// Detect common hashbang patterns and return offset after them
#[tracing::instrument]
pub fn detect_hashbang(content: &[u8]) -> Option<usize> {
    if content.starts_with(b"#!/") {
        // Unix-style shebang: #!/path/to/interpreter
        memchr::memchr(b'\n', content).and_then(|pos| pos.checked_add(1))
    } else if content.starts_with(b"# -*- coding:") {
        // Python encoding declaration
        memchr::memchr(b'\n', content).and_then(|pos| pos.checked_add(1))
    } else if content.starts_with(b"# vim:") {
        // Vim modeline
        memchr::memchr(b'\n', content).and_then(|pos| pos.checked_add(1))
    } else {
        None
    }
}

/// Get the effective header start offset considering all possible prefixes
#[tracing::instrument]
pub fn effective_header_start(content: &[u8]) -> usize {
    detect_shebang(content)
        .or_else(|| detect_xml_declaration(content))
        .or_else(|| detect_hashbang(content))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_shebang_unix() {
        let content = b"#!/usr/bin/env python3\nprint('hello')";
        assert_eq!(detect_shebang(content), Some(23));
    }

    #[test]
    fn detect_shebang_no_newline() {
        let content = b"#!/bin/bash";
        assert_eq!(detect_shebang(content), None); // No newline, so no end
    }

    #[test]
    fn detect_shebang_no_shebang() {
        let content = b"print('hello')";
        assert_eq!(detect_shebang(content), None);
    }

    #[test]
    fn test_detect_xml_declaration() {
        let content = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root>";
        assert_eq!(detect_xml_declaration(content), Some(39));
    }

    #[test]
    fn detect_xml_declaration_no_newline() {
        let content = b"<?xml version=\"1.0\"?><root>";
        assert_eq!(detect_xml_declaration(content), Some(21));
    }

    #[test]
    fn detect_xml_declaration_no_xml() {
        let content = b"<root>";
        assert_eq!(detect_xml_declaration(content), None);
    }

    #[test]
    fn header_start_offset_shebang() {
        let content = b"#!/bin/bash\necho hello";
        assert_eq!(header_start_offset(content), 12);
    }

    #[test]
    fn header_start_offset_xml() {
        let content = b"<?xml version=\"1.0\"?><root>";
        assert_eq!(header_start_offset(content), 21);
    }

    #[test]
    fn header_start_offset_no_prefix() {
        let content = b"package main";
        assert_eq!(header_start_offset(content), 0);
    }

    #[test]
    fn header_start_offset_shebang_precedence() {
        // Shebang should take precedence over XML
        let content = b"#!/bin/bash\n<?xml version=\"1.0\"?>";
        assert_eq!(header_start_offset(content), 12);
    }

    #[test]
    fn detect_hashbang_python_encoding() {
        let content = b"# -*- coding: utf-8 -*-\nprint('hello')";
        assert_eq!(detect_hashbang(content), Some(24));
    }

    #[test]
    fn detect_hashbang_vim_modeline() {
        let content = b"# vim: set ft=ruby:\nputs 'hello'";
        assert_eq!(detect_hashbang(content), Some(20));
    }

    #[test]
    fn detect_hashbang_no_hashbang() {
        let content = b"puts 'hello'";
        assert_eq!(detect_hashbang(content), None);
    }

    #[test]
    fn effective_header_start_with_shebang() {
        let content = b"#!/bin/bash\necho hello";
        assert_eq!(effective_header_start(content), 12);
    }

    #[test]
    fn effective_header_start_with_encoding() {
        let content = b"# -*- coding: utf-8 -*-\nprint('hello')";
        assert_eq!(effective_header_start(content), 24);
    }

    #[test]
    fn effective_header_start_no_prefix() {
        let content = b"package main";
        assert_eq!(effective_header_start(content), 0);
    }

    #[test]
    fn effective_header_start_precedence_order() {
        // Shebang > XML > Hashbang
        let content = b"#!/bin/bash\n# -*- coding: utf-8 -*-\ncode";
        assert_eq!(effective_header_start(content), 12);
    }
}
