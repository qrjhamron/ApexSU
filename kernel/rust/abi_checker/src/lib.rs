//! Standalone Rust ABI checks for ApexSU kernel profile structs.
//!
//! This crate is deliberately not wired into kernel Kbuild. It provides
//! host-side Rust mirrors and tests for the C ABI in `kernel/app_profile.h`
//! and selected ioctl command payloads from `kernel/supercalls.h`.

pub const KSU_APP_PROFILE_VER: u32 = 2;
pub const KSU_MAX_PACKAGE_NAME: usize = 256;
pub const KSU_MAX_GROUPS: usize = 32;
pub const KSU_SELINUX_DOMAIN: usize = 64;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    pub effective: u64,
    pub permitted: u64,
    pub inheritable: u64,
}

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

impl RootProfile {
    pub fn set_selinux_domain_for_tests(&mut self, domain: &str) {
        write_fixed_cstr(&mut self.selinux_domain, domain);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NonRootProfile {
    pub umount_modules: bool,
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
            use_default: false,
            template_name: [0; KSU_MAX_PACKAGE_NAME],
            profile: RootProfile::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NrpConfig {
    pub use_default: bool,
    pub profile: NonRootProfile,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union AppProfileConfig {
    pub rp_config: RpConfig,
    pub nrp_config: NrpConfig,
}

impl Default for AppProfileConfig {
    fn default() -> Self {
        Self {
            rp_config: RpConfig::default(),
        }
    }
}

impl core::fmt::Debug for AppProfileConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AppProfileConfig").finish_non_exhaustive()
    }
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

impl Default for AppProfile {
    fn default() -> Self {
        Self {
            version: KSU_APP_PROFILE_VER,
            key: [0; KSU_MAX_PACKAGE_NAME],
            current_uid: 0,
            allow_su: false,
            config: AppProfileConfig::default(),
        }
    }
}

impl AppProfile {
    pub fn new_for_tests(key: &str, current_uid: i32) -> Self {
        let mut profile = Self {
            current_uid,
            ..Self::default()
        };
        write_fixed_cstr(&mut profile.key, key);
        profile
    }

    pub fn root_config_mut_for_tests(&mut self) -> &mut RpConfig {
        // SAFETY: This host-side checker uses the same selector contract as
        // the C union. Tests call this only to initialize the root-profile arm
        // before validation reads it for `allow_su == true` or key "#".
        unsafe { &mut self.config.rp_config }
    }

    fn root_config(&self) -> &RpConfig {
        // SAFETY: `validate_app_profile` calls this only when the C validator
        // would read `rp_config`: `allow_su` is true or the key is "#".
        unsafe { &self.config.rp_config }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileValidationError {
    UnsupportedVersion,
    InvalidKey,
    InvalidTemplateName,
    InvalidGroupsCount,
    InvalidSelinuxDomain,
}

pub fn validate_app_profile(profile: &AppProfile) -> Result<(), ProfileValidationError> {
    if profile.version < KSU_APP_PROFILE_VER {
        return Err(ProfileValidationError::UnsupportedVersion);
    }

    if fixed_cstr_len(&profile.key, false).is_none() {
        return Err(ProfileValidationError::InvalidKey);
    }

    if profile.allow_su || fixed_cstr_eq(&profile.key, b"#") {
        let config = profile.root_config();
        if fixed_cstr_len(&config.template_name, true).is_none() {
            return Err(ProfileValidationError::InvalidTemplateName);
        }
        validate_root_profile(&config.profile)?;
    }

    Ok(())
}

fn validate_root_profile(profile: &RootProfile) -> Result<(), ProfileValidationError> {
    if profile.groups_count < 0 || profile.groups_count > KSU_MAX_GROUPS as i32 {
        return Err(ProfileValidationError::InvalidGroupsCount);
    }

    if fixed_cstr_len(&profile.selinux_domain, false).is_none() {
        return Err(ProfileValidationError::InvalidSelinuxDomain);
    }

    Ok(())
}

fn fixed_cstr_len(field: &[u8], allow_empty: bool) -> Option<usize> {
    let len = field.iter().position(|&byte| byte == 0)?;
    if !allow_empty && len == 0 {
        return None;
    }
    Some(len)
}

fn fixed_cstr_eq(field: &[u8], literal: &[u8]) -> bool {
    matches!(fixed_cstr_len(field, false), Some(len) if len == literal.len())
        && field[..literal.len()] == *literal
}

fn write_fixed_cstr(field: &mut [u8], value: &str) {
    assert!(value.len() < field.len());
    field.fill(0);
    field[..value.len()].copy_from_slice(value.as_bytes());
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuGetInfoCmd {
    pub version: u32,
    pub flags: u32,
    pub features: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuGetAllowListCmd {
    pub uids: [u32; 128],
    pub count: u32,
    pub allow: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuNewGetAllowListCmd {
    pub count: u16,
    pub total_count: u16,
    pub uids: [u32; 0],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuSetSepolicyCmd {
    pub cmd: u64,
    pub arg: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuSepolData {
    pub cmd: u32,
    pub subcmd: u32,
    pub sepol1: u64,
    pub sepol2: u64,
    pub sepol3: u64,
    pub sepol4: u64,
    pub sepol5: u64,
    pub sepol6: u64,
    pub sepol7: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuCompatSepolData {
    pub cmd: u32,
    pub subcmd: u32,
    pub sepol1: u32,
    pub sepol2: u32,
    pub sepol3: u32,
    pub sepol4: u32,
    pub sepol5: u32,
    pub sepol6: u32,
    pub sepol7: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuGetAppProfileCmd {
    pub profile: AppProfile,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuSetAppProfileCmd {
    pub profile: AppProfile,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuGetFeatureCmd {
    pub feature_id: u32,
    pub value: u64,
    pub supported: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuSetFeatureCmd {
    pub feature_id: u32,
    pub value: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuManageMarkCmd {
    pub operation: u32,
    pub pid: i32,
    pub result: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KsuAddTryUmountCmd {
    pub arg: u64,
    pub flags: u32,
    pub mode: u8,
}
