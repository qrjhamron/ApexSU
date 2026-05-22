//! App profile management for per-app root and SELinux policy configuration.

use crate::utils::ensure_dir_exists;
use crate::{defs, ksucalls, sepolicy};
use anyhow::{Context, Result, anyhow};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use rand::RngCore;
use std::io::Read;
use std::path::{Path, PathBuf};

const ALLOWLIST_ENC_PATH: &str = "/data/adb/ksu/.allowlist.enc";
const KEY_FILE: &str = "/data/adb/ksu/.allowlist.key";
const ALLOWLIST_FORMAT_V1: u8 = 1;
const NONCE_SIZE: usize = 12;

fn get_key() -> Result<[u8; 32]> {
    if !Path::new(KEY_FILE).exists() {
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

fn encrypt_allowlist_payload(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(&(*key).into());
    let mut nonce_buf = [0_u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_buf);
    let nonce = Nonce::from_slice(&nonce_buf);
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow!("Encryption failed: {e}"))?;

    let mut out = Vec::with_capacity(1 + NONCE_SIZE + ciphertext.len());
    out.push(ALLOWLIST_FORMAT_V1);
    out.extend_from_slice(&nonce_buf);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt_allowlist_payload(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(&(*key).into());
    if data.len() > (1 + NONCE_SIZE) && data[0] == ALLOWLIST_FORMAT_V1 {
        let nonce = Nonce::from_slice(&data[1..1 + NONCE_SIZE]);
        return cipher
            .decrypt(nonce, &data[1 + NONCE_SIZE..])
            .map_err(|e| anyhow!("Decryption failed for v1 format: {e}"));
    }

    // Legacy compatibility path: previous format used a fixed zero nonce and
    // stored ciphertext only. Keep read support so existing installs migrate
    // automatically on next sync.
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    cipher
        .decrypt(nonce, data)
        .map_err(|e| anyhow!("Decryption failed for legacy format: {e}"))
}

fn validate_identifier(kind: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(anyhow!("{kind} cannot be empty"));
    }
    if value.chars().any(char::is_control) {
        return Err(anyhow!("{kind} contains control characters"));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(anyhow!("{kind} contains path separators"));
    }
    if value == "." || value == ".." || value.contains("..") {
        return Err(anyhow!("{kind} contains forbidden traversal sequence"));
    }
    if Path::new(value).is_absolute() {
        return Err(anyhow!("{kind} must not be absolute"));
    }
    Ok(())
}

fn confined_join(base: &str, kind: &str, value: &str) -> Result<PathBuf> {
    validate_identifier(kind, value)?;
    let base_path = Path::new(base);
    ensure_dir_exists(base)?;
    let canonical_base = std::fs::canonicalize(base_path)
        .with_context(|| format!("Failed to canonicalize base path: {base}"))?;
    let candidate = canonical_base.join(value);

    if candidate.parent() != Some(canonical_base.as_path()) {
        return Err(anyhow!("{kind} escaped base directory"));
    }
    Ok(candidate)
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
    let encrypted = encrypt_allowlist_payload(&key, data.as_slice())?;
    std::fs::write(ALLOWLIST_ENC_PATH, encrypted)?;
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
    let data = decrypt_allowlist_payload(&key, ciphertext.as_slice())?;

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
    let policy_file = confined_join(defs::PROFILE_SELINUX_DIR, "package", &pkg)?;
    std::fs::write(&policy_file, policy)?;
    sepolicy::apply_file(&policy_file)?;
    Ok(())
}

pub fn get_sepolicy(pkg: String) -> Result<()> {
    let policy_file = confined_join(defs::PROFILE_SELINUX_DIR, "package", &pkg)?;
    let policy = std::fs::read_to_string(policy_file)?;
    println!("{policy}");
    Ok(())
}

// ksud doesn't guarteen the correctness of template, it just save
pub fn set_template(id: String, template: String) -> Result<()> {
    let template_file = confined_join(defs::PROFILE_TEMPLATE_DIR, "template id", &id)?;
    std::fs::write(template_file, template)?;
    Ok(())
}

pub fn get_template(id: String) -> Result<()> {
    let template_file = confined_join(defs::PROFILE_TEMPLATE_DIR, "template id", &id)?;
    let template = std::fs::read_to_string(template_file)?;
    println!("{template}");
    Ok(())
}

pub fn delete_template(id: String) -> Result<()> {
    let template_file = confined_join(defs::PROFILE_TEMPLATE_DIR, "template id", &id)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_validation_rejects_traversal_and_separators() {
        for bad in ["../a", "..", "a/../../b", "/abs", "a\\b", "", "."] {
            assert!(validate_identifier("id", bad).is_err(), "{bad} should fail");
        }
        assert!(validate_identifier("id", "com.example.test").is_ok());
        assert!(validate_identifier("id", "alpha_1-2").is_ok());
    }

    #[test]
    fn allowlist_crypto_roundtrip_and_uniqueness() {
        let key = [7u8; 32];
        let plain = b"sample-profile-data";
        let enc1 = encrypt_allowlist_payload(&key, plain).unwrap();
        let enc2 = encrypt_allowlist_payload(&key, plain).unwrap();
        assert_ne!(enc1, enc2, "nonce must randomize ciphertext");
        let out = decrypt_allowlist_payload(&key, &enc1).unwrap();
        assert_eq!(out, plain);
    }

    #[test]
    fn allowlist_crypto_tamper_detected() {
        let key = [3u8; 32];
        let plain = b"sensitive";
        let mut enc = encrypt_allowlist_payload(&key, plain).unwrap();
        let last = enc.len() - 1;
        enc[last] ^= 0x80;
        assert!(decrypt_allowlist_payload(&key, &enc).is_err());
    }

    #[test]
    fn allowlist_legacy_ciphertext_still_decrypts() {
        let key = [9u8; 32];
        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        let legacy = cipher.encrypt(nonce, b"legacy-data".as_slice()).unwrap();
        let out = decrypt_allowlist_payload(&key, &legacy).unwrap();
        assert_eq!(out, b"legacy-data");
    }
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
