//! Edge case and error handling integration tests

mod common;

use common::{create_test_config, TestFixture};
use fast_license_checker::{checker::HeaderChecker, scanner::Scanner};
use std::fs;

#[test]
fn handles_very_long_files() {
    let fixture = TestFixture::new();

    // Create file with 10MB of content after header
    let mut content = "// MIT License\n\n// Copyright (c) 2024 Test\n\n".to_string();
    content.push_str(&"// Long comment\n".repeat(100_000));

    let file_path = fixture.create_file("large.rs", &content);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");

    let status = checker.check_file(&file_path).expect("Check failed");

    // Should detect header without reading entire file
    assert!(matches!(status, fast_license_checker::types::FileStatus::HasHeader));
}

#[test]
fn handles_unicode_in_headers() {
    let fixture = TestFixture::new();

    let content = "// MIT License\n// Copyright © 2024 Tëst Ûser 中文\nfn main() {}";
    let file_path = fixture.create_file("unicode.rs", content);

    let mut config = create_test_config();
    config.license_header = "MIT License\nCopyright © 2024 Tëst Ûser 中文".to_string();

    let checker = HeaderChecker::new(&config).expect("Failed to create checker");
    let status = checker.check_file(&file_path).expect("Check failed");

    assert!(matches!(status, fast_license_checker::types::FileStatus::HasHeader));
}

#[test]
fn handles_different_line_endings() {
    let fixture = TestFixture::new();

    // CRLF (Windows) line endings
    let crlf_content = "// MIT License\r\n\r\n// Copyright (c) 2024 Test\r\nfn main() {}";
    let crlf_path = fixture.create_file("crlf.rs", crlf_content);

    // LF (Unix) line endings
    let lf_content = "// MIT License\n\n// Copyright (c) 2024 Test\nfn main() {}";
    let lf_path = fixture.create_file("lf.rs", lf_content);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");

    // Both should be detected as having headers
    let crlf_status = checker.check_file(&crlf_path).expect("CRLF check failed");
    let lf_status = checker.check_file(&lf_path).expect("LF check failed");

    assert!(matches!(crlf_status, fast_license_checker::types::FileStatus::HasHeader));
    assert!(matches!(lf_status, fast_license_checker::types::FileStatus::HasHeader));
}

#[test]
fn handles_files_with_only_whitespace() {
    let fixture = TestFixture::new();

    let file_path = fixture.create_file("whitespace.rs", "   \n\n\t\n   ");

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    // Should skip or handle gracefully
    assert_eq!(summary.total, 0, "Whitespace-only files should be skipped");
}

#[test]
fn handles_symlinks() {
    let fixture = TestFixture::new();

    // Create original file
    let original = fixture.create_rust_file("original.rs", true);

    // Create symlink (skip on Windows if not supported)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = fixture.path().join("link.rs");
        symlink(&original, &link_path).expect("Failed to create symlink");

        let config = create_test_config();
        let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
        let summary = scanner.scan();

        // Should handle symlinks without infinite loops
        assert!(summary.total >= 1);
    }
}

#[test]
fn handles_files_with_bom() {
    let fixture = TestFixture::new();

    // UTF-8 BOM + content
    let bom_content = vec![0xEF, 0xBB, 0xBF, b'/', b'/', b' ', b'M', b'I', b'T'];
    let file_path = fixture.create_binary_file("bom.rs", &bom_content);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");

    // Should handle BOM gracefully
    let result = checker.check_file(&file_path);
    assert!(result.is_ok(), "Should handle BOM files");
}

#[test]
fn handles_mixed_comment_styles_in_same_file() {
    let fixture = TestFixture::new();

    let content = "// MIT License\n/* Copyright (c) 2024 Test */\nfn main() {}";
    let file_path = fixture.create_file("mixed.rs", content);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");
    let status = checker.check_file(&file_path).expect("Check failed");

    // Should handle mixed comment styles
    assert!(matches!(
        status,
        fast_license_checker::types::FileStatus::HasHeader
            | fast_license_checker::types::FileStatus::MalformedHeader { .. }
    ));
}

#[test]
fn handles_extremely_nested_directories() {
    let fixture = TestFixture::new();

    // Create deeply nested structure (50 levels)
    let mut path = "a".to_string();
    for _ in 0..50 {
        path.push_str("/b");
    }
    path.push_str("/file.rs");

    fixture.create_rust_file(&path, true);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 1);
}

#[test]
fn handles_special_characters_in_filenames() {
    let fixture = TestFixture::new();

    // Create files with special characters
    let special_names = [
        "file with spaces.rs",
        "file-with-dashes.rs",
        "file_with_underscores.rs",
        "file.multiple.dots.rs",
    ];

    for name in &special_names {
        fixture.create_rust_file(name, true);
    }

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, special_names.len());
}

#[test]
fn handles_files_with_no_extension() {
    let fixture = TestFixture::new();

    let content = "#!/bin/bash\n# MIT License\n\n# Copyright (c) 2024 Test\necho 'Hello'";
    let file_path = fixture.create_file("Makefile", content);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");

    // Should handle files without extensions
    let result = checker.check_file(&file_path);
    assert!(result.is_ok());
}

#[test]
fn handles_shebang_with_arguments() {
    let fixture = TestFixture::new();

    let content = "#!/usr/bin/env python3 -u\n# MIT License\n\n# Copyright (c) 2024 Test\nprint('test')";
    let file_path = fixture.create_file("script.py", content);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");
    let status = checker.check_file(&file_path).expect("Check failed");

    // Should detect header after shebang with arguments
    assert!(matches!(status, fast_license_checker::types::FileStatus::HasHeader));
}

#[test]
fn handles_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let fixture = TestFixture::new();

    // Create test files
    for i in 0..20 {
        fixture.create_rust_file(&format!("file_{}.rs", i), i % 2 == 0);
    }

    let config = Arc::new(create_test_config());
    let path = fixture.path().to_path_buf();

    // Spawn multiple threads doing scans
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let config = Arc::clone(&config);
            let path = path.clone();
            thread::spawn(move || {
                let scanner = Scanner::new(&path, (*config).clone()).expect("Failed to create scanner");
                scanner.scan()
            })
        })
        .collect();

    // All threads should complete successfully
    for handle in handles {
        let summary = handle.join().expect("Thread panicked");
        assert_eq!(summary.total, 20);
    }
}

#[test]
fn handles_invalid_utf8_gracefully() {
    let fixture = TestFixture::new();

    // Create file with invalid UTF-8
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, b'/', b'/', b' ', b't', b'e', b's', b't'];
    let file_path = fixture.create_binary_file("invalid.rs", &invalid_utf8);

    let config = create_test_config();
    let checker = HeaderChecker::new(&config).expect("Failed to create checker");

    // Should skip or handle invalid UTF-8 gracefully
    let result = checker.check_file(&file_path);

    if let Ok(status) = result {
        assert!(matches!(
            status,
            fast_license_checker::types::FileStatus::Skipped { .. }
        ));
    } else {
        // Error is also acceptable
        assert!(result.is_err());
    }
}
