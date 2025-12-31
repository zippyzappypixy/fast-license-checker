# Fast License Checker (flc)

Blazing-fast license header verification for your codebase.

[![CI](https://github.com/zippyzappypixy/fast-license-checker/actions/workflows/ci.yml/badge.svg)](https://github.com/zippyzappypixy/fast-license-checker/actions/workflows/ci.yml)
![Benchmark](https://img.shields.io/badge/100K%20files-407ms-brightgreen)

## Features

- **Fast**: Scans 100,000 files in under 1 second
- **Git-aware**: Automatically respects .gitignore
- **Safe**: Never corrupts binary files
- **Smart**: Handles shebangs, XML declarations, and various comment styles
- **CI-ready**: JSON and GitHub Actions output formats
- **Parallel**: Uses all available CPU cores for maximum performance

## Installation

### From source
```bash
cargo install fast-license-checker
```

### Pre-built binaries
Download from [Releases](https://github.com/zippyzappypixy/fast-license-checker/releases)

## Usage

### Scan mode (check for missing headers)
```bash
flc .                           # Scan current directory
flc src/                        # Scan specific directory
flc --header "MIT License" .    # Specify header text
flc --license LICENSE.txt .     # Specify header from file
flc --output json .             # JSON output for CI
flc --output github .           # GitHub Actions annotations
```

### Fix mode (add missing headers)
```bash
flc --fix .                     # Add headers to files missing them
flc --fix --dry-run .           # Preview changes without applying
```

### Advanced usage
```bash
# Custom comment styles
flc --comment-style "rs=//" --comment-style "py=#" .

# Ignore additional patterns
flc --ignore "target/" --ignore "node_modules/" .

# Control parallelism
flc --jobs 4 .                  # Use 4 CPU cores
flc --jobs 1 .                  # Single-threaded for debugging

# Fuzzy matching for license variations
flc --similarity 85 .           # Allow 85% similarity threshold
```

## Configuration

Create `.license-checker.toml` in your project root:

```toml
license_header = """
Copyright (c) 2024 Your Name
Licensed under the MIT License
"""

[comment_styles]
rs = { prefix = "//" }
py = { prefix = "#" }
js = { prefix = "//" }
html = { prefix = "<!--", suffix = "-->" }
xml = { prefix = "<!--", suffix = "-->" }
yaml = { prefix = "#" }
toml = { prefix = "#" }
md = { prefix = "<!--", suffix = "-->" }

ignore_patterns = ["vendor/", "generated/", "target/", "node_modules/"]
max_header_bytes = 8192
parallel_jobs = 0  # 0 = auto-detect CPU cores
similarity_threshold = 90  # 0-100, higher = stricter matching
```

## Exit Codes

- `0`: All files have valid headers (scan) or fixes applied successfully (fix)
- `1`: Missing headers found (scan) or errors encountered (fix)
- `2`: Configuration error or invalid arguments

## Performance Benchmarks

Fast License Checker is designed for speed. On a modern laptop:

| Workload | Time | Notes |
|----------|------|-------|
| **1K files** | 6ms | Baseline overhead |
| **10K files** | 51ms | Typical small project |
| **100K files** | 424ms | **Large codebase** ✅ |

### Test Environment
- **CPU**: 11th Gen Intel Core i5-1135G7 @ 2.40GHz (4 cores, 8 threads)
- **RAM**: 7.5GB DDR4
- **Disk**: 147GB NVMe SSD
- **OS**: Ubuntu 24.04 LTS
- **Rust**: 1.92.0 (stable)

### Run Your Own Benchmarks
```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks
cargo bench -- scan

# View HTML report
open docs/benchmarks/index.html
```

> **Note**: First run is slower due to disk cache warming.
> Results shown are from second run (warm cache).

## Supported File Types

### Automatic Detection
- **Rust** (.rs): `//` comments
- **Python** (.py): `#` comments
- **JavaScript/TypeScript** (.js, .ts, .jsx, .tsx): `//` comments
- **Go** (.go): `//` comments
- **Java** (.java): `//` or `/* */` comments
- **C/C++** (.c, .cpp, .h, .hpp): `//` or `/* */` comments
- **HTML/XML** (.html, .xml): `<!-- -->` comments
- **YAML** (.yml, .yaml): `#` comments
- **TOML** (.toml): `#` comments
- **Markdown** (.md): `<!-- -->` comments
- **Shell scripts** (.sh, .bash): `#` comments

### Custom Comment Styles
For unsupported file types, define custom comment styles:

```toml
[comment_styles]
custom = { prefix = "/*", suffix = "*/" }  # Block comments
lua = { prefix = "--" }                    # Lua line comments
```

## CI Integration

### GitHub Actions
```yaml
- name: Check license headers
  uses: zippyzappypixy/fast-license-checker@v0.1.0
  with:
    path: '.'
    header: 'Copyright (c) 2024 My Company'
    fix: false  # Set to true to auto-fix

- name: Annotate PR
  run: flc --output github .
```

### GitLab CI
```yaml
license-check:
  script:
    - curl -L https://github.com/zippyzappypixy/fast-license-checker/releases/latest/download/flc-linux-x86_64 -o flc
    - chmod +x flc
    - ./flc --output json . > license-report.json
  artifacts:
    reports:
      license_scanning: license-report.json
```

### Jenkins
```groovy
pipeline {
    agent any
    steps {
        sh '''
            curl -L https://github.com/zippyzappypixy/fast-license-checker/releases/latest/download/flc-linux-x86_64 -o flc
            chmod +x flc
            ./flc --output json . > license-report.json
        '''
        publishCoverage adapters: [jacocoAdapter(path: 'license-report.json')]
    }
}
```

## Development

### Prerequisites
- Rust 1.70+
- Cargo

### Building
```bash
git clone https://github.com/zippyzappypixy/fast-license-checker
cd fast-license-checker
cargo build --release
```

### Testing
```bash
cargo test --workspace          # Run all tests
cargo bench                     # Run benchmarks
cargo test --doc                # Test documentation examples
```

### Quality Gates
```bash
cargo fmt --all -- --check      # Check formatting
cargo clippy -- -D warnings     # Strict linting
cargo deny check                # Security audit
cargo llvm-cov --workspace      # Code coverage
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes with tests
4. Run quality gates: `cargo fmt && cargo clippy && cargo test`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

## License

Licensed under MIT OR Apache-2.0 (choose one that fits your project).

This tool is dual-licensed to maximize compatibility with your codebase.
