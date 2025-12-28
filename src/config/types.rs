//! Configuration types for the fast-license-checker.
//!
//! Defines the main configuration structure with default values
//! and support for various file types and comment styles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for the license checker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The license header text to check for
    pub license_header: String,

    /// Comment styles per file extension
    pub comment_styles: HashMap<String, CommentStyleConfig>,

    /// Additional glob patterns to ignore (beyond .gitignore)
    pub ignore_patterns: Vec<String>,

    /// Maximum bytes to read from file start for header check
    pub max_header_bytes: usize,

    /// Skip empty files (0 bytes)
    pub skip_empty_files: bool,

    /// Number of parallel jobs (None = num_cpus)
    pub parallel_jobs: Option<usize>,

    /// Similarity threshold for malformed header detection (0-100)
    pub similarity_threshold: u8,
}

/// Comment style configuration for different file types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentStyleConfig {
    /// The comment prefix (e.g., "//", "#", "/*")
    pub prefix: String,
    /// Optional comment suffix (e.g., "*/" for block comments)
    #[serde(default)]
    pub suffix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            license_header: String::new(),
            comment_styles: default_comment_styles(),
            ignore_patterns: vec![],
            max_header_bytes: 8192,
            skip_empty_files: true,
            parallel_jobs: None,
            similarity_threshold: 70,
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the license header text
    pub fn with_license_header(mut self, header: impl Into<String>) -> Self {
        self.license_header = header.into();
        self
    }

    /// Add a custom comment style for a file extension
    pub fn with_comment_style(
        mut self,
        extension: impl Into<String>,
        style: CommentStyleConfig,
    ) -> Self {
        self.comment_styles.insert(extension.into(), style);
        self
    }

    /// Add an ignore pattern
    pub fn with_ignore_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.ignore_patterns.push(pattern.into());
        self
    }

    /// Set maximum header bytes
    pub fn with_max_header_bytes(mut self, bytes: usize) -> Self {
        self.max_header_bytes = bytes;
        self
    }

    /// Set similarity threshold
    pub fn with_similarity_threshold(mut self, threshold: u8) -> Self {
        self.similarity_threshold = threshold.min(100);
        self
    }

    /// Get the comment style for a file extension
    pub fn get_comment_style(&self, extension: &str) -> Option<&CommentStyleConfig> {
        self.comment_styles.get(extension)
    }

    /// Check if a file extension has a configured comment style
    pub fn has_comment_style(&self, extension: &str) -> bool {
        self.comment_styles.contains_key(extension)
    }
}

