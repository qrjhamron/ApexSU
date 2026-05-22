//! Module installation, lifecycle management, and overlay handling.
//!
//! KernelSU modules are standard zip files that overlay the Android system
//! via the magic-mount (overlayfs) mechanism.

#[allow(clippy::wildcard_imports)]
use crate::utils::*;
use crate::{
    assets, defs, ksucalls, metamodule, module_validator,
    module_lifecycle::{
        PreservedFlags, cleanup_staged_dir, finalize_successful_promotion,
        is_valid_active_module_id, mark_install_complete, promote_staged_module,
        run_staging_transaction, should_promote_staged_module,
    },
    restorecon::{restore_syscon, setsyscon},
    sepolicy,
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use const_format::concatcp;
use is_executable::is_executable;
use java_properties::PropertiesIter;
use log::{debug, info, warn};

use std::fs::copy;
use std::{
    collections::HashMap,
    env::var as env_var,
    fs::{File, Permissions, canonicalize, remove_dir_all, set_permissions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use zip_extensions::inflate::zip_extract::zip_extract_file_to_memory;

use crate::defs::{MODULE_DIR, MODULE_UPDATE_DIR, UPDATE_FILE_NAME};
use crate::module::ModuleType::{Active, All};
#[cfg(unix)]
use std::os::unix::{prelude::PermissionsExt, process::CommandExt};

const INSTALLER_CONTENT: &str = include_str!("./installer.sh");
const INSTALL_MODULE_SCRIPT: &str = concatcp!(
    INSTALLER_CONTENT,
    "\n",
    "install_module",
    "\n",
    "exit 0",
    "\n"
);

/// Validate module_id format and security.
/// Module ID must match: ^[a-zA-Z][a-zA-Z0-9._-]+$
pub fn validate_module_id(module_id: &str) -> Result<()> {
    module_validator::validate_id(module_id)
}

/// Returns environment variables required for busybox script execution.
pub fn get_common_script_envs() -> Vec<(&'static str, String)> {
    vec![
        /* Standalone mode prevents interference with existing shell environments */
        ("ASH_STANDALONE", "1".to_string()),
        ("KSU", "true".to_string()),
        ("KSU_KERNEL_VER_CODE", ksucalls::get_version().to_string()),
        ("KSU_VER_CODE", defs::VERSION_CODE.to_string()),
        ("KSU_VER", defs::VERSION_NAME.to_string()),
        (
            "PATH",
            format!(
                "{}:{}",
                env_var("PATH").unwrap_or_default(),
                defs::BINARY_DIR.trim_end_matches('/')
            ),
        ),
    ]
}

fn exec_install_script(module_file: &str, is_metamodule: bool) -> Result<()> {
    let realpath = std::fs::canonicalize(module_file)
        .with_context(|| format!("realpath: {module_file} failed"))?;

    let install_script =
        metamodule::get_install_script(is_metamodule, INSTALLER_CONTENT, INSTALL_MODULE_SCRIPT)?;

    let result = Command::new(assets::BUSYBOX_PATH)
        .args(["sh", "-c", &install_script])
        .envs(get_common_script_envs())
        .env("OUTFD", "1")
        .env("ZIPFILE", realpath)
        .status()?;
    ensure!(result.success(), "Failed to install module script");
    Ok(())
}

fn ensure_boot_completed() -> Result<()> {
    /* Refuse modifications during early boot to prevent race conditions 
     * with the mount system. */
    if getprop("sys.boot_completed").as_deref() != Some("1") {
        bail!("Android is Booting!");
    }
    Ok(())
}

#[derive(PartialEq, Eq)]
pub enum ModuleType {
    All,
    Active,
    Updated,
}

/// Iterates over modules of a specific type.
pub fn foreach_module(
    module_type: ModuleType,
    mut f: impl FnMut(&Path) -> Result<()>,
) -> Result<()> {
    let modules_dir = Path::new(match module_type {
        ModuleType::Updated => MODULE_UPDATE_DIR,
        _ => defs::MODULE_DIR,
    });
    let dir = std::fs::read_dir(modules_dir)?;
    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            warn!("{} is not a directory, skip", path.display());
            continue;
        }

        if module_type == Active {
            let Some(name) = entry.file_name().to_str().map(ToString::to_string) else {
                warn!(
                    "Active module directory has non-utf8 name, skip: {}",
                    path.display()
                );
                continue;
            };

            if !is_valid_active_module_id(&name) {
                warn!("Skipping invalid/internal module directory: {}", path.display());
                continue;
            }

            if path.join(defs::DISABLE_FILE_NAME).exists() {
                info!("{} is disabled, skip", path.display());
                continue;
            }
            if path.join(defs::REMOVE_FILE_NAME).exists() {
                warn!("{} is removed, skip", path.display());
                continue;
            }
        }

        f(&path)?;
    }

    Ok(())
}

fn foreach_active_module(f: impl FnMut(&Path) -> Result<()>) -> Result<()> {
    foreach_module(Active, f)
}

/// Load and apply SELinux policy rules from all active modules.
pub fn load_sepolicy_rule() -> Result<()> {
    foreach_active_module(|path| {
        let rule_file = path.join("sepolicy.rule");
        if !rule_file.exists() {
            return Ok(());
        }
        info!("load policy: {}", &rule_file.display());

        if sepolicy::apply_file(&rule_file).is_err() {
            /* Log but continue — a single broken rule file should not block others */
            warn!("Failed to load sepolicy.rule for {}", &rule_file.display());
        }
        Ok(())
    })?;

    Ok(())
}

/// Execute a shell script with busybox, optionally waiting for completion and with a timeout.
pub fn exec_script<T: AsRef<Path>>(path: T, wait: bool, timeout_sec: u32) -> Result<()> {
    info!("exec {}", path.as_ref().display());

    let is_module_script = path.as_ref().starts_with(defs::MODULE_DIR);
    let module_id = if is_module_script {
        path.as_ref()
            .strip_prefix(defs::MODULE_DIR)
            .ok()
            .and_then(|p| p.components().next())
            .and_then(|c| c.as_os_str().to_str())
            .map(ToString::to_string)
    } else {
        None
    };

    let validated_module_id = module_id
        .as_ref()
        .and_then(|id| match validate_module_id(id) {
            Ok(()) => {
                debug!("Module ID extracted from script path: '{id}'");
                Some(id.as_str())
            }
            Err(e) => {
                warn!(
                    "Invalid module ID '{id}' extracted from script path '{}': {e}",
                    path.as_ref().display(),
                );
                None
            }
        });

    let mut command = &mut Command::new(assets::BUSYBOX_PATH);
    #[cfg(unix)]
    {
        /* detach from parent process group to prevent init from killing our children */
        command = command.process_group(0);
        // SAFETY: pre_exec runs in a forked child before exec; switch_cgroups is safe here.
        command = unsafe {
            command.pre_exec(|| {
                /* move to root cgroups so we aren't throttled by app freezer */
                switch_cgroups();
                Ok(())
            })
        };
    }

    command = command.current_dir(
        path.as_ref()
            .parent()
            .context("script path has no parent directory")?,
    );

    if timeout_sec > 0 {
        command = command
            .arg("timeout")
            .arg("-s")
            .arg("9")
            .arg(format!("{timeout_sec}s"));
    }

    command = command
        .arg("sh")
        .arg(path.as_ref())
        .envs(get_common_script_envs());

    if let Some(id) = validated_module_id {
        command = command.env("KSU_MODULE", id);
    }

    let result = if wait {
        command.status().map(|_| ())
    } else {
        command.spawn().map(|_| ())
    };
    result.map_err(|e| anyhow!("Failed to exec {}: {e}", path.as_ref().display()))
}

/// Execute stage-specific scripts (e.g., post-fs-data, service) for all active modules.
pub fn exec_stage_script(stage: &str, block: bool, timeout_sec: u32) -> Result<()> {
    let metamodule_dir = metamodule::get_metamodule_path().and_then(|path| canonicalize(path).ok());

    foreach_active_module(|module| {
        /* skip metamodule as it handles its own stages */
        if metamodule_dir.as_ref().is_some_and(|meta_dir| {
            canonicalize(module)
                .map(|resolved| resolved == *meta_dir)
                .unwrap_or(false)
        }) {
            return Ok(());
        }

        let script_path = module.join(format!("{stage}.sh"));
        if !script_path.exists() {
            return Ok(());
        }

        exec_script(&script_path, block, timeout_sec)
    })?;

    Ok(())
}

/// Execute all scripts in a common directory (e.g., post-fs-data.d, service.d).
pub fn exec_common_scripts(dir: &str, wait: bool, timeout_sec: u32) -> Result<()> {
    let script_dir = Path::new(defs::ADB_DIR).join(dir);
    if !script_dir.exists() {
        info!("{} not exists, skip", script_dir.display());
        return Ok(());
    }

    let dir = std::fs::read_dir(&script_dir)?;
    for entry in dir.flatten() {
        let path = entry.path();

        if !is_executable(&path) {
            warn!("{} is not executable, skip", path.display());
            continue;
        }

        exec_script(path, wait, timeout_sec)?;
    }

    Ok(())
}

/// Load system.prop files from all active modules.
pub fn load_system_prop() -> Result<()> {
    foreach_active_module(|module| {
        let system_prop = module.join("system.prop");
        if !system_prop.exists() {
            return Ok(());
        }
        info!("load {} system.prop", module.display());

        /* Use resetprop -n to bypass immediate property service notifications */
        Command::new(assets::RESETPROP_PATH)
            .arg("-n")
            .arg("--file")
            .arg(&system_prop)
            .status()
            .with_context(|| format!("Failed to exec {}", system_prop.display()))?;

        Ok(())
    })?;

    Ok(())
}

/// Remove modules marked for deletion (those with `remove` flag file).
pub fn prune_modules() -> Result<()> {
    const PRUNE_FAILED_FILE: &str = concatcp!(defs::MODULE_DIR, ".prune_failed");

    /* retry cleanup of directories that were busy during the last prune attempt */
    if Path::new(PRUNE_FAILED_FILE).exists() {
        if let std::result::Result::Ok(content) = std::fs::read_to_string(PRUNE_FAILED_FILE) {
            let mut remaining = Vec::new();
            for line in content.lines() {
                let path = Path::new(line);
                if path.exists() {
                    if let Err(e) = remove_dir_all(path) {
                        warn!("Retry prune failed for {}: {e}", path.display());
                        remaining.push(line);
                    } else {
                        info!("Retry prune success for {}", path.display());
                    }
                }
            }
            if remaining.is_empty() {
                let _ = std::fs::remove_file(PRUNE_FAILED_FILE);
            } else {
                let _ = std::fs::write(PRUNE_FAILED_FILE, remaining.join("\n"));
            }
        }
    }

    foreach_module(All, |module| {
        if !module.join(defs::REMOVE_FILE_NAME).exists() {
            return Ok(());
        }

        info!("remove module: {}", module.display());

        let module_id = module.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let is_metamodule = read_module_prop(module)
            .map(|props| metamodule::is_metamodule(&props))
            .unwrap_or(false);

        if is_metamodule {
            info!("Removing metamodule symlink");
            if let Err(e) = metamodule::remove_symlink() {
                warn!("Failed to remove metamodule symlink: {e}");
            }
        } else if let Err(e) = metamodule::exec_metauninstall_script(module_id) {
            warn!("Failed to exec metamodule uninstall for {module_id}: {e}",);
        }

        let uninstaller = module.join("uninstall.sh");
        if uninstaller.exists()
            && let Err(e) = exec_script(uninstaller, true, 60)
        {
            warn!("Failed to exec uninstaller: {e}");
        }

        if let Err(e) = crate::module_config::clear_module_configs(module_id) {
            warn!("Failed to clear configs for {module_id}: {e}");
        }

        if let Err(e) = remove_dir_all(module) {
            warn!("Failed to remove {}: {e}", module.display());
            /* Persistent record for retry; likely busy due to active mounts */
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(PRUNE_FAILED_FILE)?;
            writeln!(file, "{}", module.display())?;
        }

        Ok(())
    })?;

    Ok(())
}

/// Process module updates: move pending updates into active module directory.
pub fn handle_updated_modules() -> Result<()> {
    let modules_root = Path::new(MODULE_DIR);
    foreach_module(ModuleType::Updated, |updated_module| {
        if !updated_module.is_dir() {
            return Ok(());
        }

        if let Some(name) = updated_module.file_name() {
            let module_dir = modules_root.join(name);
            let Some(module_id) = name.to_str() else {
                warn!(
                    "Updated module directory has non-utf8 name, deleting staged dir: {}",
                    updated_module.display()
                );
                cleanup_staged_dir(updated_module)?;
                return Ok(());
            };

            if validate_module_id(module_id).is_err() {
                warn!(
                    "Updated module id is invalid, deleting staged dir: {}",
                    updated_module.display()
                );
                cleanup_staged_dir(updated_module)?;
                return Ok(());
            }

            if !should_promote_staged_module(updated_module) {
                warn!(
                    "Skipping staged module without success marker: {}",
                    updated_module.display()
                );
                cleanup_staged_dir(updated_module)?;
                return Ok(());
            }

            let preserved_flags = PreservedFlags {
                disabled: module_dir.join(defs::DISABLE_FILE_NAME).exists(),
                removed: module_dir.join(defs::REMOVE_FILE_NAME).exists(),
            };

            let backup = promote_staged_module(updated_module, &module_dir)?;
            if let Err(e) = finalize_successful_promotion(
                &module_dir,
                backup,
                preserved_flags,
                defs::DISABLE_FILE_NAME,
                defs::REMOVE_FILE_NAME,
            ) {
                /* ensure the broken module is disabled to prevent bootloops */
                let disable_marker = module_dir.join(defs::DISABLE_FILE_NAME);
                if let Err(disable_err) = ensure_file_exists(&disable_marker) {
                    warn!(
                        "Failed to place fail-closed disable marker after promotion finalization error for {}: {disable_err:#}",
                        module_dir.display()
                    );
                }
                warn!(
                    "Promotion completed but post-promotion cleanup/flags failed for {}: {e:#}",
                    module_dir.display()
                );
                return Err(anyhow!(
                    "Fail-closed promotion finalization error for {}: {e:#}",
                    module_dir.display()
                ));
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn install_module_to_system(zip: &str) -> Result<()> {
    ensure_boot_completed()?;

    /* Pre-extraction validation — prevents path traversal and malicious overlay structure */
    let report = module_validator::validate_module_zip(zip)?;
    for issue in &report.issues {
        if issue.severity == module_validator::IssueSeverity::Warning {
            warn!("Module validation warning: {}", issue.message);
        }
    }
    if !report.is_valid() {
        let errors: Vec<&str> = report
            .issues
            .iter()
            .filter(|i| i.severity == module_validator::IssueSeverity::Error)
            .map(|i| i.message.as_str())
            .collect();
        bail!("Module rejected: {}", errors.join("; "));
    }
    info!(
        "Module ZIP passed validation: {} ({} entries, {} bytes)",
        report.module_id.as_deref().unwrap_or("unknown"),
        report.entry_count,
        report.total_size
    );

    println!(include_str!("banner"));

    assets::ensure_binaries(false).with_context(|| "Failed to extract assets")?;

    ensure_dir_exists(defs::WORKING_DIR).with_context(|| "Failed to create working dir")?;
    ensure_dir_exists(defs::BINARY_DIR).with_context(|| "Failed to create bin dir")?;

    let mut buffer: Vec<u8> = Vec::new();
    let entry_path = PathBuf::from_str("module.prop")?;
    let zip_path = PathBuf::from_str(zip)?;
    let zip_path = zip_path.canonicalize()?;
    zip_extract_file_to_memory(&zip_path, &entry_path, &mut buffer)?;

    let mut module_prop = HashMap::new();
    PropertiesIter::new_with_encoding(Cursor::new(buffer), encoding_rs::UTF_8).read_into(
        |k, v| {
            module_prop.insert(k, v);
        },
    )?;
    info!("module prop: {module_prop:?}");

    let Some(module_id) = module_prop.get("id") else {
        bail!("module id not found in module.prop!");
    };
    let module_id = module_id.trim();

    validate_module_id(module_id)
        .with_context(|| format!("Invalid module ID in module.prop: '{module_id}'"))?;

    let is_metamodule = metamodule::is_metamodule(&module_prop);

    if !is_metamodule && let Err(is_disabled) = metamodule::check_install_safety() {
        println!("\n❌ Installation Blocked");
        println!("┌────────────────────────────────");
        println!("│ A metamodule with custom installer is active");
        println!("│");
        if is_disabled {
            println!("│ Current state: Disabled");
            println!("│ Action required: Re-enable or uninstall it, then reboot");
        } else {
            println!("│ Current state: Pending changes");
            println!("│ Action required: Reboot to apply changes first");
        }
        println!("└─────────────────────────────────\n");
        bail!("Metamodule installation blocked");
    }

    let updated_dir = Path::new(defs::MODULE_UPDATE_DIR).join(module_id);

    if is_metamodule {
        info!("Installing metamodule: {module_id}");

        if metamodule::has_metamodule()
            && let Some(existing_path) = metamodule::get_metamodule_path()
        {
            let existing_id = read_module_prop(&existing_path)
                .ok()
                .and_then(|m| m.get("id").cloned())
                .unwrap_or_else(|| "unknown".to_string());

            if existing_id != module_id {
                println!("\n❌ Installation Failed");
                println!("┌────────────────────────────────");
                println!("│ A metamodule is already installed");
                println!("│   Current metamodule: {existing_id}");
                println!("│");
                println!("│ Only one metamodule can be active at a time.");
                println!("└─────────────────────────────────\n");
                bail!("Cannot install multiple metamodules");
            }
        }
    }

    let zip_uncompressed_size = get_zip_uncompressed_size(zip)?;
    println!(
        "- Module size: {}",
        humansize::format_size(zip_uncompressed_size, humansize::DECIMAL)
    );

    ensure_dir_exists(defs::MODULE_UPDATE_DIR)?;
    setsyscon(defs::MODULE_UPDATE_DIR)?;

    /* Transactional install — uses promote_staged_module for atomicity */
    run_staging_transaction(&updated_dir, || {
        println!("- Installing to {}", updated_dir.display());
        ensure_clean_dir(&updated_dir)?;
        info!("target dir: {}", updated_dir.display());

        println!("- Extracting module files");
        let file = File::open(zip)?;
        let mut archive = zip::ZipArchive::new(file)?;
        extract_zip_secure(&mut archive, &updated_dir)?;

        let module_system_dir = updated_dir.join("system");
        if module_system_dir.exists() {
            #[cfg(unix)]
            set_permissions(&module_system_dir, Permissions::from_mode(0o755))?;
            restore_syscon(&module_system_dir)?;
        }

        println!("- Running module installer");
        exec_install_script(zip, is_metamodule)?;

        let module_dir = Path::new(MODULE_DIR).join(module_id);
        ensure_dir_exists(&module_dir)?;
        copy(
            updated_dir.join("module.prop"),
            module_dir.join("module.prop"),
        )?;
        ensure_file_exists(module_dir.join(UPDATE_FILE_NAME))?;

        if is_metamodule {
            println!("- Creating metamodule symlink");
            metamodule::ensure_symlink(&module_dir)?;
        }

        mark_install_complete(&updated_dir)?;
        Ok(())
    })?;

    println!("- Module installed successfully!");
    info!("Module {module_id} installed successfully!");

    Ok(())
}

fn secure_entry_destination(base: &Path, raw_name: &str) -> Result<PathBuf> {
    let entry_path = Path::new(raw_name);
    ensure!(
        !entry_path.is_absolute(),
        "ZIP entry uses absolute path: {raw_name}"
    );

    let mut rel = PathBuf::new();
    for comp in entry_path.components() {
        match comp {
            std::path::Component::Normal(segment) => rel.push(segment),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                bail!("ZIP entry contains parent traversal: {raw_name}")
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                bail!("ZIP entry contains unsupported path prefix: {raw_name}")
            }
        }
    }
    ensure!(!rel.as_os_str().is_empty(), "ZIP entry path is empty");

    let base_real = base
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize base: {}", base.display()))?;
    let dest = base_real.join(&rel);
    ensure!(
        dest.starts_with(&base_real),
        "ZIP entry escaped module directory: {raw_name}"
    );
    Ok(dest)
}

fn extract_zip_secure<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    target: &Path,
) -> Result<()> {
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if let Some(mode) = entry.unix_mode() {
            let file_type = mode & 0o170000;
            /* refuse nodes that could bypass system boundaries */
            ensure!(file_type != 0o120000, "Symlink entry rejected: {name}");
            ensure!(file_type != 0o060000, "Block device entry rejected: {name}");
            ensure!(
                file_type != 0o020000,
                "Character device entry rejected: {name}"
            );
        }

        let destination = secure_entry_destination(target, &name)?;

        if entry.is_dir() || name.ends_with('/') {
            std::fs::create_dir_all(&destination)?;
            continue;
        }

        let Some(parent) = destination.parent() else {
            bail!("Destination path missing parent: {}", destination.display());
        };
        std::fs::create_dir_all(parent)?;

        let mut out = File::create(&destination)
            .with_context(|| format!("Failed to create {}", destination.display()))?;
        let mut buf = [0_u8; 8192];
        loop {
            let n = entry.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;
        }
        out.flush()?;
    }
    Ok(())
}

/// Public API for installing a module from a ZIP.
pub fn install_module(zip: &str) -> Result<()> {
    let result = install_module_to_system(zip);
    if let Err(ref e) = result {
        println!("- Error: {e}");
    }
    result
}

/// Undoes a removal mark.
pub fn undo_uninstall_module(id: &str) -> Result<()> {
    validate_module_id(id)?;

    let module_path = Path::new(defs::MODULE_DIR).join(id);
    ensure!(module_path.exists(), "Module {id} not found");

    let remove_file = module_path.join(defs::REMOVE_FILE_NAME);
    if remove_file.exists() {
        std::fs::remove_file(&remove_file)
            .with_context(|| format!("Failed to delete remove file for module '{id}'"))?;
        info!("Removed the remove mark for module {id}");
    }

    Ok(())
}

/// Marks a module for deletion on next boot.
pub fn uninstall_module(id: &str) -> Result<()> {
    validate_module_id(id)?;

    let module_path = Path::new(defs::MODULE_DIR).join(id);
    ensure!(module_path.exists(), "Module {id} not found");

    let remove_file = module_path.join(defs::REMOVE_FILE_NAME);
    File::create(remove_file).with_context(|| "Failed to create remove file")?;

    info!("Module {id} marked for removal");

    Ok(())
}

/// Executes module's action script (if present).
pub fn run_action(id: &str) -> Result<()> {
    validate_module_id(id)?;

    let action_script_path = format!("/data/adb/modules/{id}/action.sh");
    exec_script(&action_script_path, true, 0)
}

/// Enables a previously disabled module.
pub fn enable_module(id: &str) -> Result<()> {
    validate_module_id(id)?;

    let module_path = Path::new(defs::MODULE_DIR).join(id);
    ensure!(module_path.exists(), "Module {id} not found");

    let disable_path = module_path.join(defs::DISABLE_FILE_NAME);
    if disable_path.exists() {
        std::fs::remove_file(&disable_path).with_context(|| {
            format!("Failed to remove disable file: {}", disable_path.display())
        })?;
        info!("Module {id} enabled");
    }

    Ok(())
}

/// Disables a module immediately.
pub fn disable_module(id: &str) -> Result<()> {
    let module_path = Path::new(defs::MODULE_DIR).join(id);
    ensure!(module_path.exists(), "Module {id} not found");

    let disable_path = module_path.join(defs::DISABLE_FILE_NAME);
    ensure_file_exists(disable_path)?;

    info!("Module {id} disabled");

    Ok(())
}

/// Global disable toggle.
pub fn disable_all_modules() -> Result<()> {
    mark_all_modules(defs::DISABLE_FILE_NAME)
}

/// Global removal toggle.
pub fn uninstall_all_modules() -> Result<()> {
    info!("Uninstalling all modules");
    mark_all_modules(defs::REMOVE_FILE_NAME)
}

fn mark_all_modules(flag_file: &str) -> Result<()> {
    let dir = std::fs::read_dir(defs::MODULE_DIR)?;
    for entry in dir.flatten() {
        let path = entry.path();
        let flag = path.join(flag_file);
        if let Err(e) = ensure_file_exists(flag) {
            warn!("Failed to mark module: {}: {e}", path.display());
        }
    }

    Ok(())
}

/// Reads module properties from disk.
pub fn read_module_prop(module_path: &Path) -> Result<HashMap<String, String>> {
    let module_prop = module_path.join("module.prop");
    ensure!(
        module_prop.exists(),
        "module.prop not found in {}",
        module_path.display()
    );

    let content = std::fs::read(&module_prop)
        .with_context(|| format!("Failed to read module.prop: {}", module_prop.display()))?;

    let mut prop_map: HashMap<String, String> = HashMap::new();
    PropertiesIter::new_with_encoding(Cursor::new(content), encoding_rs::UTF_8)
        .read_into(|k, v| {
            prop_map.insert(k, v);
        })
        .with_context(|| format!("Failed to parse module.prop: {}", module_prop.display()))?;

    Ok(prop_map)
}

fn resolve_module_icon_path(
    module_prop_map: &mut HashMap<String, String>,
    key: &str,
    module_path: &Path,
) {
    if let Some(icon_value) = module_prop_map.get(key) {
        let icon_value = icon_value.trim();
        if icon_value.is_empty() {
            return;
        }
        let path = std::path::Path::new(icon_value);
        if path.is_absolute() {
            log::warn!(
                "Rejected {} with absolute path for module {}: {}",
                key,
                module_prop_map.get("id").map_or("", String::as_str),
                icon_value
            );
            return;
        }
        let has_parent = path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        if has_parent {
            log::warn!(
                "Rejected {} with parent traversal for module {}: {}",
                key,
                module_prop_map.get("id").map_or("", String::as_str),
                icon_value
            );
            return;
        }
        let candidate = module_path.join(path);
        if candidate.exists() && candidate.is_file() {
            if let Some(s) = candidate.to_str() {
                module_prop_map.insert(key.to_owned(), s.to_string());
            }
        } else {
            log::debug!(
                "{} not found for module {}: {}",
                key,
                module_prop_map.get("id").map_or("", String::as_str),
                candidate.display()
            );
        }
    }
}

fn list_module(path: &str) -> Vec<HashMap<String, String>> {
    let all_configs = match crate::module_config::get_all_module_configs() {
        Ok(configs) => configs,
        Err(e) => {
            warn!("Failed to load module configs: {e}");
            HashMap::new()
        }
    };

    let dir = std::fs::read_dir(path);
    let Ok(dir) = dir else {
        return Vec::new();
    };

    let mut modules: Vec<HashMap<String, String>> = Vec::new();

    for entry in dir.flatten() {
        let path = entry.path();
        info!("path: {}", path.display());

        if !path.join("module.prop").exists() {
            continue;
        }

        let mut module_prop_map = match read_module_prop(&path) {
            Ok(prop) => prop,
            Err(e) => {
                warn!("Failed to read module.prop for {}: {e}", path.display());
                continue;
            }
        };

        if !module_prop_map.contains_key("id") || module_prop_map["id"].is_empty() {
            if let Some(id) = entry.file_name().to_str() {
                info!("Use dir name as module id: {id}");
                module_prop_map.insert("id".to_owned(), id.to_owned());
            } else {
                info!("Failed to get module id from dir name");
                continue;
            }
        }

        let enabled = !path.join(defs::DISABLE_FILE_NAME).exists();
        let update = path.join(defs::UPDATE_FILE_NAME).exists();
        let remove = path.join(defs::REMOVE_FILE_NAME).exists();
        let web = path.join(defs::MODULE_WEB_DIR).exists();
        let action = path.join(defs::MODULE_ACTION_SH).exists();
        let need_mount = path.join("system").exists() && !path.join("skip_mount").exists();

        module_prop_map.insert("enabled".to_owned(), enabled.to_string());
        module_prop_map.insert("update".to_owned(), update.to_string());
        module_prop_map.insert("remove".to_owned(), remove.to_string());
        module_prop_map.insert("web".to_owned(), web.to_string());
        module_prop_map.insert("action".to_owned(), action.to_string());
        module_prop_map.insert("mount".to_owned(), need_mount.to_string());

        resolve_module_icon_path(&mut module_prop_map, "actionIcon", &path);
        resolve_module_icon_path(&mut module_prop_map, "webuiIcon", &path);

        if let Some(module_id) = module_prop_map.get("id")
            && let Some(config) = all_configs.get(module_id.as_str())
        {
            if let Some(desc) = config.get("override.description") {
                module_prop_map.insert("description".to_owned(), desc.clone());
            }

            let managed_features: Vec<String> = config
                .iter()
                .filter_map(|(k, v)| {
                    if k.starts_with("manage.") && crate::module_config::parse_bool_config(v) {
                        k.strip_prefix("manage.")
                            .map(std::string::ToString::to_string)
                    } else {
                        None
                    }
                })
                .collect();

            if !managed_features.is_empty() {
                module_prop_map.insert("managedFeatures".to_owned(), managed_features.join(","));
            }
        }

        modules.push(module_prop_map);
    }

    modules
}

/// Lists all modules in JSON format for the manager app.
pub fn list_modules() -> Result<()> {
    let modules = list_module(defs::MODULE_DIR);
    println!("{}", serde_json::to_string_pretty(&modules)?);
    Ok(())
}

/// Retrieves list of features managed by active modules.
pub fn get_managed_features() -> Result<HashMap<String, Vec<String>>> {
    let mut managed_features_map: HashMap<String, Vec<String>> = HashMap::new();

    foreach_active_module(|module_path| {
        let Some(module_id) = module_path.file_name().and_then(|n| n.to_str()) else {
            warn!(
                "Failed to get module id from path: {}",
                module_path.display()
            );
            return Ok(());
        };

        let config = match crate::module_config::merge_configs(module_id) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to merge configs for module '{module_id}': {e}");
                return Ok(());
            }
        };

        let mut feature_list = Vec::new();
        for (key, value) in &config {
            if key.starts_with("manage.") {
                if let Some(feature_name) = key.strip_prefix("manage.")
                    && crate::module_config::parse_bool_config(value)
                {
                    feature_list.push(feature_name.to_string());
                }
            }
        }

        if !feature_list.is_empty() {
            managed_features_map.insert(module_id.to_string(), feature_list);
        }

        Ok(())
    })?;

    Ok(managed_features_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use zip::write::FileOptions;

    #[test]
    fn valid_module_ids() {
        assert!(validate_module_id("my.module").is_ok());
        assert!(validate_module_id("com.example.mod_v2").is_ok());
        assert!(validate_module_id("ab").is_ok());
        assert!(validate_module_id("A-Module-Name").is_ok());
        assert!(validate_module_id("test123").is_ok());
    }

    #[test]
    fn invalid_module_ids() {
        assert!(validate_module_id("a").is_err());
        assert!(validate_module_id("1abc").is_err());
        assert!(validate_module_id("my module").is_err());
        assert!(validate_module_id("").is_err());
        assert!(validate_module_id("mod@name").is_err());
        assert!(validate_module_id("mod/name").is_err());
    }

    #[test]
    fn read_module_prop_missing_file() {
        let tmp = std::env::temp_dir().join("nonexistent_module");
        assert!(read_module_prop(&tmp).is_err());
    }

    #[test]
    fn read_module_prop_valid() {
        let tmp = std::env::temp_dir().join("test_module_prop");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("module.prop"),
            "id=test_module\nname=Test Module\nversion=1.0\n",
        )
        .unwrap();

        let props = read_module_prop(&tmp).unwrap();
        assert_eq!(props.get("id").unwrap(), "test_module");
        assert_eq!(props.get("name").unwrap(), "Test Module");
        assert_eq!(props.get("version").unwrap(), "1.0");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn write_zip(path: &Path, name: &str, content: &[u8], unix_mode: Option<u32>) {
        let file = File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let mut options: FileOptions<'_, ()> = FileOptions::default();
        if let Some(mode) = unix_mode {
            options = options.unix_permissions(mode);
        }
        writer.start_file(name, options).unwrap();
        writer.write_all(content).unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn secure_zip_rejects_parent_traversal_entry() {
        let root = std::env::temp_dir().join(format!("apexsu_zip_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let zip_path = root.join("bad.zip");
        write_zip(&zip_path, "../escape.txt", b"bad", None);
        let target = root.join("out");
        std::fs::create_dir_all(&target).unwrap();

        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let err = extract_zip_secure(&mut archive, &target).unwrap_err();
        assert!(err.to_string().contains("parent traversal"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn secure_zip_rejects_symlink_entry() {
        let root =
            std::env::temp_dir().join(format!("apexsu_zip_test_symlink_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let zip_path = root.join("bad_symlink.zip");
        write_zip(&zip_path, "system/link", b"/system/bin/sh", Some(0o120777));
        let target = root.join("out");
        std::fs::create_dir_all(&target).unwrap();

        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let err = extract_zip_secure(&mut archive, &target).unwrap_err();
        assert!(err.to_string().contains("Symlink entry rejected"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
