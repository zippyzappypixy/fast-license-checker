use fast_license_checker::types::ScanSummary;

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output for programmatic use
    Json,
    /// GitHub Actions annotation format
    Github,
}

pub fn print_summary(summary: &ScanSummary, format: OutputFormat, color: bool) {
    match format {
        OutputFormat::Text => print_text(summary, color),
        OutputFormat::Json => print_json(summary),
        OutputFormat::Github => print_github(summary),
    }
}

fn print_text(summary: &ScanSummary, color: bool) {
    use std::io::Write;

    let mut stdout = std::io::stdout();

    if summary.total == 0 {
        writeln!(stdout, "No files found to check").unwrap();
        return;
    }

    // Summary header
    if color {
        write!(stdout, "\x1b[1;36m").unwrap(); // Cyan bold
    }
    write!(stdout, "License Header Check Results").unwrap();
    if color {
        write!(stdout, "\x1b[0m").unwrap();
    }
    writeln!(stdout).unwrap();

    // Progress bar style summary
    let passed_pct = if summary.total > 0 {
        (summary.passed as f64 / summary.total as f64 * 100.0) as u8
    } else {
        0
    };

    if color {
        write!(stdout, "\x1b[32m").unwrap(); // Green
    }
    write!(stdout, "✓ Passed: {}", summary.passed).unwrap();
    if color {
        write!(stdout, "\x1b[0m").unwrap();
    }

    if summary.failed > 0 {
        if color {
            write!(stdout, "  \x1b[31m").unwrap(); // Red
        }
        write!(stdout, "✗ Failed: {}", summary.failed).unwrap();
        if color {
            write!(stdout, "\x1b[0m").unwrap();
        }
    }

    if summary.skipped > 0 {
        if color {
            write!(stdout, "  \x1b[33m").unwrap(); // Yellow
        }
        write!(stdout, "⚠ Skipped: {}", summary.skipped).unwrap();
        if color {
            write!(stdout, "\x1b[0m").unwrap();
        }
    }

    if color {
        write!(stdout, "  \x1b[36m").unwrap(); // Cyan
    }
    write!(stdout, "Total: {}", summary.total).unwrap();
    if color {
        write!(stdout, "\x1b[0m").unwrap();
    }
    writeln!(stdout).unwrap();

    // Show progress bar
    if summary.total > 0 {
        let bar_width = 40;
        let filled = (passed_pct as usize * bar_width) / 100;
        let empty = bar_width - filled;

        if color {
            write!(stdout, "\x1b[32m").unwrap(); // Green
        }
        write!(stdout, "[").unwrap();
        for _ in 0..filled {
            write!(stdout, "█").unwrap();
        }
        if color {
            write!(stdout, "\x1b[31m").unwrap(); // Red
        }
        for _ in 0..empty {
            write!(stdout, "░").unwrap();
        }
        if color {
            write!(stdout, "\x1b[0m").unwrap();
        }
        writeln!(stdout, "] {}%", passed_pct).unwrap();
    }

    // Show details if there are failures or if verbose
    if summary.failed > 0 || summary.skipped > 0 {
        writeln!(stdout).unwrap();
        if color {
            write!(stdout, "\x1b[1m").unwrap(); // Bold
        }
        writeln!(stdout, "Details:").unwrap();
        if color {
            write!(stdout, "\x1b[0m").unwrap();
        }

        // Show failed files
        if summary.failed > 0 {
            if color {
                write!(stdout, "\x1b[31m").unwrap(); // Red
            }
            writeln!(stdout, "Failed files:").unwrap();
            if color {
                write!(stdout, "\x1b[0m").unwrap();
            }

            // Note: In a real implementation, we'd iterate through results
            // For now, just show the count
            writeln!(stdout, "  {} files missing license headers", summary.failed).unwrap();
        }

        // Show skipped files
        if summary.skipped > 0 {
            if color {
                write!(stdout, "\x1b[33m").unwrap(); // Yellow
            }
            writeln!(stdout, "Skipped files:").unwrap();
            if color {
                write!(stdout, "\x1b[0m").unwrap();
            }

            // Note: In a real implementation, we'd show reasons
            writeln!(stdout, "  {} files skipped (binary, unsupported, etc.)", summary.skipped).unwrap();
        }
    }
}

fn print_json(summary: &ScanSummary) {
    use std::io::Write;

    let mut stdout = std::io::stdout();

    // Create JSON structure
    let json = serde_json::json!({
        "summary": {
            "total": summary.total,
            "passed": summary.passed,
            "failed": summary.failed,
            "skipped": summary.skipped
        },
        "results": []  // In a full implementation, this would contain detailed results
    });

    writeln!(stdout, "{}", serde_json::to_string_pretty(&json).unwrap()).unwrap();
}

fn print_github(summary: &ScanSummary) {
    use std::io::Write;

    let mut stdout = std::io::stdout();

    // GitHub Actions annotations format
    if summary.failed > 0 {
        writeln!(stdout, "::error title=License Check Failed::Found {} files missing license headers out of {} total files",
                summary.failed, summary.total).unwrap();
    } else if summary.total == 0 {
        writeln!(stdout, "::warning title=No Files Found::No files found to check for license headers").unwrap();
    } else {
        writeln!(stdout, "::notice title=License Check Passed::All {} files have valid license headers",
                summary.total).unwrap();
    }
}
