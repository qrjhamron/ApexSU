pub const KSU_MAX_PACKAGE_NAME: usize = 256;
pub const KSU_MAX_GROUPS: usize = 32;
pub const KSU_SELINUX_DOMAIN: usize = 64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RootProfile {
    pub uid: i32,
    pub gid: i32,
    pub groups_count: i32,
    pub groups: [i32; KSU_MAX_GROUPS],
    pub capabilities: Capabilities,
    pub selinux_domain: [u8; KSU_SELINUX_DOMAIN],
    pub namespaces: i32,
}

impl Default for RootProfile {
    fn default() -> Self {
        Self {
            uid: 0,
            gid: 0,
            groups_count: 0,
            groups: [0; KSU_MAX_GROUPS],
            capabilities: Capabilities::default(),
            selinux_domain: [0; KSU_SELINUX_DOMAIN],
            namespaces: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    pub effective: u64,
    pub permitted: u64,
    pub inheritable: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NonRootProfile {
    pub umount_modules: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AppProfile {
    pub version: u32,
    pub key: [u8; KSU_MAX_PACKAGE_NAME],
    pub current_uid: i32,
    pub allow_su: bool,
    pub config: AppProfileConfig,
}

fn fixed_cstr_len(field: &[u8], allow_empty: bool) -> Result<usize, &'static str> {
    let len = field
        .iter()
        .position(|&b| b == 0)
        .ok_or("missing NUL terminator")?;
    if !allow_empty && len == 0 {
        return Err("empty string");
    }
    Ok(len)
}

fn fixed_cstr_eq(field: &[u8], literal: &[u8]) -> bool {
    matches!(fixed_cstr_len(field, false), Ok(len) if len == literal.len())
        && field[..literal.len()] == *literal
}

fn validate_root_profile(profile: &RootProfile) -> Result<(), &'static str> {
    if profile.groups_count < 0 || profile.groups_count > KSU_MAX_GROUPS as i32 {
        return Err("invalid groups_count");
    }

    fixed_cstr_len(&profile.selinux_domain, false)?;
    Ok(())
}

impl AppProfile {
    pub fn validate_abi_strings(&self) -> Result<(), &'static str> {
        fixed_cstr_len(&self.key, false)?;

        if self.allow_su || fixed_cstr_eq(&self.key, b"#") {
            // SAFETY: `allow_su` selects the root-profile arm of the C union.
            // The special "#" key also stores the root-profile arm as the
            // default root profile, independent of allow_su.
            let rp_config = unsafe { &self.config.rp_config };
            fixed_cstr_len(&rp_config.template_name, true)?;
            validate_root_profile(&rp_config.profile)?;
        }

        Ok(())
    }
}

pub fn validate_ioctl_cstr_len(value: &str, max_with_nul: usize) -> Result<(), &'static str> {
    if value.is_empty() {
        return Err("string cannot be empty");
    }
    if value.as_bytes().len() >= max_with_nul {
        return Err("string too long for ioctl ABI");
    }
    Ok(())
}

