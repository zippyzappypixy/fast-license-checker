//! End-to-end integration tests for fix mode

mod common;

use common::{create_test_config, TestFixture, MIT_LICENSE_HEADER};
use fast_license_checker::fixer::HeaderFixer;
use std::fs;

#[test]
fn fix_adds_header_to_file_without_header() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_rust_file("main.rs", false);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    let result = fixer.fix_file(&file_path).expect("Fix failed");

    assert!(matches!(result.action, fast_license_checker::types::FixAction::Fixed));

    // Verify header was added
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert!(content.contains("MIT License"));
    assert!(content.contains("Copyright (c) 2024 Test"));
}

#[test]
fn fix_preserves_file_with_header() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_rust_file("main.rs", true);

    let original_content = fs::read_to_string(&file_path).expect("Failed to read");

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    let result = fixer.fix_file(&file_path).expect("Fix failed");

    assert!(matches!(result.action, fast_license_checker::types::FixAction::AlreadyHasHeader));

    // Verify file unchanged
    let new_content = fs::read_to_string(&file_path).expect("Failed to read");
    assert_eq!(original_content, new_content);
}

#[test]
fn fix_is_idempotent() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_rust_file("main.rs", false);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    // First fix
    let result1 = fixer.fix_file(&file_path).expect("First fix failed");
    assert!(matches!(result1.action, fast_license_checker::types::FixAction::Fixed));

    let content_after_first = fs::read_to_string(&file_path).expect("Failed to read");

    // Second fix
    let result2 = fixer.fix_file(&file_path).expect("Second fix failed");
    assert!(matches!(result2.action, fast_license_checker::types::FixAction::AlreadyHasHeader));

    let content_after_second = fs::read_to_string(&file_path).expect("Failed to read");

    // Content should be identical - no duplicate headers
    assert_eq!(content_after_first, content_after_second);

    // Verify no duplicate headers
    let header_count = content_after_second.matches("MIT License").count();
    assert_eq!(header_count, 1, "Header should appear exactly once");
}

#[test]
fn fix_preserves_shebang() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_python_file_with_shebang("script.py", false);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    fixer.fix_file(&file_path).expect("Fix failed");

    let content = fs::read_to_string(&file_path).expect("Failed to read");

    // Shebang must be first line
    assert!(content.starts_with("#!/usr/bin/env python3\n"));

    // Header should come after shebang
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "#!/usr/bin/env python3");
    assert!(lines[1].contains("MIT License") || lines[2].contains("MIT License"));
}

#[test]
fn fix_preserves_xml_declaration() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_xml_file("config.xml", false);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    fixer.fix_file(&file_path).expect("Fix failed");

    let content = fs::read_to_string(&file_path).expect("Failed to read");

    // XML declaration must be first
    assert!(content.starts_with("<?xml version"));

    // Header should come after declaration
    assert!(content.contains("<!-- MIT License -->"));
}

#[test]
fn fix_skips_binary_files() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_binary_file("image.png", &[0x89, 0x50, 0x4E, 0x47]);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    let result = fixer.fix_file(&file_path).expect("Fix operation failed");

    assert!(matches!(
        result.action,
        fast_license_checker::types::FixAction::Skipped { .. }
    ));
}

#[test]
fn fix_uses_atomic_writes() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_rust_file("main.rs", false);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    // Fix should complete successfully
    fixer.fix_file(&file_path).expect("Fix failed");

    // No .tmp files should remain
    let parent = file_path.parent().expect("No parent directory");
    let entries = fs::read_dir(parent).expect("Failed to read directory");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let filename = entry.file_name();
        assert!(
            !filename.to_string_lossy().ends_with(".tmp"),
            "Temporary file left behind: {:?}",
            filename
        );
    }
}

#[test]
fn fix_multiple_files_in_directory() {
    let fixture = TestFixture::new();

    // Create multiple files without headers
    fixture.create_rust_file("src/main.rs", false);
    fixture.create_rust_file("src/lib.rs", false);
    fixture.create_rust_file("src/helper.rs", false);

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    // Fix all files
    let summary = fixer.fix_all(fixture.path()).expect("Fix all failed");

    assert_eq!(summary.fixed, 3);

    // Verify all files have headers
    let files = ["src/main.rs", "src/lib.rs", "src/helper.rs"];
    for file in &files {
        let content = fs::read_to_string(fixture.path().join(file)).expect("Failed to read");
        assert!(content.contains("MIT License"));
    }
}

#[test]
fn fix_handles_readonly_files() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_rust_file("readonly.rs", false);

    // Make file read-only
    let mut perms = fs::metadata(&file_path).expect("Failed to get metadata").permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file_path, perms).expect("Failed to set readonly");

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    let result = fixer.fix_file(&file_path);

    // Should return error (cannot write to readonly file)
    assert!(result.is_err());
}

#[test]
fn fix_refuses_to_modify_malformed_headers() {
    let fixture = TestFixture::new();

    // Create file with malformed header
    let malformed_content = "// Some License Text\n// Copyright Wrong\nfn main() {}";
    let file_path = fixture.create_file("malformed.rs", malformed_content);

    let original_content = fs::read_to_string(&file_path).expect("Failed to read");

    let config = create_test_config();
    let fixer = HeaderFixer::new(&config).expect("Failed to create fixer");

    let result = fixer.fix_file(&file_path);

    // Should either error or skip malformed headers (don't silently overwrite)
    if let Ok(fix_result) = result {
        assert!(!matches!(fix_result.action, fast_license_checker::types::FixAction::Fixed));
    }

    // Content should be unchanged
    let new_content = fs::read_to_string(&file_path).expect("Failed to read");
    assert_eq!(original_content, new_content);
}
