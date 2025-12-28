//! Performance benchmarks for the fast-license-checker.
//!
//! Run with: `cargo bench`
//!
//! Target: 100,000 files scanned in under 1 second (warm cache)
//!
//! ## Benchmark Scenarios
//!
//! - `scan_1k_files` - Small project baseline
//! - `scan_10k_files` - Medium project
//! - `scan_100k_files` - Large monorepo (target: < 1 second)
//! - `scan_100k_warm_cache` - With filesystem cache warmed
//!
//! ## File Content Distribution
//!
//! Files are created with realistic sizes (500 bytes - 5KB) and include:
//! - 50% with correct license headers
//! - 40% missing headers
//! - 5% with shebangs (Python/Shell scripts)
//! - 5% binary files (should be skipped)

// Benchmark code is allowed to use patterns forbidden in production
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::disallowed_methods)]
#![allow(clippy::manual_contains)]
#![allow(missing_docs)] // criterion macros generate undocumented functions

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::io::Read;
use std::path::Path;
use tempfile::TempDir;

/// Default license header for Rust-style comments
const RUST_HEADER: &[u8] = b"// Copyright 2024 Example Corp\n// Licensed under MIT License\n\n";

/// Default license header for Python-style comments
const PYTHON_HEADER: &[u8] = b"# Copyright 2024 Example Corp\n# Licensed under MIT License\n\n";

/// Default license header for HTML/XML-style comments
const HTML_HEADER: &[u8] =
    b"<!-- Copyright 2024 Example Corp -->\n<!-- Licensed under MIT License -->\n\n";

/// Represents different types of test files
#[derive(Debug, Clone, Copy)]
enum FileType {
    /// File with correct license header
    WithHeader,
    /// File missing license header
    WithoutHeader,
    /// Script file with shebang (header goes after shebang)
    Shebang,
    /// Binary file (should be skipped)
    Binary,
}

/// Creates test files with realistic content distribution.
///
/// # File Distribution
/// - 50% have correct headers
/// - 40% are missing headers
/// - 5% have shebangsss
/// - 5% are binary
///
/// # File Sizes
/// Files range from 500 to 5000 bytes to simulate real source files.
fn create_test_files(dir: &TempDir, count: usize) {
    for i in 0..count {
        // Create subdirectories (100 files per directory)
        let subdir_index = i / 100;
        let subdir = dir.path().join(format!("dir_{:04}", subdir_index));
        if i % 100 == 0 {
            fs::create_dir_all(&subdir).ok();
        }

        // Determine file type based on distribution
        let file_type = match i % 20 {
            0..=9 => FileType::WithHeader,      // 50%
            10..=17 => FileType::WithoutHeader, // 40%
            18 => FileType::Shebang,            // 5%
            _ => FileType::Binary,              // 5%
        };

        // Determine extension based on file type and variety
        let ext = match file_type {
            FileType::Binary => "bin",
            FileType::Shebang => {
                if i % 2 == 0 {
                    "py"
                } else {
                    "sh"
                }
            }
            _ => match i % 8 {
                0 | 1 => "rs",
                2 | 3 => "py",
                4 | 5 => "js",
                6 => "go",
                _ => "html",
            },
        };

        // Calculate file size: 500 to 5000 bytes
        // Use deterministic "random" based on index for reproducibility
        let base_size = 500;
        let variable_size = ((i * 7919) % 4500) + base_size; // Prime for better distribution
        let file_size = variable_size.min(5000);

        // Generate content
        let content = generate_file_content(file_type, ext, file_size, i);

        let path = subdir.join(format!("file_{:06}.{}", i, ext));
        fs::write(path, content).ok();
    }
}

/// Generates file content based on type and extension.
fn generate_file_content(file_type: FileType, ext: &str, size: usize, index: usize) -> Vec<u8> {
    match file_type {
        FileType::WithHeader => generate_file_with_header(ext, size, index),
        FileType::WithoutHeader => generate_file_without_header(ext, size, index),
        FileType::Shebang => generate_shebang_file(ext, size, index),
        FileType::Binary => generate_binary_content(size),
    }
}

