//! Entry point for the ksud daemon, the KernelSU userspace component.

#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::cast_possible_wrap
)]

mod apk_sign;
mod assets;
mod boot_patch;
#[cfg(target_os = "android")]
mod cli;
#[cfg(not(target_os = "android"))]
mod cli_non_android;
#[cfg(target_os = "android")]
mod debug;
mod defs;
#[cfg(target_os = "android")]
mod diagnostics;
#[cfg(target_os = "android")]
mod feature;
#[cfg(target_os = "android")]
mod init_event;
mod ksu_types;
#[cfg(target_os = "android")]
mod ksucalls;
#[cfg(target_os = "android")]
mod metamodule;
#[cfg(target_os = "android")]
mod module;
#[cfg(target_os = "android")]
mod module_config;
mod module_lifecycle;
mod module_validator;
#[cfg(target_os = "android")]
mod profile;
#[cfg(target_os = "android")]
mod restorecon;
#[cfg(any(target_os = "android", test))]
mod sepolicy;
#[cfg(target_os = "android")]
mod su;
#[cfg(target_os = "android")]
mod utils;

#[cfg(all(not(target_os = "android"), test))]
mod module {
    mod tests {
        use anyhow::anyhow;
        use std::fs;
        use std::path::{Path, PathBuf};

        use crate::module_lifecycle::{
            PreservedFlags, cleanup_staged_dir, finalize_successful_promotion, is_internal_module_dir,
            is_valid_active_module_id, mark_install_complete, promote_staged_module,
            promote_staged_module_for_test, run_staging_transaction, should_promote_staged_module,
        };

        fn create_dir(path: &Path) {
            fs::create_dir_all(path).unwrap();
        }

        fn collect_active_modules(root: &Path) -> Vec<PathBuf> {
            let mut modules = Vec::new();
            for entry in fs::read_dir(root).unwrap().flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(name) = entry.file_name().to_str().map(ToString::to_string) else {
                    continue;
                };
                if !is_valid_active_module_id(&name) {
                    continue;
                }
                if path.join("disable").exists() || path.join("remove").exists() {
                    continue;
                }
                modules.push(path);
            }
            modules
        }

        #[test]
        fn failed_installer_after_extraction_removes_staged_dir() {
            let root = tempfile::tempdir().unwrap();
            let staged = root.path().join("staged");
            create_dir(&staged);
            fs::write(staged.join("module.prop"), b"id=test\n").unwrap();

            let res: anyhow::Result<()> =
                run_staging_transaction(&staged, || Err(anyhow!("installer failed")));
            assert!(res.is_err());
            assert!(!staged.exists(), "staged directory must be cleaned on failure");
        }

        #[test]
        fn boot_skips_staged_dir_without_success_marker() {
            let root = tempfile::tempdir().unwrap();
            let staged = root.path().join("staged");
            create_dir(&staged);
            fs::write(staged.join("module.prop"), b"id=test\n").unwrap();

            assert!(!should_promote_staged_module(&staged));
            cleanup_staged_dir(&staged).unwrap();
            assert!(!staged.exists());
        }

        #[test]
        fn successful_install_marker_allows_promotion() {
            let root = tempfile::tempdir().unwrap();
            let staged = root.path().join("staged_mod");
            let active = root.path().join("active_mod");
            create_dir(&staged);
            fs::write(staged.join("module.prop"), b"id=test\n").unwrap();
            mark_install_complete(&staged).unwrap();

            assert!(should_promote_staged_module(&staged));
            let backup = promote_staged_module(&staged, &active).unwrap();
            assert!(backup.is_none());
            assert!(active.exists());
            assert!(!staged.exists());
        }

