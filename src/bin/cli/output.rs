use fast_license_checker::types::ScanSummary;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for programmatic use
    Json,
    /// GitHub Actions annotation format
    Github,
}

/// Helper function to write to stdout, ignoring errors (e.g., broken pipe)
/// This is acceptable for CLI output where broken pipes are common
#[inline]
fn write_stdout(writer: &mut impl Write, s: &str) {
    let _ = writer.write_all(s.as_bytes());
}

/// Helper function to write formatted output to stdout
#[inline]
fn write_fmt_stdout(writer: &mut impl Write, args: std::fmt::Arguments) {
    let _ = writer.write_fmt(args);
}

pub fn print_summary(summary: &ScanSummary, format: OutputFormat, color: bool) {
    match format {
        OutputFormat::Text => print_text(summary, color),
        OutputFormat::Json => print_json(summary),
        OutputFormat::Github => print_github(summary),
    }
}

#[allow(clippy::arithmetic_side_effects)] // Intentional arithmetic for progress bar calculation
fn print_text(summary: &ScanSummary, color: bool) {
    let mut stdout = std::io::stdout().lock();

    if summary.total == 0 {
        write_fmt_stdout(&mut stdout, format_args!("No files found to check\n"));
        return;
    }

    // Summary header
    if color {
        write_stdout(&mut stdout, "\x1b[1;36m"); // Cyan bold
    }
    write_stdout(&mut stdout, "License Header Check Results");
    if color {
        write_stdout(&mut stdout, "\x1b[0m");
    }
    write_stdout(&mut stdout, "\n");

    // Progress bar style summary
    let passed_pct = if summary.total > 0 {
        (summary.passed as f64 / summary.total as f64 * 100.0) as u8
    } else {
        0
    };

    if color {
        write_stdout(&mut stdout, "\x1b[32m"); // Green
    }
    write_fmt_stdout(&mut stdout, format_args!("✓ Passed: {}", summary.passed));
    if color {
        write_stdout(&mut stdout, "\x1b[0m");
    }

    if summary.failed > 0 {
        if color {
            write_stdout(&mut stdout, "  \x1b[31m"); // Red
        }
        write_fmt_stdout(&mut stdout, format_args!("✗ Failed: {}", summary.failed));
        if color {
            write_stdout(&mut stdout, "\x1b[0m");
        }
    }

    if summary.skipped > 0 {
        if color {
            write_stdout(&mut stdout, "  \x1b[33m"); // Yellow
        }
        write_fmt_stdout(&mut stdout, format_args!("⚠ Skipped: {}", summary.skipped));
        if color {
            write_stdout(&mut stdout, "\x1b[0m");
        }
    }

    if color {
        write_stdout(&mut stdout, "  \x1b[36m"); // Cyan
    }
    write_fmt_stdout(&mut stdout, format_args!("Total: {}", summary.total));
    if color {
        write_stdout(&mut stdout, "\x1b[0m");
    }
    write_stdout(&mut stdout, "\n");

    // Show progress bar
    if summary.total > 0 {
        let bar_width = 40;
        let filled = (passed_pct as usize * bar_width) / 100;
        let empty = bar_width - filled;

        if color {
            write_stdout(&mut stdout, "\x1b[32m"); // Green
        }
        write_stdout(&mut stdout, "[");
        for _ in 0..filled {
            write_stdout(&mut stdout, "█");
        }
        if color {
            write_stdout(&mut stdout, "\x1b[31m"); // Red
        }
        for _ in 0..empty {
            write_stdout(&mut stdout, "░");
        }
        if color {
            write_stdout(&mut stdout, "\x1b[0m");
        }
        write_fmt_stdout(&mut stdout, format_args!("] {}%\n", passed_pct));
    }

    // Show details if there are failures or if verbose
    if summary.failed > 0 || summary.skipped > 0 {
        write_stdout(&mut stdout, "\n");
        if color {
            write_stdout(&mut stdout, "\x1b[1m"); // Bold
        }
        write_stdout(&mut stdout, "Details:\n");
        if color {
            write_stdout(&mut stdout, "\x1b[0m");
        }

        // Show failed files
        if summary.failed > 0 {
            if color {
                write_stdout(&mut stdout, "\x1b[31m"); // Red
            }
            write_stdout(&mut stdout, "Failed files:\n");
            if color {
                write_stdout(&mut stdout, "\x1b[0m");
            }

            // Note: In a real implementation, we'd iterate through results
            // For now, just show the count
            write_fmt_stdout(
                &mut stdout,
                format_args!("  {} files missing license headers\n", summary.failed),
            );
        }

        // Show skipped files
        if summary.skipped > 0 {
            if color {
                write_stdout(&mut stdout, "\x1b[33m"); // Yellow
            }
            write_stdout(&mut stdout, "Skipped files:\n");
            if color {
                write_stdout(&mut stdout, "\x1b[0m");
            }

            // Note: In a real implementation, we'd show reasons
            write_fmt_stdout(
                &mut stdout,
                format_args!("  {} files skipped (binary, unsupported, etc.)\n", summary.skipped),
            );
        }
    }
}

fn print_json(summary: &ScanSummary) {
    let mut stdout = std::io::stdout().lock();

    // Create JSON structure
    let mut summary_obj = serde_json::Map::new();
    summary_obj.insert("total".to_string(), serde_json::Value::Number(summary.total.into()));
    summary_obj.insert("passed".to_string(), serde_json::Value::Number(summary.passed.into()));
    summary_obj.insert("failed".to_string(), serde_json::Value::Number(summary.failed.into()));
    summary_obj.insert("skipped".to_string(), serde_json::Value::Number(summary.skipped.into()));

    let mut root_obj = serde_json::Map::new();
    root_obj.insert("summary".to_string(), serde_json::Value::Object(summary_obj));
    root_obj.insert("results".to_string(), serde_json::Value::Array(Vec::new()));

    let json = serde_json::Value::Object(root_obj);

    // JSON serialization should never fail for our simple structure
    if let Ok(json_str) = serde_json::to_string_pretty(&json) {
        write_fmt_stdout(&mut stdout, format_args!("{}\n", json_str));
    }
}

fn print_github(summary: &ScanSummary) {
    let mut stdout = std::io::stdout().lock();

    // GitHub Actions annotations format
    if summary.failed > 0 {
        write_fmt_stdout(&mut stdout, format_args!(
            "::error title=License Check Failed::Found {} files missing license headers out of {} total files\n",
            summary.failed, summary.total
        ));
    } else if summary.total == 0 {
        write_stdout(
            &mut stdout,
            "::warning title=No Files Found::No files found to check for license headers\n",
        );
    } else {
        write_fmt_stdout(
            &mut stdout,
            format_args!(
                "::notice title=License Check Passed::All {} files have valid license headers\n",
                summary.total
            ),
        );
    }
}
