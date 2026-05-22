//! Module ZIP validation for safe installation.
//!
//! Performs strict validation of module ZIP archives before installation,
//! checking for path traversal, required fields, size limits, and structural
//! integrity.

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::io::{Read, Seek};
use std::path::Path;

/// Maximum module ID length in characters.
pub const MAX_MODULE_ID_LEN: usize = 64;

/// Maximum module name length in characters.
pub const MAX_MODULE_NAME_LEN: usize = 256;

/// Maximum module version string length in characters.
pub const MAX_MODULE_VERSION_LEN: usize = 64;

/// Maximum module description length in characters.
pub const MAX_MODULE_DESCRIPTION_LEN: usize = 1024;

/// Maximum module author length in characters.
pub const MAX_MODULE_AUTHOR_LEN: usize = 256;

/// Maximum total uncompressed size of a module ZIP (500 MB).
pub const MAX_TOTAL_UNCOMPRESSED_SIZE: u64 = 500 * 1024 * 1024;

/// Maximum single file size within the ZIP (100 MB).
pub const MAX_SINGLE_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Required fields in module.prop.
const REQUIRED_FIELDS: &[&str] = &["id", "name", "version", "versionCode"];

/// Result of a single validation check.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    /// What check failed.
    pub check: String,
    /// Severity of the issue.
    pub severity: IssueSeverity,
    /// Human-readable description of the problem.
    pub message: String,
    /// How to fix the issue.
    pub suggestion: String,
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum IssueSeverity {
    /// Blocks installation.
    Error,
    /// Allowed but suspicious.
    Warning,
}

/// Result of validating a module ZIP.
#[derive(Debug, Serialize)]
pub struct ValidationReport {
    /// All issues found during validation.
    pub issues: Vec<ValidationIssue>,
    /// Module ID extracted from module.prop, if available.
    pub module_id: Option<String>,
    /// Total uncompressed size of all entries.
    pub total_size: u64,
    /// Number of entries in the ZIP.
    pub entry_count: usize,
}

impl ValidationReport {
    /// Returns true if there are no blocking errors.
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }
}

/// Validate a module ZIP archive for safe installation.
///
/// Checks for:
/// - Valid ZIP structure
/// - Presence and validity of module.prop
/// - Required fields (id, name, version, versionCode)
/// - Module ID format (alphanumeric + underscore, max 64 chars)
/// - No path traversal in entry names (../ or absolute paths)
/// - No symlinks pointing outside the module directory
/// - File size limits per entry and total
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read. Validation
/// failures are reported as issues in the returned `ValidationReport`.
pub fn validate_module_zip<P: AsRef<Path>>(path: P) -> Result<ValidationReport> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open module ZIP: {}", path.display()))?;

    validate_module_zip_from_reader(file)
}

