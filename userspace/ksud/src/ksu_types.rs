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
