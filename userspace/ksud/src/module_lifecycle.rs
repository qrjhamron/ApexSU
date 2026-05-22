//! Transactional staged-module lifecycle helpers.
//!
//! These helpers are intentionally std-only so they can be unit-tested on host
//! while enforcing fail-closed semantics used by Android module flows.

use anyhow::{Context, Result, anyhow, ensure};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const INSTALL_COMPLETE_MARKER: &str = ".apexsu-install-complete";
const BACKUP_PREFIX: &str = ".apexsu-backup-";
const INTERNAL_PREFIX: &str = ".apexsu-";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PreservedFlags {
    pub disabled: bool,
    pub removed: bool,
}

pub fn install_complete_marker_path(staged_dir: &Path) -> PathBuf {
    staged_dir.join(INSTALL_COMPLETE_MARKER)
}

pub fn mark_install_complete(staged_dir: &Path) -> Result<()> {
    let marker = install_complete_marker_path(staged_dir);
    fs::write(&marker, b"ok")
        .with_context(|| format!("Failed to write install marker: {}", marker.display()))
}

pub fn has_install_complete_marker(staged_dir: &Path) -> bool {
    install_complete_marker_path(staged_dir).is_file()
}

pub fn cleanup_staged_dir(staged_dir: &Path) -> Result<()> {
    if staged_dir.exists() {
        fs::remove_dir_all(staged_dir).with_context(|| {
            format!(
                "Failed to clean staged module directory: {}",
                staged_dir.display()
            )
        })?;
    }
    Ok(())
}

pub fn run_staging_transaction<T, F>(staged_dir: &Path, op: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    match op() {
        Ok(v) => Ok(v),
        Err(e) => {
            if let Err(clean_err) = cleanup_staged_dir(staged_dir) {
                return Err(anyhow!("{e}; cleanup failed: {clean_err:#}"));
            }
            Err(e)
        }
    }
}

pub fn should_promote_staged_module(staged_dir: &Path) -> bool {
    has_install_complete_marker(staged_dir)
}

pub fn is_internal_module_dir(name: &str) -> bool {
    name.is_empty() || name == "." || name == ".." || name.starts_with('.') || name.starts_with(INTERNAL_PREFIX)
}

pub fn is_valid_active_module_id(name: &str) -> bool {
    if is_internal_module_dir(name) {
        return false;
    }

    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return false;
    }

    crate::module_validator::validate_id(name).is_ok()
}

fn unique_backup_path(active_dir: &Path) -> Result<PathBuf> {
    let parent = active_dir
        .parent()
        .ok_or_else(|| anyhow!("Active module path has no parent: {}", active_dir.display()))?;
    let name = active_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Active module path has invalid name: {}", active_dir.display()))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    Ok(parent.join(format!("{BACKUP_PREFIX}{name}-{pid}-{ts}")))
}

fn restore_backup_or_fail(
    backup: &Path,
    active_dir: &Path,
    original_err: &anyhow::Error,
) -> anyhow::Error {
    match fs::rename(backup, active_dir) {
        Ok(_) => anyhow!(
            "{original_err:#}; rolled back active module from backup {}",
            backup.display()
        ),
        Err(restore_err) => anyhow!(
            "{original_err:#}; rollback failed, manual recovery required: backup={} active={} restore_err={restore_err}",
            backup.display(),
            active_dir.display()
        ),
    }
}

fn promote_staged_module_with_renamer(
    staged_dir: &Path,
    active_dir: &Path,
    renamer: &mut dyn FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<Option<PathBuf>> {
    ensure!(
        staged_dir.exists() && staged_dir.is_dir(),
        "Staged module missing or not a directory: {}",
        staged_dir.display()
    );
    ensure!(
        should_promote_staged_module(staged_dir),
        "Staged module missing install marker: {}",
        staged_dir.display()
    );

    let mut backup_path = None;

    if active_dir.exists() {
        let backup = unique_backup_path(active_dir)?;
        renamer(active_dir, &backup).with_context(|| {
            format!(
                "Failed to move active module to backup: {} -> {}",
                active_dir.display(),
                backup.display()
            )
        })?;
        backup_path = Some(backup);
    }

    if let Err(e) = renamer(staged_dir, active_dir).with_context(|| {
        format!(
            "Failed to promote staged module: {} -> {}",
            staged_dir.display(),
            active_dir.display()
        )
    }) {
        if let Some(backup) = backup_path.as_ref() {
            return Err(restore_backup_or_fail(backup, active_dir, &e));
        }
        return Err(e);
    }

    Ok(backup_path)
}

pub fn promote_staged_module(staged_dir: &Path, active_dir: &Path) -> Result<Option<PathBuf>> {
    let mut renamer = |from: &Path, to: &Path| fs::rename(from, to);
    promote_staged_module_with_renamer(staged_dir, active_dir, &mut renamer)
}

pub fn finalize_successful_promotion(
    active_dir: &Path,
    backup_path: Option<PathBuf>,
    preserved_flags: PreservedFlags,
    disable_marker_name: &str,
    remove_marker_name: &str,
) -> Result<()> {
    if preserved_flags.removed {
        if let Err(e) = fs::write(active_dir.join(remove_marker_name), b"") {
            let disable_marker = active_dir.join(disable_marker_name);
            let _ = fs::write(&disable_marker, b"");
            return Err(anyhow!(
                "Failed to preserve remove marker: {e}; module disabled for fail-closed recovery at {}",
                disable_marker.display()
            ));
        }
    } else if preserved_flags.disabled {
        let disable_marker = active_dir.join(disable_marker_name);
        fs::write(&disable_marker, b"").with_context(|| {
            format!(
                "Failed to preserve disable marker for promoted module: {}",
                disable_marker.display()
            )
        })?;
    }

    if let Some(backup) = backup_path {
        if backup.exists() {
            fs::remove_dir_all(&backup).with_context(|| {
                format!(
                    "Failed to remove module backup after promotion: {}",
                    backup.display()
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn promote_staged_module_for_test(
    staged_dir: &Path,
    active_dir: &Path,
    renamer: &mut dyn FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<Option<PathBuf>> {
    promote_staged_module_with_renamer(staged_dir, active_dir, renamer)
}