/// Validate a module ZIP from any reader that implements Read + Seek.
pub fn validate_module_zip_from_reader<R: Read + Seek>(reader: R) -> Result<ValidationReport> {
    let mut issues = Vec::new();
    let mut module_id = None;
    let mut total_size: u64 = 0;

    let mut archive = match zip::ZipArchive::new(reader) {
        Ok(a) => a,
        Err(e) => {
            issues.push(ValidationIssue {
                check: "zip_structure".into(),
                severity: IssueSeverity::Error,
                message: format!("Not a valid ZIP archive: {e}"),
                suggestion: "Provide a valid ZIP file".into(),
            });
            return Ok(ValidationReport {
                issues,
                module_id: None,
                total_size: 0,
                entry_count: 0,
            });
        }
    };

    let entry_count = archive.len();
    let mut has_module_prop = false;

    // First pass: check all entry names for path traversal and sizes
    for i in 0..entry_count {
        let entry = archive
            .by_index(i)
            .with_context(|| format!("Failed to read ZIP entry {i}"))?;
        let name = entry.name().to_string();

        // Check for path traversal and absolute paths
        if has_path_traversal(&name) {
            issues.push(ValidationIssue {
                check: "path_traversal".into(),
                severity: IssueSeverity::Error,
                message: format!("Entry contains path traversal or absolute path: '{name}'"),
                suggestion: "Remove ../ components and use relative paths in ZIP entries".into(),
            });
        }

        // Check single file size
        let size = entry.size();
        if size > MAX_SINGLE_FILE_SIZE {
            issues.push(ValidationIssue {
                check: "file_size".into(),
                severity: IssueSeverity::Error,
                message: format!(
                    "Entry '{name}' is too large: {size} bytes (max: {MAX_SINGLE_FILE_SIZE} bytes)"
                ),
                suggestion: "Reduce file size or split into multiple files".into(),
            });
        }

        total_size = total_size.saturating_add(size);

        if let Some(mode) = entry.unix_mode() {
            let file_type = mode & 0o170000;
            let is_regular = file_type == 0 || file_type == 0o100000;
            let is_dir = file_type == 0o040000;
            if !(is_regular || is_dir) {
                issues.push(ValidationIssue {
                    check: "unsupported_entry_type".into(),
                    severity: IssueSeverity::Error,
                    message: format!(
                        "Entry '{name}' uses unsupported unix file type (mode: {mode:#o})"
                    ),
                    suggestion: "Use only regular files and directories in module ZIP".into(),
                });
            }
        }

        if name == "module.prop" {
            has_module_prop = true;
        }
    }

    // Check total uncompressed size
    if total_size > MAX_TOTAL_UNCOMPRESSED_SIZE {
        issues.push(ValidationIssue {
            check: "total_size".into(),
            severity: IssueSeverity::Error,
            message: format!(
                "Total uncompressed size too large: {total_size} bytes (max: {MAX_TOTAL_UNCOMPRESSED_SIZE} bytes)"
            ),
            suggestion: "Reduce total module size".into(),
        });
    }

    // Check for module.prop
    if has_module_prop {
        // Parse and validate module.prop
        let mut prop_entry = archive
            .by_name("module.prop")
            .with_context(|| "Failed to read module.prop from ZIP")?;
        let mut prop_content = String::new();
        prop_entry
            .read_to_string(&mut prop_content)
            .with_context(|| "Failed to read module.prop content")?;

        validate_module_prop(&prop_content, &mut issues, &mut module_id);
    } else {
        issues.push(ValidationIssue {
            check: "module_prop_missing".into(),
            severity: IssueSeverity::Error,
            message: "module.prop not found in ZIP root".into(),
            suggestion: "Add a module.prop file at the root of the ZIP".into(),
        });
    }

    Ok(ValidationReport {
        issues,
        module_id,
        total_size,
        entry_count,
    })
}

/// Validate module.prop content and extract module ID.
fn validate_module_prop(
    content: &str,
    issues: &mut Vec<ValidationIssue>,
    module_id: &mut Option<String>,
) {
    let mut props = std::collections::HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            props.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // Check required fields
    for &field in REQUIRED_FIELDS {
        match props.get(field) {
            None => {
                issues.push(ValidationIssue {
                    check: format!("required_field_{field}"),
                    severity: IssueSeverity::Error,
                    message: format!("Required field '{field}' missing from module.prop"),
                    suggestion: format!("Add '{field}=<value>' to module.prop"),
                });
            }
            Some(v) if v.is_empty() => {
                issues.push(ValidationIssue {
                    check: format!("empty_field_{field}"),
                    severity: IssueSeverity::Error,
                    message: format!("Required field '{field}' is empty in module.prop"),
                    suggestion: format!("Set a value for '{field}' in module.prop"),
                });
            }
            _ => {}
        }
    }

    // Validate module ID format
    if let Some(id) = props.get("id") {
        if id.len() > MAX_MODULE_ID_LEN {
            issues.push(ValidationIssue {
                check: "module_id_length".into(),
                severity: IssueSeverity::Error,
                message: format!(
                    "Module ID too long: {} chars (max: {MAX_MODULE_ID_LEN})",
                    id.len()
                ),
                suggestion: "Shorten the module ID".into(),
            });
        }

        let id_re =
            regex_lite::Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$").expect("static regex is valid");
        if !id_re.is_match(id) {
            issues.push(ValidationIssue {
                check: "module_id_format".into(),
                severity: IssueSeverity::Error,
                message: format!("Invalid module ID format: '{id}'"),
                suggestion:
                    "ID must start with a letter, followed by alphanumeric, dot, underscore, or hyphen"
                        .into(),
            });
        }

        *module_id = Some(id.clone());
    }

    // Validate versionCode is numeric
    if let Some(vc) = props.get("versionCode")
        && vc.parse::<u64>().is_err()
    {
        issues.push(ValidationIssue {
            check: "version_code_format".into(),
            severity: IssueSeverity::Error,
            message: format!("versionCode is not a valid integer: '{vc}'"),
            suggestion: "Set versionCode to a positive integer".into(),
        });
    }

    // Validate field lengths
    let field_limits = [
        ("name", MAX_MODULE_NAME_LEN),
        ("version", MAX_MODULE_VERSION_LEN),
        ("description", MAX_MODULE_DESCRIPTION_LEN),
        ("author", MAX_MODULE_AUTHOR_LEN),
    ];

    for (field, max_len) in field_limits {
        if let Some(value) = props.get(field)
            && value.len() > max_len
        {
            let len = value.len();
            issues.push(ValidationIssue {
                check: format!("{field}_length"),
                severity: IssueSeverity::Warning,
                message: format!(
                    "Field '{field}' is very long: {len} chars (recommended max: {max_len})"
                ),
                suggestion: format!("Consider shortening '{field}'"),
            });
        }
    }
}

