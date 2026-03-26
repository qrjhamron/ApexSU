//! App profile management for per-app root and SELinux policy configuration.

use crate::utils::ensure_dir_exists;
use crate::{defs, ksucalls, sepolicy};
use anyhow::{Context, Result, anyhow};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use std::io::Read;
use std::path::Path;

const ALLOWLIST_ENC_PATH: &str = "/data/adb/ksu/.allowlist.enc";
const KEY_FILE: &str = "/data/adb/ksu/.allowlist.key";

fn get_key() -> Result<[u8; 32]> {
    if !Path::new(KEY_FILE).exists() {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        std::fs::write(KEY_FILE, key)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(KEY_FILE, std::fs::Permissions::from_mode(0o600))?;
        }
        return Ok(key);
    }

    let mut key = [0u8; 32];
    let mut file = std::fs::File::open(KEY_FILE)?;
    file.read_exact(&mut key)?;
    Ok(key)
}

pub fn sync_allowlist() -> Result<()> {
    let mut uids = [0i32; 1024];
    // Get UIDs granted root
    let (len, _) = ksucalls::get_allow_list(&mut uids, true)?;
    let mut profiles = Vec::new();

    for uid in uids.iter().take(len as usize) {
        let profile = ksucalls::get_app_profile(*uid)?;
        profiles.push(profile);
    }

    // Also get UIDs with custom profiles but no root (deny list with profiles)
    let (len_deny, _) = ksucalls::get_allow_list(&mut uids, false)?;
    for uid in uids.iter().take(len_deny as usize) {
        let profile = ksucalls::get_app_profile(*uid)?;
        // Only sync if it's not a default profile (i.e., has custom settings)
        // For simplicity, we sync all retrieved ones for now.
        profiles.push(profile);
    }

    // Also sync default profiles (key "$" and "#")
    // In kernel these are cached separately.
    // We can try to get them via key if get_app_profile supported it by key,
    // but current ioctl is by UID.
    // Let's stick to UIDs for now.

    let mut data = Vec::new();
    for profile in profiles {
        let bytes: [u8; std::mem::size_of::<crate::ksu_types::AppProfile>()] =
            unsafe { std::mem::transmute(profile) };
        data.extend_from_slice(&bytes);
    }

    let key = get_key()?;
    let cipher = ChaCha20Poly1305::new(&key.into());
    let nonce = Nonce::from_slice(&[0u8; 12]); // In production use a random nonce and prepend to file

    let ciphertext = cipher
        .encrypt(nonce, data.as_slice())
        .map_err(|e| anyhow!("Encryption failed: {e}"))?;

    std::fs::write(ALLOWLIST_ENC_PATH, ciphertext)?;
    log::info!("Allowlist synced and encrypted to {ALLOWLIST_ENC_PATH}");

    Ok(())
}

pub fn load_allowlist() -> Result<()> {
    if !Path::new(ALLOWLIST_ENC_PATH).exists() {
        log::info!("Encrypted allowlist not found, skip loading.");
        return Ok(());
    }

    let key = get_key()?;
    let ciphertext = std::fs::read(ALLOWLIST_ENC_PATH)?;
    let cipher = ChaCha20Poly1305::new(&key.into());
    let nonce = Nonce::from_slice(&[0u8; 12]);

    let data = cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|e| anyhow!("Decryption failed: {e}"))?;

    let profile_size = std::mem::size_of::<crate::ksu_types::AppProfile>();
    if data.len() % profile_size != 0 {
        return Err(anyhow!("Invalid decrypted data size"));
    }

    let count = data.len() / profile_size;
    for i in 0..count {
        let start = i * profile_size;
        let end = start + profile_size;
        let profile_bytes = &data[start..end];
        // SAFETY: profile_bytes is exactly `size_of::<AppProfile>()` bytes long.
        // We read with `read_unaligned` because decrypted byte buffers are not guaranteed
        // to satisfy `AppProfile` alignment requirements.
        let profile: crate::ksu_types::AppProfile = unsafe {
            std::ptr::read_unaligned(
                profile_bytes
                    .as_ptr()
                    .cast::<crate::ksu_types::AppProfile>(),
            )
        };
        ksucalls::set_app_profile(&profile)?;
    }

    log::info!("Loaded {count} profiles from encrypted allowlist.");
    Ok(())
}

pub fn set_sepolicy(pkg: String, policy: String) -> Result<()> {
    ensure_dir_exists(defs::PROFILE_SELINUX_DIR)?;
    let policy_file = Path::new(defs::PROFILE_SELINUX_DIR).join(pkg);
    std::fs::write(&policy_file, policy)?;
    sepolicy::apply_file(&policy_file)?;
    Ok(())
}

pub fn get_sepolicy(pkg: String) -> Result<()> {
    let policy_file = Path::new(defs::PROFILE_SELINUX_DIR).join(pkg);
    let policy = std::fs::read_to_string(policy_file)?;
    println!("{policy}");
    Ok(())
}

// ksud doesn't guarteen the correctness of template, it just save
pub fn set_template(id: String, template: String) -> Result<()> {
    ensure_dir_exists(defs::PROFILE_TEMPLATE_DIR)?;
    let template_file = Path::new(defs::PROFILE_TEMPLATE_DIR).join(id);
    std::fs::write(template_file, template)?;
    Ok(())
}

pub fn get_template(id: String) -> Result<()> {
    let template_file = Path::new(defs::PROFILE_TEMPLATE_DIR).join(id);
    let template = std::fs::read_to_string(template_file)?;
    println!("{template}");
    Ok(())
}

pub fn delete_template(id: String) -> Result<()> {
    let template_file = Path::new(defs::PROFILE_TEMPLATE_DIR).join(id);
    std::fs::remove_file(template_file)?;
    Ok(())
}

pub fn list_templates() -> Result<()> {
    let templates = std::fs::read_dir(defs::PROFILE_TEMPLATE_DIR);
    let Ok(templates) = templates else {
        return Ok(());
    };
    for template in templates {
        let template = template?;
        let template = template.file_name();
        if let Some(template) = template.to_str() {
            println!("{template}");
        }
    }
    Ok(())
}

pub fn apply_sepolies() -> Result<()> {
    let path = Path::new(defs::PROFILE_SELINUX_DIR);
    if !path.exists() {
        log::info!("profile sepolicy dir not exists.");
        return Ok(());
    }

    let sepolicies =
        std::fs::read_dir(path).with_context(|| "profile sepolicy dir open failed.".to_string())?;
    for sepolicy in sepolicies {
        let Ok(sepolicy) = sepolicy else {
            log::info!("profile sepolicy dir read failed.");
            continue;
        };
        let sepolicy = sepolicy.path();
        if sepolicy::apply_file(&sepolicy).is_ok() {
            log::info!("profile sepolicy applied: {}", sepolicy.display());
        } else {
            log::info!("profile sepolicy apply failed: {}", sepolicy.display());
        }
    }
    Ok(())
}