/// Generates a source file with a proper license header.
fn generate_file_with_header(ext: &str, size: usize, index: usize) -> Vec<u8> {
    let header: &[u8] = match ext {
        "rs" | "js" | "go" | "c" | "cpp" | "java" => RUST_HEADER,
        "py" | "sh" | "rb" | "yaml" | "yml" => PYTHON_HEADER,
        "html" | "xml" | "svg" => HTML_HEADER,
        _ => RUST_HEADER,
    };

    let mut content = header.to_vec();
    append_filler_code(&mut content, ext, size, index);
    content
}

/// Generates a source file without a license header.
fn generate_file_without_header(ext: &str, size: usize, index: usize) -> Vec<u8> {
    let mut content = Vec::with_capacity(size);
    append_filler_code(&mut content, ext, size, index);
    content
}

/// Generates a script file with shebang, header should go after shebang.
fn generate_shebang_file(ext: &str, size: usize, index: usize) -> Vec<u8> {
    let shebang: &[u8] = match ext {
        "py" => b"#!/usr/bin/env python3\n",
        "sh" => b"#!/bin/bash\n",
        _ => b"#!/usr/bin/env node\n",
    };

    let mut content = shebang.to_vec();
    // 50% of shebang files have headers (after shebang)
    if index % 2 == 0 {
        content.extend_from_slice(PYTHON_HEADER);
    }
    append_filler_code(&mut content, ext, size, index);
    content
}

/// Generates binary content that should trigger binary detection.
fn generate_binary_content(size: usize) -> Vec<u8> {
    // Start with PNG-like magic bytes including NULL byte
    let mut content = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];

    // Fill with pseudo-random binary data
    while content.len() < size {
        content.push(((content.len() * 251) % 256) as u8);
    }
    content.truncate(size);
    content
}

/// Appends realistic-looking code to reach the target size.
fn append_filler_code(content: &mut Vec<u8>, ext: &str, target_size: usize, index: usize) {
    let code_templates: &[&[u8]] = match ext {
        "rs" => &[
            b"pub fn example_function() {\n    let value = 42;\n    println!(\"Value: {}\", value);\n}\n\n",
            b"#[derive(Debug, Clone)]\npub struct Example {\n    pub field: String,\n    pub count: usize,\n}\n\n",
            b"impl Example {\n    pub fn new() -> Self {\n        Self { field: String::new(), count: 0 }\n    }\n}\n\n",
        ],
        "py" => &[
            b"def example_function():\n    value = 42\n    print(f\"Value: {value}\")\n\n",
            b"class Example:\n    def __init__(self):\n        self.field = \"\"\n        self.count = 0\n\n",
            b"    def process(self, data):\n        return [x * 2 for x in data]\n\n",
        ],
        "js" => &[
            b"function exampleFunction() {\n    const value = 42;\n    console.log(`Value: ${value}`);\n}\n\n",
            b"class Example {\n    constructor() {\n        this.field = '';\n        this.count = 0;\n    }\n}\n\n",
            b"const processData = (data) => data.map(x => x * 2);\n\n",
        ],
        _ => &[
            b"// This is filler content for benchmarking purposes.\n",
            b"// The actual content doesn't matter for performance testing.\n",
            b"// We just need realistic file sizes.\n\n",
        ],
    };

    let mut template_index = index;
    while content.len() < target_size {
        let template = code_templates[template_index % code_templates.len()];
        content.extend_from_slice(template);
        template_index = template_index.wrapping_add(1);
    }
    content.truncate(target_size);
}

/// Warms the filesystem cache by reading all files once.
///
/// This simulates a "warm cache" scenario where the OS has recently
/// accessed the files and they're in the page cache.
fn warm_cache(dir: &Path) {
    fn walk_and_read(path: &Path) {
        if path.is_file() {
            let mut file = match fs::File::open(path) {
                Ok(f) => f,
                Err(_) => return,
            };
            let mut buffer = [0u8; 8192];
            while file.read(&mut buffer).unwrap_or(0) > 0 {}
        } else if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    walk_and_read(&entry.path());
                }
            }
        }
    }
    walk_and_read(dir);
}

