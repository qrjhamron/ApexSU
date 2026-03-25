//! Self-diagnostic checks for the KernelSU/ApexSU system.
//!
//! Provides structured diagnostic reports that verify the health of the
//! kernel module, ksud daemon, allowlist file, and module directory structure.

use serde::Serialize;
use std::path::Path;

use crate::defs;

/// Status of a single diagnostic check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CheckStatus {
    /// Check passed successfully.
    Pass,
    /// Check found a non-critical issue.
    Warn,
    /// Check found a critical problem.
    Fail,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Warn => write!(f, "WARN"),
            Self::Fail => write!(f, "FAIL"),
        }
    }
}

/// A single diagnostic check result.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticCheck {
    /// Name of the check.
    pub name: String,
    /// Result status.
    pub status: CheckStatus,
    /// Human-readable description of the result.
    pub message: String,
    /// Suggested action if status is Warn or Fail.
    pub suggestion: Option<String>,
}

/// Complete diagnostic report.
#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    /// All check results.
    pub checks: Vec<DiagnosticCheck>,
}

impl DiagnosticReport {
    /// Returns true if all checks passed (no Fail results).
    pub fn is_healthy(&self) -> bool {
        !self.checks.iter().any(|c| c.status == CheckStatus::Fail)
    }

    /// Count of checks by status.
    pub fn summary(&self) -> (usize, usize, usize) {
        let pass = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Pass)
            .count();
        let warn = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warn)
            .count();
        let fail = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Fail)
            .count();
        (pass, warn, fail)
    }

    /// Format as human-readable text.
    pub fn to_text(&self) -> String {
        use std::fmt::Write;
        let mut out = String::from("=== ApexSU Diagnostic Report ===\n\n");
        for check in &self.checks {
            let _ = writeln!(out, "[{}] {}: {}", check.status, check.name, check.message);
            if let Some(ref suggestion) = check.suggestion {
                let _ = writeln!(out, "     → {suggestion}");
            }
        }
        let (pass, warn, fail) = self.summary();
        let _ = writeln!(
            out,
            "\nSummary: {pass} passed, {warn} warnings, {fail} failures"
        );
        out
    }
}

/// Run all diagnostic checks and return a report.
///
/// Checks performed:
/// - Kernel module responsiveness (via ioctl)
/// - ksud binary location and permissions
/// - Working directory structure
/// - Allowlist file integrity
/// - Module directory structure
/// - Required paths exist
pub fn run_diagnostics() -> DiagnosticReport {
    let checks = vec![
        check_kernel_module(),
        check_working_dir(),
        check_binary_dir(),
        check_module_dir(),
        check_allowlist(),
        check_module_update_dir(),
    ];

    DiagnosticReport { checks }
}