        #[test]
        fn update_rename_failure_restores_old_active_module() {
            let root = tempfile::tempdir().unwrap();
            let active = root.path().join("mod");
            let staged = root.path().join("staged_mod");
            create_dir(&active);
            create_dir(&staged);
            fs::write(active.join("old.txt"), b"old").unwrap();
            fs::write(staged.join("new.txt"), b"new").unwrap();
            mark_install_complete(&staged).unwrap();

            let mut call_count = 0usize;
            let mut renamer = |from: &Path, to: &Path| {
                call_count += 1;
                if call_count == 2 {
                    return Err(std::io::Error::other("forced rename failure"));
                }
                fs::rename(from, to)
            };

            let err = promote_staged_module_for_test(&staged, &active, &mut renamer).unwrap_err();
            let err_msg = format!("{err:#}");
            assert!(err_msg.contains("rolled back active module"));
            assert!(active.exists(), "active module must be restored");
            assert!(active.join("old.txt").exists(), "restored active content missing");
            assert!(staged.exists(), "staged module should remain for investigation");
        }

        #[test]
        fn staged_module_without_marker_never_reaches_active_path() {
            let root = tempfile::tempdir().unwrap();
            let staged = root.path().join("staged_mod");
            let active = root.path().join("mod");
            create_dir(&staged);
            fs::write(staged.join("post-fs-data.sh"), b"echo stage").unwrap();

            assert!(!should_promote_staged_module(&staged));
            cleanup_staged_dir(&staged).unwrap();
            assert!(!active.exists());
            assert!(!staged.exists());
        }

        #[test]
        fn partial_staging_cleanup_does_not_remove_active_module() {
            let root = tempfile::tempdir().unwrap();
            let active = root.path().join("mod");
            let staged = root.path().join("staged_mod");
            create_dir(&active);
            create_dir(&staged);
            fs::write(active.join("keep.txt"), b"keep").unwrap();
            fs::write(staged.join("partial.bin"), b"partial").unwrap();

            assert!(!should_promote_staged_module(&staged));
            cleanup_staged_dir(&staged).unwrap();
            assert!(active.exists());
            assert!(active.join("keep.txt").exists());
        }

        #[test]
        fn disable_remove_flags_preserved_after_successful_update() {
            let root = tempfile::tempdir().unwrap();
            let active = root.path().join("mod");
            let staged = root.path().join("staged_mod");
            create_dir(&active);
            create_dir(&staged);
            fs::write(active.join("disable"), b"").unwrap();
            fs::write(active.join("remove"), b"").unwrap();
            fs::write(staged.join("module.prop"), b"id=test\n").unwrap();
            mark_install_complete(&staged).unwrap();

            let preserved = PreservedFlags {
                disabled: true,
                removed: true,
            };

            let backup = promote_staged_module(&staged, &active).unwrap();
            finalize_successful_promotion(&active, backup, preserved, "disable", "remove").unwrap();

            assert!(active.join("remove").exists());
            assert!(
                !active.join("disable").exists(),
                "remove marker should take precedence over disable"
            );
        }

        #[test]
        fn backup_prefixed_dir_under_modules_root_is_skipped_by_active_iteration() {
            let root = tempfile::tempdir().unwrap();
            let backup = root.path().join(".apexsu-backup-my.mod-1-2");
            let valid = root.path().join("com.example.module");
            create_dir(&backup);
            create_dir(&valid);

            let active = collect_active_modules(root.path());
            assert_eq!(active, vec![valid]);
        }

        #[test]
        fn backup_prefixed_dir_never_reaches_exec_stage_script() {
            let root = tempfile::tempdir().unwrap();
            let backup = root.path().join(".apexsu-backup-test-1");
            let valid = root.path().join("com.example.module");
            create_dir(&backup);
            create_dir(&valid);
            fs::write(backup.join("post-fs-data.sh"), b"echo backup").unwrap();
            fs::write(valid.join("post-fs-data.sh"), b"echo valid").unwrap();

            let stage_scripts: Vec<_> = collect_active_modules(root.path())
                .into_iter()
                .map(|module| module.join("post-fs-data.sh"))
                .filter(|path| path.exists())
                .collect();
            assert_eq!(stage_scripts, vec![valid.join("post-fs-data.sh")]);
        }