/// Create default comment styles for common file extensions
fn default_comment_styles() -> HashMap<String, CommentStyleConfig> {
    let mut styles = HashMap::new();

    // Line comments with "//"
    let slash_slash_extensions = vec![
        "rs", "js", "ts", "jsx", "tsx", "c", "cpp", "cc", "cxx", "h", "hpp", "hxx", "java", "kt",
        "scala", "go", "swift", "cs", "vb", "fs", "ml", "fsx", "elm",
    ];
    for ext in slash_slash_extensions {
        styles
            .insert(ext.to_string(), CommentStyleConfig { prefix: "//".to_string(), suffix: None });
    }

    // Line comments with "#"
    let hash_extensions = vec![
        "py", "rb", "sh", "bash", "zsh", "fish", "pl", "pm", "tcl", "lua", "r", "yaml", "yml",
        "toml", "ini", "cfg", "conf", "ex", "exs", "clj", "cljs", "coffee", "dart", "nim",
        "nimble", "cr", "rspec", "thor",
    ];
    for ext in hash_extensions {
        styles
            .insert(ext.to_string(), CommentStyleConfig { prefix: "#".to_string(), suffix: None });
    }

    // Block comments with "<!--" and "-->"
    let html_extensions = vec!["html", "htm", "xml", "svg", "vue", "jsx", "tsx", "xsd"];
    for ext in html_extensions {
        styles.insert(
            ext.to_string(),
            CommentStyleConfig { prefix: "<!--".to_string(), suffix: Some("-->".to_string()) },
        );
    }

    // Block comments with "/*" and "*/"
    let css_extensions = vec!["css", "scss", "sass", "less", "styl"];
    for ext in css_extensions {
        styles.insert(
            ext.to_string(),
            CommentStyleConfig { prefix: "/*".to_string(), suffix: Some("*/".to_string()) },
        );
    }

    // Line comments with "--"
    let sql_extensions = vec!["sql", "hs", "lhs"];
    for ext in sql_extensions {
        styles
            .insert(ext.to_string(), CommentStyleConfig { prefix: "--".to_string(), suffix: None });
    }

    // Line comments with "%"
    let erlang_extensions = vec!["erl", "hrl"];
    for ext in erlang_extensions {
        styles
            .insert(ext.to_string(), CommentStyleConfig { prefix: "%".to_string(), suffix: None });
    }

    // Line comments with ";;"
    let lisp_extensions = vec!["lisp", "lsp", "scm", "ss", "rkt"];
    for ext in lisp_extensions {
        styles
            .insert(ext.to_string(), CommentStyleConfig { prefix: ";;".to_string(), suffix: None });
    }

    // Line comments with "\""
    let vim_extensions = vec!["vim", "vimrc"];
    for ext in vim_extensions {
        styles
            .insert(ext.to_string(), CommentStyleConfig { prefix: "\"".to_string(), suffix: None });
    }

    // Line comments with "REM"
    let batch_extensions = vec!["bat", "cmd"];
    for ext in batch_extensions {
        styles.insert(
            ext.to_string(),
            CommentStyleConfig { prefix: "REM".to_string(), suffix: None },
        );
    }

    // Special cases
    styles.insert("php".to_string(), CommentStyleConfig { prefix: "//".to_string(), suffix: None });
    styles.insert("asp".to_string(), CommentStyleConfig { prefix: "'".to_string(), suffix: None });
    styles.insert("asm".to_string(), CommentStyleConfig { prefix: ";".to_string(), suffix: None });
    styles.insert("pas".to_string(), CommentStyleConfig { prefix: "//".to_string(), suffix: None });
    styles.insert("d".to_string(), CommentStyleConfig { prefix: "//".to_string(), suffix: None });

    styles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default() {
        let config = Config::default();
        assert_eq!(config.license_header, "");
        assert_eq!(config.max_header_bytes, 8192);
        assert_eq!(config.similarity_threshold, 70);
        assert!(config.skip_empty_files);
        assert!(config.parallel_jobs.is_none());
        assert!(!config.comment_styles.is_empty());
    }

    #[test]
    fn config_with_methods() {
        let config = Config::new()
            .with_license_header("MIT License")
            .with_max_header_bytes(4096)
            .with_similarity_threshold(80)
            .with_ignore_pattern("*.tmp");

        assert_eq!(config.license_header, "MIT License");
        assert_eq!(config.max_header_bytes, 4096);
        assert_eq!(config.similarity_threshold, 80);
        assert_eq!(config.ignore_patterns, vec!["*.tmp"]);
    }

    #[test]
    fn config_get_comment_style() {
        let config = Config::default();
        let rust_style = config.get_comment_style("rs").unwrap();
        assert_eq!(rust_style.prefix, "//");
        assert_eq!(rust_style.suffix, None);

        let css_style = config.get_comment_style("css").unwrap();
        assert_eq!(css_style.prefix, "/*");
        assert_eq!(css_style.suffix.as_deref(), Some("*/"));
    }

    #[test]
    fn config_has_comment_style() {
        let config = Config::default();
        assert!(config.has_comment_style("rs"));
        assert!(config.has_comment_style("py"));
        assert!(!config.has_comment_style("unknown"));
    }

    #[test]
    fn comment_style_config_serialization() {
        let style = CommentStyleConfig { prefix: "//".to_string(), suffix: Some("*/".to_string()) };

        let serialized = serde_json::to_string(&style).unwrap();
        let deserialized: CommentStyleConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.prefix, "//");
        assert_eq!(deserialized.suffix, Some("*/".to_string()));
    }

    #[test]
    fn config_serialization() {
        let config = Config::new().with_license_header("MIT License").with_max_header_bytes(4096);

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.license_header, "MIT License");
        assert_eq!(deserialized.max_header_bytes, 4096);
    }

    #[test]
    fn default_comment_styles_coverage() {
        let styles = default_comment_styles();

        // Test a few key extensions are present
        assert!(styles.contains_key("rs"));
        assert!(styles.contains_key("py"));
        assert!(styles.contains_key("html"));
        assert!(styles.contains_key("css"));
        assert!(styles.contains_key("sql"));
        assert!(styles.contains_key("java"));
        assert!(styles.contains_key("go"));

        // Verify specific styles
        let rust = styles.get("rs").unwrap();
        assert_eq!(rust.prefix, "//");
        assert_eq!(rust.suffix, None);

        let python = styles.get("py").unwrap();
        assert_eq!(python.prefix, "#");
        assert_eq!(python.suffix, None);

        let css = styles.get("css").unwrap();
        assert_eq!(css.prefix, "/*");
        assert_eq!(css.suffix, Some("*/".to_string()));
    }

    #[test]
    fn similarity_threshold_bounds() {
        let config = Config::new().with_similarity_threshold(150);
        assert_eq!(config.similarity_threshold, 100); // Should be clamped to 100

        let config = Config::new().with_similarity_threshold(50);
        assert_eq!(config.similarity_threshold, 50); // Should remain as-is
    }
}
