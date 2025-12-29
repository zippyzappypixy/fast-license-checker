//! Performance benchmarks for scanning operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fast_license_checker::{config::Config, scanner::Scanner};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create realistic test files with proper size distribution
fn create_realistic_test_files(dir: &Path, count: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let filename = format!("file_{}.rs", i);
        let filepath = dir.join(&filename);

        // Realistic file size: 500-5000 bytes
        let size = rng.gen_range(500..5000);

        // Distribution:
        // 50% with correct headers
        // 40% missing headers
        // 5% with shebangs
        // 5% binary files
        let rand_val = rng.gen_range(0..100);

        let content = if rand_val < 50 {
            // File with header
            let mut c = "// MIT License\n\n// Copyright (c) 2024 Test\n\n".to_string();
            c.push_str(&"fn test() { /* code */ }\n".repeat(size / 30));
            c
        } else if rand_val < 90 {
            // File without header
            "fn test() { /* code */ }\n".repeat(size / 30)
        } else if rand_val < 95 {
            // File with shebang
            let mut c = "#!/usr/bin/env rust-script\n// MIT License\n\n".to_string();
            c.push_str(&"fn test() {}\n".repeat(size / 30));
            c
        } else {
            // Binary file simulation (with null bytes)
            let mut c = vec![0u8; size];
            rng.fill(&mut c[..]);
            fs::write(&filepath, c).expect("Failed to write binary");
            continue;
        };

        fs::write(&filepath, content).expect("Failed to write file");
    }
}

/// Warm up filesystem cache by reading all files
fn warm_cache(dir: &Path) {
    for entry in fs::read_dir(dir).expect("Failed to read dir") {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                let _ = fs::read(&entry.path());
            }
        }
    }
}

fn create_test_config() -> Config {
    let mut config = Config::default();
    config.license_header = "MIT License\n\nCopyright (c) 2024 Test".to_string();
    config
}

fn bench_scan_small(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_realistic_test_files(temp_dir.path(), 1_000);
    warm_cache(temp_dir.path());

    let config = create_test_config();

    c.bench_function("scan_1k_files_warm", |b| {
        b.iter(|| {
            let scanner =
                Scanner::new(temp_dir.path(), config.clone()).expect("Failed to create scanner");
            black_box(scanner.scan())
        });
    });
}

fn bench_scan_medium(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_realistic_test_files(temp_dir.path(), 10_000);
    warm_cache(temp_dir.path());

    let config = create_test_config();

    c.bench_function("scan_10k_files_warm", |b| {
        b.iter(|| {
            let scanner =
                Scanner::new(temp_dir.path(), config.clone()).expect("Failed to create scanner");
            black_box(scanner.scan())
        });
    });
}

fn bench_scan_large(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_realistic_test_files(temp_dir.path(), 100_000);
    warm_cache(temp_dir.path());

    let config = create_test_config();

    c.bench_function("scan_100k_files_warm", |b| {
        b.iter(|| {
            let scanner =
                Scanner::new(temp_dir.path(), config.clone()).expect("Failed to create scanner");
            black_box(scanner.scan())
        });
    });
}

fn bench_parallel_jobs(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_jobs");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_realistic_test_files(temp_dir.path(), 10_000);
    warm_cache(temp_dir.path());

    for num_jobs in &[1, 2, 4, 8] {
        group.bench_with_input(BenchmarkId::from_parameter(num_jobs), num_jobs, |b, &num_jobs| {
            b.iter(|| {
                let mut config = create_test_config();
                config.parallel_jobs = Some(num_jobs);
                let scanner =
                    Scanner::new(temp_dir.path(), config).expect("Failed to create scanner");
                black_box(scanner.scan())
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_scan_small,
    bench_scan_medium,
    bench_scan_large,
    bench_parallel_jobs
);
criterion_main!(benches);