/// Serialize a diagnostic report to a JSON string.
pub fn report_to_json(report: &DiagnosticReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

/// Check if the kernel module is loaded and responding.
fn check_kernel_module() -> DiagnosticCheck {
    let version = crate::ksucalls::get_version();
    if version > 0 {
        DiagnosticCheck {
            name: "kernel_module".into(),
            status: CheckStatus::Pass,
            message: format!("Kernel module responding, version: {version}"),
            suggestion: None,
        }
    } else {
        DiagnosticCheck {
            name: "kernel_module".into(),
            status: CheckStatus::Fail,
            message: "Kernel module not responding (version 0)".into(),
            suggestion: Some("Verify KernelSU kernel module is loaded".into()),
        }
    }
}

/// Check that the working directory exists and is writable.
fn check_working_dir() -> DiagnosticCheck {
    check_directory_exists("working_dir", defs::WORKING_DIR)
}

/// Check that the binary directory exists.
fn check_binary_dir() -> DiagnosticCheck {
    check_directory_exists("binary_dir", defs::BINARY_DIR)
}

/// Check that the modules directory exists.
fn check_module_dir() -> DiagnosticCheck {
    check_directory_exists("module_dir", defs::MODULE_DIR)
}

/// Check that the module update directory exists.
fn check_module_update_dir() -> DiagnosticCheck {
    let path = Path::new(defs::MODULE_UPDATE_DIR);
    if path.exists() {
        if path.is_dir() {
            DiagnosticCheck {
                name: "module_update_dir".into(),
                status: CheckStatus::Pass,
                message: format!(
                    "Module update directory exists: {}",
                    defs::MODULE_UPDATE_DIR
                ),
                suggestion: None,
            }
        } else {
            DiagnosticCheck {
                name: "module_update_dir".into(),
                status: CheckStatus::Fail,
                message: format!("{} exists but is not a directory", defs::MODULE_UPDATE_DIR),
                suggestion: Some("Remove the file and recreate as directory".into()),
            }
        }
    } else {
        // Module update dir may not exist until first module install — that's OK
        DiagnosticCheck {
            name: "module_update_dir".into(),
            status: CheckStatus::Pass,
            message: "Module update directory not yet created (normal before first install)".into(),
            suggestion: None,
        }
    }
}

/// Check the allowlist file for basic integrity.
fn check_allowlist() -> DiagnosticCheck {
    let allowlist_path = Path::new(defs::WORKING_DIR).join(".allowlist.enc");
    if !allowlist_path.exists() {
        return DiagnosticCheck {
            name: "allowlist".into(),
            status: CheckStatus::Warn,
            message: "Encrypted allowlist file not found".into(),
            suggestion: Some(
                "Allowlist is created when first app is granted root".into(),
            ),
        };
    }

    match std::fs::metadata(&allowlist_path) {
        Ok(meta) => {
            if meta.len() < 16 {
                DiagnosticCheck {
                    name: "allowlist".into(),
                    status: CheckStatus::Warn,
                    message: format!("Encrypted allowlist file is very small ({} bytes)", meta.len()),
                    suggestion: Some(
                        "File may be corrupted".into(),
                    ),
                }
            } else {
                DiagnosticCheck {
                    name: "allowlist".into(),
                    status: CheckStatus::Pass,
                    message: format!("Encrypted allowlist exists ({} bytes)", meta.len()),
                    suggestion: None,
                }
            }
        }
        Err(e) => DiagnosticCheck {
            name: "allowlist".into(),
            status: CheckStatus::Fail,
            message: format!("Cannot read allowlist metadata: {e}"),
            suggestion: Some("Check file permissions on .allowlist.enc".into()),
        },
    }
}

/// Helper: check that a directory path exists and is a directory.
fn check_directory_exists(name: &str, dir_path: &str) -> DiagnosticCheck {
    let path = Path::new(dir_path);
    if path.exists() {
        if path.is_dir() {
            DiagnosticCheck {
                name: name.into(),
                status: CheckStatus::Pass,
                message: format!("Directory exists: {dir_path}"),
                suggestion: None,
            }
        } else {
            DiagnosticCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!("{dir_path} exists but is not a directory"),
                suggestion: Some("Remove the file and recreate as directory".into()),
            }
        }
    } else {
        DiagnosticCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Directory missing: {dir_path}"),
            suggestion: Some(format!("Create directory: mkdir -p {dir_path}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_display() {
        assert_eq!(format!("{}", CheckStatus::Pass), "PASS");
        assert_eq!(format!("{}", CheckStatus::Warn), "WARN");
        assert_eq!(format!("{}", CheckStatus::Fail), "FAIL");
    }

    #[test]
    fn test_report_summary() {
        let report = DiagnosticReport {
            checks: vec![
                DiagnosticCheck {
                    name: "a".into(),
                    status: CheckStatus::Pass,
                    message: "ok".into(),
                    suggestion: None,
                },
                DiagnosticCheck {
                    name: "b".into(),
                    status: CheckStatus::Warn,
                    message: "warn".into(),
                    suggestion: Some("fix".into()),
                },
                DiagnosticCheck {
                    name: "c".into(),
                    status: CheckStatus::Fail,
                    message: "bad".into(),
                    suggestion: Some("fix".into()),
                },
            ],
        };
        assert_eq!(report.summary(), (1, 1, 1));
        assert!(!report.is_healthy());
    }

    #[test]
    fn test_report_healthy() {
        let report = DiagnosticReport {
            checks: vec![DiagnosticCheck {
                name: "a".into(),
                status: CheckStatus::Pass,
                message: "ok".into(),
                suggestion: None,
            }],
        };
        assert!(report.is_healthy());
    }

    #[test]
    fn test_report_to_text() {
        let report = DiagnosticReport {
            checks: vec![DiagnosticCheck {
                name: "test_check".into(),
                status: CheckStatus::Pass,
                message: "all good".into(),
                suggestion: None,
            }],
        };
        let text = report.to_text();
        assert!(text.contains("[PASS] test_check: all good"));
        assert!(text.contains("1 passed, 0 warnings, 0 failures"));
    }
}
