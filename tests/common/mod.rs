//! Common test utilities and fixtures for integration tests

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture for creating temporary test directories with files
pub struct TestFixture {
    temp_dir: TempDir,
}

impl TestFixture {
    /// Create a new test fixture with a temporary directory
    pub fn new() -> Self {
        Self { temp_dir: TempDir::new().expect("Failed to create temp directory") }
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a file with the given content
    pub fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.temp_dir.path().join(relative_path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    /// Create a binary file with the given content
    pub fn create_binary_file(&self, relative_path: &str, content: &[u8]) -> PathBuf {
        let file_path = self.temp_dir.path().join(relative_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        fs::write(&file_path, content).expect("Failed to write binary file");
        file_path
    }

    /// Create a .gitignore file with the given patterns
    pub fn create_gitignore(&self, patterns: &[&str]) -> PathBuf {
        let content = patterns.join("\n");
        self.create_file(".gitignore", &content)
    }

    /// Create a Rust source file with or without a license header
    pub fn create_rust_file(&self, name: &str, has_header: bool) -> PathBuf {
        let content = if has_header {
            "// MIT License\n\n// Copyright (c) 2024 Test\n\nfn main() {\n    println!(\"Hello, world!\");\n}\n"
        } else {
            "fn main() {\n    println!(\"Hello, world!\");\n}\n"
        };
        self.create_file(name, content)
    }

    /// Create a Python file with shebang and optional header
    pub fn create_python_file_with_shebang(&self, name: &str, has_header: bool) -> PathBuf {
        let content = if has_header {
            "#!/usr/bin/env python3\n# MIT License\n\n# Copyright (c) 2024 Test\n\nprint('Hello, world!')\n"
        } else {
            "#!/usr/bin/env python3\n\nprint('Hello, world!')\n"
        };
        self.create_file(name, content)
    }

    /// Create an XML file with declaration and optional header
    pub fn create_xml_file(&self, name: &str, has_header: bool) -> PathBuf {
        let content = if has_header {
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- MIT License -->\n\n<!-- Copyright (c) 2024 Test -->\n<root></root>\n"
        } else {
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root></root>\n"
        };
        self.create_file(name, content)
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard MIT license header for tests
pub const MIT_LICENSE_HEADER: &str = "MIT License\n\nCopyright (c) 2024 Test";

/// Create a test configuration with the MIT license header
pub fn create_test_config() -> fast_license_checker::config::Config {
    let mut config = fast_license_checker::config::Config::default();
    config.license_header = MIT_LICENSE_HEADER.to_string();
    config.similarity_threshold = 70;

    // Add common comment styles
    use fast_license_checker::config::CommentStyleConfig;

    config
        .comment_styles
        .insert("rs".to_string(), CommentStyleConfig { prefix: "//".to_string(), suffix: None });

    config
        .comment_styles
        .insert("py".to_string(), CommentStyleConfig { prefix: "#".to_string(), suffix: None });

    config.comment_styles.insert(
        "xml".to_string(),
        CommentStyleConfig { prefix: "<!--".to_string(), suffix: Some("-->".to_string()) },
    );

    config
}