impl Default for AppProfile {
    fn default() -> Self {
        Self {
            version: 0,
            key: [0; KSU_MAX_PACKAGE_NAME],
            current_uid: 0,
            allow_su: false,
            config: AppProfileConfig::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RpConfig {
    pub use_default: bool,
    pub template_name: [u8; KSU_MAX_PACKAGE_NAME],
    pub profile: RootProfile,
}

impl Default for RpConfig {
    fn default() -> Self {
        Self {
            use_default: true,
            template_name: [0; KSU_MAX_PACKAGE_NAME],
            profile: RootProfile::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NrpConfig {
    pub use_default: bool,
    pub profile: NonRootProfile,
}

impl Default for NrpConfig {
    fn default() -> Self {
        Self {
            use_default: true,
            profile: NonRootProfile::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union AppProfileConfig {
    pub rp_config: RpConfig,
    pub nrp_config: NrpConfig,
}

impl Default for AppProfileConfig {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl std::fmt::Debug for AppProfileConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppProfileConfig").finish()
    }
}

// Since union can't automatically derive Serialize/Deserialize,
// we'll treat AppProfile as a raw byte array for crypto purposes.

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn c_abi_layout_matches_kernel_app_profile_header() {
        assert_eq!(size_of::<Capabilities>(), 24);
        assert_eq!(offset_of!(Capabilities, effective), 0);
        assert_eq!(offset_of!(Capabilities, permitted), 8);
        assert_eq!(offset_of!(Capabilities, inheritable), 16);

        assert_eq!(size_of::<RootProfile>(), 240);
        assert_eq!(offset_of!(RootProfile, uid), 0);
        assert_eq!(offset_of!(RootProfile, gid), 4);
        assert_eq!(offset_of!(RootProfile, groups_count), 8);
        assert_eq!(offset_of!(RootProfile, groups), 12);
        assert_eq!(offset_of!(RootProfile, capabilities), 144);
        assert_eq!(offset_of!(RootProfile, selinux_domain), 168);
        assert_eq!(offset_of!(RootProfile, namespaces), 232);

        assert_eq!(size_of::<NonRootProfile>(), 1);
        assert_eq!(offset_of!(NonRootProfile, umount_modules), 0);

        assert_eq!(size_of::<RpConfig>(), 504);
        assert_eq!(offset_of!(RpConfig, use_default), 0);
        assert_eq!(offset_of!(RpConfig, template_name), 1);
        assert_eq!(offset_of!(RpConfig, profile), 264);

        assert_eq!(size_of::<NrpConfig>(), 2);
        assert_eq!(offset_of!(NrpConfig, use_default), 0);
        assert_eq!(offset_of!(NrpConfig, profile), 1);

        assert_eq!(size_of::<AppProfileConfig>(), 504);
        assert_eq!(size_of::<AppProfile>(), 776);
        assert_eq!(offset_of!(AppProfile, version), 0);
        assert_eq!(offset_of!(AppProfile, key), 4);
        assert_eq!(offset_of!(AppProfile, current_uid), 260);
        assert_eq!(offset_of!(AppProfile, allow_su), 264);
        assert_eq!(offset_of!(AppProfile, config), 272);
    }

    #[test]
    fn app_profile_rejects_full_non_nul_key() {
        let profile = AppProfile {
            key: [b'a'; KSU_MAX_PACKAGE_NAME],
            ..Default::default()
        };

        assert!(profile.validate_abi_strings().is_err());
    }

    #[test]
    fn app_profile_rejects_full_non_nul_selinux_domain() {
        let mut profile = AppProfile {
            allow_su: true,
            ..Default::default()
        };
        profile.key[0] = b'a';
        profile.config.rp_config.profile.selinux_domain = [b'a'; KSU_SELINUX_DOMAIN];

        assert!(profile.validate_abi_strings().is_err());
    }

    #[test]
    fn default_root_profile_rejects_full_non_nul_selinux_domain() {
        let mut profile = AppProfile::default();
        profile.key[0] = b'#';
        profile.config.rp_config.profile.selinux_domain = [b'a'; KSU_SELINUX_DOMAIN];

        assert!(profile.validate_abi_strings().is_err());
    }

    #[test]
    fn default_root_profile_accepts_valid_profile() {
        let mut profile = AppProfile::default();
        profile.key[0] = b'#';
        let domain = b"u:r:su_app:s0";
        unsafe {
            profile.config.rp_config.profile.selinux_domain[..domain.len()].copy_from_slice(domain);
        }

        assert!(profile.validate_abi_strings().is_ok());
    }

    #[test]
    fn app_profile_rejects_negative_groups_count() {
        let mut profile = AppProfile {
            allow_su: true,
            ..Default::default()
        };
        profile.key[0] = b'a';
        profile.config.rp_config.profile.groups_count = -1;
        let domain = b"u:r:su_app:s0";
        unsafe {
            profile.config.rp_config.profile.selinux_domain[..domain.len()].copy_from_slice(domain);
        }

        assert!(profile.validate_abi_strings().is_err());
    }

    #[test]
    fn app_profile_accepts_zero_groups_count() {
        let mut profile = AppProfile {
            allow_su: true,
            ..Default::default()
        };
        profile.key[0] = b'a';
        profile.config.rp_config.profile.groups_count = 0;
        let domain = b"u:r:su_app:s0";
        unsafe {
            profile.config.rp_config.profile.selinux_domain[..domain.len()].copy_from_slice(domain);
        }

        assert!(profile.validate_abi_strings().is_ok());
    }

    #[test]
    fn app_profile_accepts_boundary_groups_count() {
        let mut profile = AppProfile {
            allow_su: true,
            ..Default::default()
        };
        profile.key[0] = b'a';
        profile.config.rp_config.profile.groups_count = KSU_MAX_GROUPS as i32;
        let domain = b"u:r:su_app:s0";
        unsafe {
            profile.config.rp_config.profile.selinux_domain[..domain.len()].copy_from_slice(domain);
        }

        assert!(profile.validate_abi_strings().is_ok());
    }

    #[test]
    fn app_profile_rejects_groups_count_above_boundary() {
        let mut profile = AppProfile {
            allow_su: true,
            ..Default::default()
        };
        profile.key[0] = b'a';
        profile.config.rp_config.profile.groups_count = KSU_MAX_GROUPS as i32 + 1;
        let domain = b"u:r:su_app:s0";
        unsafe {
            profile.config.rp_config.profile.selinux_domain[..domain.len()].copy_from_slice(domain);
        }

        assert!(profile.validate_abi_strings().is_err());
    }

    #[test]
    fn root_profile_validator_mirrors_kernel_group_boundaries() {
        let mut root = RootProfile::default();
        root.selinux_domain[..b"u:r:su_app:s0".len()].copy_from_slice(b"u:r:su_app:s0");

        root.groups_count = -1;
        assert_eq!(validate_root_profile(&root), Err("invalid groups_count"));

        root.groups_count = 0;
        assert!(validate_root_profile(&root).is_ok());

        root.groups_count = KSU_MAX_GROUPS as i32;
        assert!(validate_root_profile(&root).is_ok());

        root.groups_count = KSU_MAX_GROUPS as i32 + 1;
        assert_eq!(validate_root_profile(&root), Err("invalid groups_count"));
    }

    #[test]
    fn app_profile_accepts_valid_strings() {
        let mut profile = AppProfile {
            allow_su: true,
            ..Default::default()
        };
        profile.key[..3].copy_from_slice(b"app");
        // SAFETY: `allow_su` selects the root-profile arm of the C union.
        unsafe {
            let domain = b"u:r:su_app:s0";
            profile.config.rp_config.profile.selinux_domain[..domain.len()].copy_from_slice(domain);
        }

        assert!(profile.validate_abi_strings().is_ok());
    }

    #[test]
    fn ioctl_cstr_len_rejects_exact_size_without_nul_room() {
        let value = "a".repeat(KSU_MAX_PACKAGE_NAME);
        assert!(validate_ioctl_cstr_len(&value, KSU_MAX_PACKAGE_NAME).is_err());
    }

    #[test]
    fn ioctl_cstr_len_accepts_max_size_with_nul_room() {
        let value = "a".repeat(KSU_MAX_PACKAGE_NAME - 1);
        assert!(validate_ioctl_cstr_len(&value, KSU_MAX_PACKAGE_NAME).is_ok());
    }
}
