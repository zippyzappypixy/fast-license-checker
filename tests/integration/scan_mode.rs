//! End-to-end integration tests for scan mode

mod common;

use common::{create_test_config, TestFixture, MIT_LICENSE_HEADER};
use fast_license_checker::{scanner::Scanner, types::FileStatus};

#[test]
fn scan_empty_directory() {
    let fixture = TestFixture::new();
    let config = create_test_config();

    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 0);
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
}

#[test]
fn scan_single_file_with_header() {
    let fixture = TestFixture::new();
    fixture.create_rust_file("main.rs", true);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 1);
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 0);
}

#[test]
fn scan_single_file_missing_header() {
    let fixture = TestFixture::new();
    fixture.create_rust_file("main.rs", false);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 1);
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 1);
}

#[test]
fn scan_multiple_files_mixed_status() {
    let fixture = TestFixture::new();

    // Create files with various states
    fixture.create_rust_file("src/main.rs", true);
    fixture.create_rust_file("src/lib.rs", true);
    fixture.create_rust_file("src/helper.rs", false);
    fixture.create_rust_file("tests/test.rs", false);
    fixture.create_rust_file("benches/bench.rs", true);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 5);
    assert_eq!(summary.passed, 3);
    assert_eq!(summary.failed, 2);
}

#[test]
fn scan_respects_gitignore() {
    let fixture = TestFixture::new();

    // Create .gitignore
    fixture.create_gitignore(&["target/", "*.log", "ignored.rs"]);

    // Create files (some should be ignored)
    fixture.create_rust_file("src/main.rs", true);
    fixture.create_rust_file("target/debug/build.rs", false);
    fixture.create_rust_file("ignored.rs", false);
    fixture.create_file("debug.log", "some log content");

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    // Only src/main.rs should be scanned
    assert_eq!(summary.total, 1);
    assert_eq!(summary.passed, 1);
}

#[test]
fn scan_skips_binary_files() {
    let fixture = TestFixture::new();

    // Create text and binary files
    fixture.create_rust_file("main.rs", true);
    fixture.create_binary_file("image.png", &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    fixture.create_binary_file("executable", &[0x7F, 0x45, 0x4C, 0x46]);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    // Only text file should be scanned
    assert_eq!(summary.total, 1);
    assert!(summary.skipped >= 2); // Binary files should be skipped
}

#[test]
fn scan_handles_empty_files() {
    let fixture = TestFixture::new();

    fixture.create_rust_file("main.rs", true);
    fixture.create_file("empty.rs", "");

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    // Empty file should be skipped if skip_empty_files is true
    assert!(summary.skipped >= 1);
}

#[test]
fn scan_parallel_processing() {
    let fixture = TestFixture::new();

    // Create many files to test parallel scanning
    for i in 0..100 {
        let filename = format!("file_{}.rs", i);
        fixture.create_rust_file(&filename, i % 2 == 0); // Alternate headers
    }

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 100);
    assert_eq!(summary.passed, 50);
    assert_eq!(summary.failed, 50);
}

#[test]
fn scan_nested_directories() {
    let fixture = TestFixture::new();

    fixture.create_rust_file("main.rs", true);
    fixture.create_rust_file("src/lib.rs", true);
    fixture.create_rust_file("src/utils/helper.rs", false);
    fixture.create_rust_file("tests/integration/test.rs", false);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    assert_eq!(summary.total, 4);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 2);
}

#[test]
fn scan_with_malformed_headers() {
    let fixture = TestFixture::new();

    // Create file with partial/malformed header
    let malformed_content = "// MIT License\n// Wrong copyright line\nfn main() {}";
    fixture.create_file("malformed.rs", malformed_content);

    let config = create_test_config();
    let scanner = Scanner::new(fixture.path(), config).expect("Failed to create scanner");
    let summary = scanner.scan();

    // Should detect as malformed or missing depending on similarity
    assert!(summary.failed > 0 || summary.malformed > 0);
}