/// Benchmark group for scanning different project sizes.
fn bench_scan_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    group.sample_size(10); // Fewer samples for large file counts

    for count in [1_000, 10_000, 100_000] {
        let dir = TempDir::new().expect("Failed to create temp directory");
        create_test_files(&dir, count);

        // Warm the cache before benchmarking
        warm_cache(dir.path());

        group.bench_with_input(BenchmarkId::new("files_warm", count), &count, |b, _| {
            b.iter(|| {
                // TODO: Replace with actual scanner once implemented
                // let config = fast_license_checker::Config::default();
                // let scanner = fast_license_checker::Scanner::new(dir.path(), config).unwrap();
                // scanner.scan()

                // Placeholder: Walk and read files to simulate scanning
                let file_count = std::cell::Cell::new(0usize);
                walk_files(dir.path(), &mut |path| {
                    let mut buffer = vec![0u8; 8192];
                    if let Ok(mut file) = fs::File::open(path) {
                        let _ = file.read(&mut buffer);
                    }
                    file_count.set(file_count.get().wrapping_add(1));
                });
                black_box(file_count.get())
            });
        });
    }

    group.finish();
}

/// Benchmark for checking a single file (micro-benchmark).
fn bench_single_file_check(c: &mut Criterion) {
    // Create a realistic file content
    let content = {
        let mut v = RUST_HEADER.to_vec();
        v.extend_from_slice(b"pub fn main() {\n    println!(\"Hello, world!\");\n}\n");
        while v.len() < 2000 {
            v.extend_from_slice(b"// Additional comment line for padding.\n");
        }
        v
    };

    c.bench_function("check_single_file", |b| {
        b.iter(|| {
            // TODO: Replace with actual header check once implemented
            // let header = fast_license_checker::LicenseHeader::new("Copyright 2024").unwrap();
            // fast_license_checker::detect_header(&content, &header, &style)

            // Placeholder: Check if content starts with header bytes
            let has_header = content.starts_with(b"// Copyright");
            black_box(has_header)
        });
    });
}

/// Benchmark for binary file detection.
fn bench_binary_detection(c: &mut Criterion) {
    let binary_content = generate_binary_content(4096);
    let text_content = generate_file_with_header("rs", 4096, 0);

    let mut group = c.benchmark_group("binary_detection");

    group.bench_function("detect_binary", |b| {
        b.iter(|| {
            // TODO: Replace with actual binary detection
            // fast_license_checker::is_binary(&binary_content)

            // Placeholder: Check for NULL byte
            let is_binary = binary_content.iter().any(|&b| b == 0);
            black_box(is_binary)
        });
    });

    group.bench_function("detect_text", |b| {
        b.iter(|| {
            // Placeholder: Check for NULL byte
            let is_binary = text_content.iter().any(|&b| b == 0);
            black_box(is_binary)
        });
    });

    group.finish();
}

/// Benchmark for shebang detection and offset calculation.
fn bench_shebang_detection(c: &mut Criterion) {
    let with_shebang = b"#!/usr/bin/env python3\n# Copyright 2024\nprint('hello')";
    let without_shebang = b"# Copyright 2024\nprint('hello')";

    let mut group = c.benchmark_group("shebang_detection");

    group.bench_function("with_shebang", |b| {
        b.iter(|| {
            // TODO: Replace with actual shebang detection
            // fast_license_checker::find_insertion_point(&with_shebang[..])

            // Placeholder: Find newline after #!
            let offset = if with_shebang.starts_with(b"#!") {
                with_shebang.iter().position(|&b| b == b'\n').unwrap_or(0) + 1
            } else {
                0
            };
            black_box(offset)
        });
    });

    group.bench_function("without_shebang", |b| {
        b.iter(|| {
            let offset = if without_shebang.starts_with(b"#!") {
                without_shebang.iter().position(|&b| b == b'\n').unwrap_or(0) + 1
            } else {
                0
            };
            black_box(offset)
        });
    });

    group.finish();
}

/// Helper function to walk files in a directory.
fn walk_files<F>(path: &Path, callback: &mut F)
where
    F: FnMut(&Path),
{
    if path.is_file() {
        callback(path);
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                walk_files(&entry.path(), callback);
            }
        }
    }
}

criterion_group!(
    benches,
    bench_scan_sizes,
    bench_single_file_check,
    bench_binary_detection,
    bench_shebang_detection,
);
criterion_main!(benches);