        #[test]
        fn backup_prefixed_dir_never_reaches_load_system_prop() {
            let root = tempfile::tempdir().unwrap();
            let backup = root.path().join(".apexsu-backup-test-2");
            let valid = root.path().join("com.example.module");
            create_dir(&backup);
            create_dir(&valid);
            fs::write(backup.join("system.prop"), b"a=b\n").unwrap();
            fs::write(valid.join("system.prop"), b"c=d\n").unwrap();

            let props: Vec<_> = collect_active_modules(root.path())
                .into_iter()
                .map(|module| module.join("system.prop"))
                .filter(|path| path.exists())
                .collect();
            assert_eq!(props, vec![valid.join("system.prop")]);
        }

        #[test]
        fn backup_prefixed_dir_never_reaches_load_sepolicy_rule() {
            let root = tempfile::tempdir().unwrap();
            let backup = root.path().join(".apexsu-backup-test-3");
            let valid = root.path().join("com.example.module");
            create_dir(&backup);
            create_dir(&valid);
            fs::write(backup.join("sepolicy.rule"), b"allow a b:c d\n").unwrap();
            fs::write(valid.join("sepolicy.rule"), b"allow x y:z q\n").unwrap();

            let policies: Vec<_> = collect_active_modules(root.path())
                .into_iter()
                .map(|module| module.join("sepolicy.rule"))
                .filter(|path| path.exists())
                .collect();
            assert_eq!(policies, vec![valid.join("sepolicy.rule")]);
        }

        #[test]
        fn finalization_failure_after_promotion_does_not_cause_backup_dir_execution() {
            let root = tempfile::tempdir().unwrap();
            let active = root.path().join("com.example.module");
            let staged = root.path().join("staged_mod");
            create_dir(&active);
            create_dir(&staged);
            fs::write(active.join("old.txt"), b"old").unwrap();
            fs::write(staged.join("module.prop"), b"id=com.example.module\n").unwrap();
            mark_install_complete(&staged).unwrap();

            let backup = promote_staged_module(&staged, &active).unwrap();
            assert!(backup.as_ref().is_some_and(|b| b.exists()));
            let err = finalize_successful_promotion(
                &active,
                backup,
                PreservedFlags {
                    disabled: false,
                    removed: true,
                },
                "disable",
                "nested/remove",
            )
            .unwrap_err();
            assert!(format!("{err:#}").contains("Failed to preserve remove marker"));
            assert!(active.join("disable").exists(), "active must be disabled on failure");

            let stage_scripts: Vec<_> = collect_active_modules(root.path())
                .into_iter()
                .map(|module| module.join("post-fs-data.sh"))
                .filter(|path| path.exists())
                .collect();
            assert!(stage_scripts.is_empty());

            let backup_dirs: Vec<_> = fs::read_dir(root.path())
                .unwrap()
                .flatten()
                .filter_map(|entry| {
                    let name = entry.file_name();
                    let name = name.to_str()?;
                    if name.starts_with(".apexsu-backup-") {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(backup_dirs.len(), 1, "backup should remain non-executable");
        }

        #[test]
        fn valid_normal_module_still_runs() {
            let root = tempfile::tempdir().unwrap();
            let valid = root.path().join("com.example.valid");
            create_dir(&valid);
            fs::write(valid.join("service.sh"), b"echo ok").unwrap();

            let stage_scripts: Vec<_> = collect_active_modules(root.path())
                .into_iter()
                .map(|module| module.join("service.sh"))
                .filter(|path| path.exists())
                .collect();
            assert_eq!(stage_scripts, vec![valid.join("service.sh")]);
        }

        #[test]
        fn invalid_names_are_skipped() {
            assert!(is_internal_module_dir(".apexsu-backup-x"));
            assert!(is_internal_module_dir(".apexsu-tmp-x"));
            assert!(is_internal_module_dir(".hidden"));

            for name in [
                ".apexsu-backup-x",
                ".apexsu-tmp-x",
                ".hidden",
                "..",
                "a/b",
                "",
            ] {
                assert!(
                    !is_valid_active_module_id(name),
                    "name should be invalid: {name}"
                );
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "android")]
    {
        cli::run()
    }
    #[cfg(not(target_os = "android"))]
    {
        cli_non_android::run()
    }
}