/// Validate a module ID string for format correctness.
///
/// Valid IDs match: `^[a-zA-Z][a-zA-Z0-9._-]+$` with max length 64.
pub fn validate_id(id: &str) -> Result<()> {
    ensure!(!id.is_empty(), "Module ID cannot be empty");
    ensure!(
        id.len() <= MAX_MODULE_ID_LEN,
        "Module ID too long: {} chars (max: {MAX_MODULE_ID_LEN})",
        id.len()
    );

    let re = regex_lite::Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$")?;
    ensure!(
        re.is_match(id),
        "Invalid module ID: '{id}'. Must match /^[a-zA-Z][a-zA-Z0-9._-]+$/"
    );
    Ok(())
}

/// Check if a ZIP entry name contains path traversal sequences.
pub fn has_path_traversal(name: &str) -> bool {
    name.contains("../") || name.contains("..\\") || name.starts_with('/') || name.starts_with('\\')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write as _};
    use zip::write::SimpleFileOptions;

    fn stored_zip_with_unix_mode(name: &str, unix_mode: u32) -> Cursor<Vec<u8>> {
        fn push_u16(out: &mut Vec<u8>, value: u16) {
            out.extend_from_slice(&value.to_le_bytes());
        }

        fn push_u32(out: &mut Vec<u8>, value: u32) {
            out.extend_from_slice(&value.to_le_bytes());
        }

        let name = name.as_bytes();
        let name_len = u16::try_from(name.len()).unwrap();
        let mut data = Vec::new();

        let local_header_offset = u32::try_from(data.len()).unwrap();
        push_u32(&mut data, 0x0403_4b50);
        push_u16(&mut data, 20);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u32(&mut data, 0);
        push_u32(&mut data, 0);
        push_u32(&mut data, 0);
        push_u16(&mut data, name_len);
        push_u16(&mut data, 0);
        data.extend_from_slice(name);

        let central_directory_offset = u32::try_from(data.len()).unwrap();
        push_u32(&mut data, 0x0201_4b50);
        push_u16(&mut data, 0x0314);
        push_u16(&mut data, 20);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u32(&mut data, 0);
        push_u32(&mut data, 0);
        push_u32(&mut data, 0);
        push_u16(&mut data, name_len);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u32(&mut data, unix_mode << 16);
        push_u32(&mut data, local_header_offset);
        data.extend_from_slice(name);

        let central_directory_size = u32::try_from(data.len()).unwrap() - central_directory_offset;
        push_u32(&mut data, 0x0605_4b50);
        push_u16(&mut data, 0);
        push_u16(&mut data, 0);
        push_u16(&mut data, 1);
        push_u16(&mut data, 1);
        push_u32(&mut data, central_directory_size);
        push_u32(&mut data, central_directory_offset);
        push_u16(&mut data, 0);

        Cursor::new(data)
    }

    #[test]
    fn test_validate_id_valid() {
        assert!(validate_id("com.example.module").is_ok());
        assert!(validate_id("my_module-v2").is_ok());
        assert!(validate_id("aB").is_ok());
    }

    #[test]
    fn test_validate_id_invalid() {
        assert!(validate_id("").is_err());
        assert!(validate_id("1bad").is_err());
        assert!(validate_id("-bad").is_err());
        assert!(validate_id("a").is_err()); // too short (needs 2+ chars)
        assert!(validate_id(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(has_path_traversal("../etc/passwd"));
        assert!(has_path_traversal("foo/../bar"));
        assert!(has_path_traversal("/absolute/path"));
        assert!(has_path_traversal("..\\windows\\path"));
        assert!(!has_path_traversal("normal/path/file.txt"));
        assert!(!has_path_traversal("module.prop"));
    }

    #[test]
    fn test_validate_module_prop_missing_fields() {
        let mut issues = Vec::new();
        let mut module_id = None;
        validate_module_prop("", &mut issues, &mut module_id);
        assert_eq!(issues.len(), REQUIRED_FIELDS.len());
        assert!(issues.iter().all(|i| i.severity == IssueSeverity::Error));
    }

    #[test]
    fn test_validate_module_prop_valid() {
        let content = "id=com.example\nname=Test\nversion=1.0\nversionCode=1\nauthor=Test\ndescription=A test module";
        let mut issues = Vec::new();
        let mut module_id = None;
        validate_module_prop(content, &mut issues, &mut module_id);
        assert!(issues.is_empty(), "Unexpected issues: {issues:?}");
        assert_eq!(module_id, Some("com.example".to_string()));
    }

    #[test]
    fn test_validate_module_prop_bad_version_code() {
        let content = "id=com.example\nname=Test\nversion=1.0\nversionCode=not_a_number";
        let mut issues = Vec::new();
        let mut module_id = None;
        validate_module_prop(content, &mut issues, &mut module_id);
        assert!(issues.iter().any(|i| i.check == "version_code_format"));
    }

    #[test]
    fn test_validate_module_prop_bad_id() {
        let content = "id=123bad\nname=Test\nversion=1.0\nversionCode=1";
        let mut issues = Vec::new();
        let mut module_id = None;
        validate_module_prop(content, &mut issues, &mut module_id);
        assert!(issues.iter().any(|i| i.check == "module_id_format"));
    }

    #[test]
    fn test_rejects_symlink_entry_type() {
        let mut data = Cursor::new(Vec::<u8>::new());
        {
            let mut writer = zip::ZipWriter::new(&mut data);
            writer
                .add_symlink("sym", "/system/bin/sh", SimpleFileOptions::default())
                .unwrap();
            writer
                .start_file("module.prop", SimpleFileOptions::default())
                .unwrap();
            writer
                .write_all(b"id=com.example\nname=Test\nversion=1\nversionCode=1\n")
                .unwrap();
            writer.finish().unwrap();
        }
        data.set_position(0);
        let report = validate_module_zip_from_reader(data).unwrap();
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.check == "unsupported_entry_type")
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn test_rejects_character_device_entry_type() {
        let data = stored_zip_with_unix_mode("dev/char", 0o020666);
        let report = validate_module_zip_from_reader(data).unwrap();

        assert!(
            report
                .issues
                .iter()
                .any(|i| i.check == "unsupported_entry_type")
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn test_rejects_block_device_entry_type() {
        let data = stored_zip_with_unix_mode("dev/block", 0o060660);
        let report = validate_module_zip_from_reader(data).unwrap();

        assert!(
            report
                .issues
                .iter()
                .any(|i| i.check == "unsupported_entry_type")
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn test_unix_permissions_cannot_encode_symlink_type() {
        let mut data = Cursor::new(Vec::<u8>::new());
        {
            let mut writer = zip::ZipWriter::new(&mut data);
            // zip::write::unix_permissions() preserves only 0o777 permission bits;
            // it cannot encode special file types such as symlink/device.
            writer
                .start_file(
                    "not_a_symlink",
                    SimpleFileOptions::default().unix_permissions(0o120777),
                )
                .unwrap();
            writer.write_all(b"/system/bin/sh").unwrap();
            writer
                .start_file("module.prop", SimpleFileOptions::default())
                .unwrap();
            writer
                .write_all(b"id=com.example\nname=Test\nversion=1\nversionCode=1\n")
                .unwrap();
            writer.finish().unwrap();
        }

        data.set_position(0);
        let report = validate_module_zip_from_reader(data).unwrap();
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.check == "unsupported_entry_type")
        );
        assert!(report.is_valid(), "unexpected issues: {:?}", report.issues);
    }
}
