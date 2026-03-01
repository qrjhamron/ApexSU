//! KernelSU JNI bridge — Rust replacement for ksu.cc + jni.cc
//!
//! Provides ioctl wrappers for the KernelSU kernel driver and JNI functions
//! consumed by `me.weishu.kernelsu.Natives` on the Kotlin side.

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jint, jobject, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::ffi::CStr;
use std::sync::atomic::{AtomicI32, Ordering};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const KSU_APP_PROFILE_VER: u32 = 2;
const KSU_MAX_PACKAGE_NAME: usize = 256;
const KSU_MAX_GROUPS: usize = 32;
const KSU_SELINUX_DOMAIN: usize = 64;

/// Maximum valid Linux capability bit index (CAP_LAST_CAP as of kernel 6.x).
const CAP_LAST_CAP: u32 = 40;

// Feature IDs
const KSU_FEATURE_SU_COMPAT: u32 = 0;
const KSU_FEATURE_KERNEL_UMOUNT: u32 = 1;

// ---------------------------------------------------------------------------
// IOCTL command numbers (pre-computed from Linux _IOC macro)
// ---------------------------------------------------------------------------

const KSU_IOCTL_GET_INFO: libc::c_ulong = 0x8000_4B02;
const KSU_IOCTL_CHECK_SAFEMODE: libc::c_ulong = 0x8000_4B05;
const KSU_IOCTL_NEW_GET_ALLOW_LIST: libc::c_ulong = 0xC004_4B06;
const KSU_IOCTL_UID_SHOULD_UMOUNT: libc::c_ulong = 0xC000_4B09;
const KSU_IOCTL_GET_APP_PROFILE: libc::c_ulong = 0xC000_4B0B;
const KSU_IOCTL_SET_APP_PROFILE: libc::c_ulong = 0x4000_4B0C;
const KSU_IOCTL_GET_FEATURE: libc::c_ulong = 0xC000_4B0D;
const KSU_IOCTL_SET_FEATURE: libc::c_ulong = 0x4000_4B0E;

// ---------------------------------------------------------------------------
// Kernel-facing C structs  (#[repr(C)] to match kernel layout)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuGetInfoCmd {
    version: u32,
    flags: u32,
    features: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuCheckSafemodeCmd {
    in_safe_mode: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuNewGetAllowListCmd {
    count: u16,
    total_count: u16,
    uids: [u32; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuUidShouldUmountCmd {
    uid: u32,
    should_umount: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuGetFeatureCmd {
    feature_id: u32,
    value: u64,
    supported: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuSetFeatureCmd {
    feature_id: u32,
    value: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Capabilities {
    effective: u64,
    permitted: u64,
    inheritable: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RootProfile {
    uid: i32,
    gid: i32,
    groups_count: i32,
    groups: [i32; KSU_MAX_GROUPS],
    capabilities: Capabilities,
    selinux_domain: [u8; KSU_SELINUX_DOMAIN],
    namespaces: i32,
}

impl Default for RootProfile {
    fn default() -> Self {
        Self {
            uid: 0,
            gid: 0,
            groups_count: 0,
            groups: [0i32; KSU_MAX_GROUPS],
            capabilities: Capabilities::default(),
            selinux_domain: [0u8; KSU_SELINUX_DOMAIN],
            namespaces: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NonRootProfile {
    umount_modules: bool,
}

/// The root-profile configuration arm of the union inside `AppProfile`.
#[repr(C)]
#[derive(Clone, Copy)]
struct RpConfig {
    use_default: bool,
    template_name: [u8; KSU_MAX_PACKAGE_NAME],
    profile: RootProfile,
}

impl Default for RpConfig {
    fn default() -> Self {
        Self {
            use_default: false,
            template_name: [0u8; KSU_MAX_PACKAGE_NAME],
            profile: RootProfile::default(),
        }
    }
}

/// The non-root-profile configuration arm of the union.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NrpConfig {
    use_default: bool,
    profile: NonRootProfile,
}

/// Mirrors the kernel `app_profile` struct.  The C version uses a C union for
/// `rp_config` / `nrp_config`; we model the larger arm (`RpConfig`) and
/// reinterpret when `allow_su == false`.
#[repr(C)]
#[derive(Clone, Copy)]
struct AppProfile {
    version: u32,
    key: [u8; KSU_MAX_PACKAGE_NAME],
    current_uid: i32,
    allow_su: bool,
    rp_config: RpConfig,
}

impl Default for AppProfile {
    fn default() -> Self {
        Self {
            version: 0,
            key: [0u8; KSU_MAX_PACKAGE_NAME],
            current_uid: 0,
            allow_su: false,
            rp_config: RpConfig::default(),
        }
    }
}

impl AppProfile {
    /// Access the union as `NrpConfig` (valid when `allow_su == false`).
    fn nrp_config(&self) -> &NrpConfig {
        // SAFETY: `rp_config` and `nrp_config` occupy the same bytes in the
        // kernel struct (C union). `NrpConfig` is smaller than `RpConfig`, so
        // the reference is within bounds.  The layout is `#[repr(C)]`.
        unsafe { &*(std::ptr::from_ref(&self.rp_config).cast::<NrpConfig>()) }
    }

    /// Mutable access to the union as `NrpConfig`.
    fn nrp_config_mut(&mut self) -> &mut NrpConfig {
        // SAFETY: same reasoning as `nrp_config()`.
        unsafe { &mut *(std::ptr::from_mut(&mut self.rp_config).cast::<NrpConfig>()) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuGetAppProfileCmd {
    profile: AppProfile,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KsuSetAppProfileCmd {
    profile: AppProfile,
}

// ---------------------------------------------------------------------------
// Driver FD management
// ---------------------------------------------------------------------------

/// Cached file-descriptor of the `[ksu_driver]` anon-inode.
static DRIVER_FD: AtomicI32 = AtomicI32::new(-1);

/// Cached version info so we only query once.
static CACHED_VERSION: AtomicI32 = AtomicI32::new(0);
static CACHED_FLAGS: AtomicI32 = AtomicI32::new(0);

/// Scan `/proc/self/fd` for the `[ksu_driver]` anon-inode link.
fn scan_driver_fd() -> i32 {
    let dir = match std::fs::read_dir("/proc/self/fd") {
        Ok(d) => d,
        Err(_) => return -1,
    };

    for entry in dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip non-numeric entries
        let fd_num: i32 = match name_str.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if fd_num < 0 {
            continue;
        }

        let link_path = format!("/proc/self/fd/{}", name_str);
        let target = match std::fs::read_link(&link_path) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let target_str = target.to_string_lossy();
        if target_str.contains("[ksu_driver]") {
            return fd_num;
        }
    }

    -1
}

/// Issue an ioctl to the KernelSU driver, lazily resolving the FD.
///
/// Returns the raw `ioctl()` result.
fn ksuctl(op: libc::c_ulong, arg: *mut libc::c_void) -> libc::c_int {
    let mut fd = DRIVER_FD.load(Ordering::Relaxed);
    if fd < 0 {
        fd = scan_driver_fd();
        DRIVER_FD.store(fd, Ordering::Relaxed);
    }
    // SAFETY: `ioctl` is called on the driver FD with kernel-defined commands
    // and properly laid-out `#[repr(C)]` structs passed by pointer.
    unsafe { libc::ioctl(fd, op, arg) }
}

// ---------------------------------------------------------------------------
// ioctl wrapper helpers
// ---------------------------------------------------------------------------

fn get_info() -> KsuGetInfoCmd {
    let ver = CACHED_VERSION.load(Ordering::Relaxed);
    if ver != 0 {
        return KsuGetInfoCmd {
            version: ver as u32,
            flags: CACHED_FLAGS.load(Ordering::Relaxed) as u32,
            features: 0,
        };
    }
    let mut cmd = KsuGetInfoCmd::default();
    ksuctl(
        KSU_IOCTL_GET_INFO,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    );
    if cmd.version != 0 {
        CACHED_VERSION.store(cmd.version as i32, Ordering::Relaxed);
        CACHED_FLAGS.store(cmd.flags as i32, Ordering::Relaxed);
    }
    cmd
}

fn legacy_get_info() -> (i32, i32) {
    let mut version: i32 = -1;
    let mut flags: i32 = 0;
    let mut result: i32 = 0;
    // SAFETY: prctl with custom KernelSU command (0xDEADBEEF) for legacy
    // detection.  The three mutable pointers are valid stack locals.
    unsafe {
        libc::prctl(
            0xDEADBEEFu32 as libc::c_int,
            2i64 as libc::c_ulong,
            &mut version as *mut i32 as libc::c_ulong,
            &mut flags as *mut i32 as libc::c_ulong,
            &mut result as *mut i32 as libc::c_ulong,
        );
    }
    (version, flags)
}

fn get_version() -> u32 {
    get_info().version
}

fn is_safe_mode() -> bool {
    let mut cmd = KsuCheckSafemodeCmd::default();
    ksuctl(
        KSU_IOCTL_CHECK_SAFEMODE,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    );
    cmd.in_safe_mode != 0
}

fn is_lkm_mode() -> bool {
    let info = get_info();
    if info.version > 0 {
        return (info.flags & 0x1) != 0;
    }
    (legacy_get_info().1 & 0x1) != 0
}

fn is_manager() -> bool {
    let info = get_info();
    if info.version > 0 {
        return (info.flags & 0x2) != 0;
    }
    legacy_get_info().0 > 0
}

fn uid_should_umount(uid: i32) -> bool {
    let mut cmd = KsuUidShouldUmountCmd {
        uid: uid as u32,
        should_umount: 0,
    };
    ksuctl(
        KSU_IOCTL_UID_SHOULD_UMOUNT,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    );
    cmd.should_umount != 0
}

fn set_app_profile(profile: &AppProfile) -> bool {
    let mut cmd = KsuSetAppProfileCmd { profile: *profile };
    ksuctl(
        KSU_IOCTL_SET_APP_PROFILE,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    ) == 0
}

fn get_app_profile(profile: &mut AppProfile) -> i32 {
    let mut cmd = KsuGetAppProfileCmd { profile: *profile };
    let ret = ksuctl(
        KSU_IOCTL_GET_APP_PROFILE,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    );
    *profile = cmd.profile;
    ret
}

fn get_feature(feature_id: u32) -> Option<(u64, bool)> {
    let mut cmd = KsuGetFeatureCmd {
        feature_id,
        ..Default::default()
    };
    if ksuctl(
        KSU_IOCTL_GET_FEATURE,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    ) != 0
    {
        return None;
    }
    Some((cmd.value, cmd.supported != 0))
}

fn set_feature(feature_id: u32, value: u64) -> bool {
    let mut cmd = KsuSetFeatureCmd { feature_id, value };
    ksuctl(
        KSU_IOCTL_SET_FEATURE,
        std::ptr::from_mut(&mut cmd).cast::<libc::c_void>(),
    ) == 0
}

fn is_su_enabled() -> bool {
    match get_feature(KSU_FEATURE_SU_COMPAT) {
        Some((value, true)) => value != 0,
        _ => false,
    }
}

fn set_su_enabled(enabled: bool) -> bool {
    set_feature(KSU_FEATURE_SU_COMPAT, u64::from(enabled))
}

fn is_kernel_umount_enabled() -> bool {
    match get_feature(KSU_FEATURE_KERNEL_UMOUNT) {
        Some((value, true)) => value != 0,
        _ => false,
    }
}

fn set_kernel_umount_enabled(enabled: bool) -> bool {
    set_feature(KSU_FEATURE_KERNEL_UMOUNT, u64::from(enabled))
}

fn get_allow_list(cmd: &mut KsuNewGetAllowListCmd) -> bool {
    ksuctl(
        KSU_IOCTL_NEW_GET_ALLOW_LIST,
        (cmd as *mut KsuNewGetAllowListCmd).cast::<libc::c_void>(),
    ) == 0
}

// ---------------------------------------------------------------------------
// JNI helper utilities
// ---------------------------------------------------------------------------

/// Copy a Rust `&str` into a fixed-size `[u8; N]` buffer (C-string style).
fn str_to_fixed<const N: usize>(buf: &mut [u8; N], s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len().min(N - 1);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len] = 0;
}

/// Read a NUL-terminated C string from a fixed `[u8; N]` buffer.
fn fixed_to_string<const N: usize>(buf: &[u8; N]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(N);
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// Extract a `String` from a JNI `JString`, returning an empty string on
/// failure.
fn jstring_to_string(env: &mut JNIEnv, js: &JString) -> String {
    env.get_string(js).map(|s| s.into()).unwrap_or_default()
}

/// Get the size of a `java.util.List`.
fn list_size(env: &mut JNIEnv, list: &JObject) -> i32 {
    env.call_method(list, "size", "()I", &[])
        .and_then(|v| v.i())
        .unwrap_or(0)
}

/// Get element `i` from a `java.util.List` as an `int`.
fn list_get_int(env: &mut JNIEnv, list: &JObject, i: i32) -> i32 {
    let obj = env
        .call_method(list, "get", "(I)Ljava/lang/Object;", &[JValue::Int(i)])
        .and_then(|v| v.l())
        .unwrap_or_default();
    env.call_method(&obj, "intValue", "()I", &[])
        .and_then(|v| v.i())
        .unwrap_or(0)
}

/// Append an `Integer` to a `java.util.List`.
fn list_add_int(env: &mut JNIEnv, list: &JObject, value: i32) {
    let integer_cls = env.find_class("java/lang/Integer").unwrap_or_default();
    let integer = env
        .new_object(&integer_cls, "(I)V", &[JValue::Int(value)])
        .unwrap_or_default();
    let _ = env.call_method(
        list,
        "add",
        "(Ljava/lang/Object;)Z",
        &[JValue::Object(&integer)],
    );
}

/// Convert a `java.util.List<Integer>` of capability indices to a bitmask.
fn cap_list_to_bits(env: &mut JNIEnv, list: &JObject) -> u64 {
    let size = list_size(env, list);
    let mut result: u64 = 0;
    for i in 0..size {
        let cap = list_get_int(env, list, i);
        if cap >= 0 && (cap as u32) <= CAP_LAST_CAP {
            result |= 1u64 << cap;
        }
    }
    result
}

// ---------------------------------------------------------------------------
// JNI exports — names must match Kotlin `Natives` external declarations
// ---------------------------------------------------------------------------

/// `Natives.getVersion` — returns the KernelSU kernel version.
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_getVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let version = get_version();
    if version > 0 {
        return version as jint;
    }
    // Legacy fallback via prctl
    legacy_get_info().0
}

/// `Natives.getSuperuserCount` — total number of allowed UIDs.
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_getSuperuserCount(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let mut cmd = KsuNewGetAllowListCmd::default();
    if get_allow_list(&mut cmd) {
        cmd.total_count as jint
    } else {
        0
    }
}

/// `Natives.isSafeMode`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_isSafeMode(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if is_safe_mode() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.isLkmMode`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_isLkmMode(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if is_lkm_mode() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.isManager`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_isManager(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if is_manager() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.uidShouldUmount`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_uidShouldUmount(
    _env: JNIEnv,
    _class: JClass,
    uid: jint,
) -> jboolean {
    if uid_should_umount(uid) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.isSuEnabled`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_isSuEnabled(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if is_su_enabled() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.setSuEnabled`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_setSuEnabled(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) -> jboolean {
    if set_su_enabled(enabled != JNI_FALSE) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.isKernelUmountEnabled`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_isKernelUmountEnabled(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    if is_kernel_umount_enabled() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.setKernelUmountEnabled`
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_setKernelUmountEnabled(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) -> jboolean {
    if set_kernel_umount_enabled(enabled != JNI_FALSE) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// `Natives.getUserName` — resolve a UID to a username via `getpwuid`.
#[unsafe(no_mangle)]
#[allow(unused_mut)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_getUserName(
    mut env: JNIEnv,
    _class: JClass,
    uid: jint,
) -> jstring {
    // SAFETY: `getpwuid` is a POSIX function; the returned pointer is valid
    // until the next call to any getpw* function (single-threaded JNI call).
    let pw = unsafe { libc::getpwuid(uid as libc::uid_t) };
    if pw.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: `pw` is non-null and `pw_name` is a valid C string per POSIX.
    let name = unsafe { CStr::from_ptr((*pw).pw_name) };
    let name_str = name.to_string_lossy();
    if name_str.is_empty() {
        return std::ptr::null_mut();
    }
    env.new_string(name_str.as_ref())
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// `Natives.getAppProfile` — read profile from kernel for a given package/uid.
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_getAppProfile(
    mut env: JNIEnv,
    _class: JClass,
    pkg: JString,
    uid: jint,
) -> jobject {
    match get_app_profile_impl(&mut env, &pkg, uid) {
        Ok(obj) => obj.into_raw(),
        Err(e) => {
            log::error!("getAppProfile failed: {e}");
            std::ptr::null_mut()
        }
    }
}

fn get_app_profile_impl<'a>(
    env: &mut JNIEnv<'a>,
    pkg: &JString,
    uid: jint,
) -> Result<JObject<'a>, jni::errors::Error> {
    let pkg_str = jstring_to_string(env, pkg);
    if pkg_str.len() > KSU_MAX_PACKAGE_NAME - 1 {
        return Ok(JObject::null());
    }

    let mut profile = AppProfile {
        version: KSU_APP_PROFILE_VER,
        current_uid: uid,
        ..AppProfile::default()
    };
    str_to_fixed(&mut profile.key, &pkg_str);

    let use_default = get_app_profile(&mut profile) != 0;

    // Construct Natives$Profile Java object
    let cls = env.find_class("me/weishu/kernelsu/Natives$Profile")?;
    let obj = env.new_object(&cls, "()V", &[])?;

    // Set key + currentUid
    let key_jstr = env.new_string(fixed_to_string(&profile.key))?;
    env.set_field(
        &obj,
        "name",
        "Ljava/lang/String;",
        JValue::Object(&key_jstr),
    )?;
    env.set_field(&obj, "currentUid", "I", JValue::Int(profile.current_uid))?;

    if use_default {
        // No profile found — return default (no root, non-root use-default)
        env.set_field(&obj, "allowSu", "Z", JValue::Bool(JNI_FALSE))?;
        env.set_field(&obj, "nonRootUseDefault", "Z", JValue::Bool(JNI_TRUE))?;
        return Ok(obj);
    }

    if profile.allow_su {
        env.set_field(&obj, "allowSu", "Z", JValue::Bool(JNI_TRUE))?;
        env.set_field(
            &obj,
            "rootUseDefault",
            "Z",
            JValue::Bool(if profile.rp_config.use_default {
                JNI_TRUE
            } else {
                JNI_FALSE
            }),
        )?;

        let template = fixed_to_string(&profile.rp_config.template_name);
        if !template.is_empty() {
            let tmpl_jstr = env.new_string(&template)?;
            env.set_field(
                &obj,
                "rootTemplate",
                "Ljava/lang/String;",
                JValue::Object(&tmpl_jstr),
            )?;
        }

        env.set_field(&obj, "uid", "I", JValue::Int(profile.rp_config.profile.uid))?;
        env.set_field(&obj, "gid", "I", JValue::Int(profile.rp_config.profile.gid))?;

        // Groups
        let group_list = env.get_field(&obj, "groups", "Ljava/util/List;")?.l()?;
        let group_count = profile
            .rp_config
            .profile
            .groups_count
            .min(KSU_MAX_GROUPS as i32);
        for i in 0..group_count {
            list_add_int(
                env,
                &group_list,
                profile.rp_config.profile.groups[i as usize],
            );
        }

        // Capabilities — expand bitmask to list of indices
        let cap_list = env
            .get_field(&obj, "capabilities", "Ljava/util/List;")?
            .l()?;
        for i in 0..=CAP_LAST_CAP {
            if profile.rp_config.profile.capabilities.effective & (1u64 << i) != 0 {
                list_add_int(env, &cap_list, i as i32);
            }
        }

        let domain = fixed_to_string(&profile.rp_config.profile.selinux_domain);
        let domain_jstr = env.new_string(&domain)?;
        env.set_field(
            &obj,
            "context",
            "Ljava/lang/String;",
            JValue::Object(&domain_jstr),
        )?;
        env.set_field(
            &obj,
            "namespace",
            "I",
            JValue::Int(profile.rp_config.profile.namespaces),
        )?;
    } else {
        let nrp = profile.nrp_config();
        env.set_field(
            &obj,
            "nonRootUseDefault",
            "Z",
            JValue::Bool(if nrp.use_default { JNI_TRUE } else { JNI_FALSE }),
        )?;
        env.set_field(
            &obj,
            "umountModules",
            "Z",
            JValue::Bool(if nrp.profile.umount_modules {
                JNI_TRUE
            } else {
                JNI_FALSE
            }),
        )?;
    }

    Ok(obj)
}

/// `Natives.setAppProfile` — write profile to kernel.
#[unsafe(no_mangle)]
pub extern "system" fn Java_me_weishu_kernelsu_Natives_setAppProfile(
    mut env: JNIEnv,
    _class: JClass,
    profile_obj: JObject,
) -> jboolean {
    match set_app_profile_impl(&mut env, &profile_obj) {
        Ok(true) => JNI_TRUE,
        Ok(false) => JNI_FALSE,
        Err(e) => {
            log::error!("setAppProfile failed: {e}");
            JNI_FALSE
        }
    }
}

fn set_app_profile_impl(
    env: &mut JNIEnv,
    profile_obj: &JObject,
) -> Result<bool, jni::errors::Error> {
    // Read key
    let key_obj = env
        .get_field(profile_obj, "name", "Ljava/lang/String;")?
        .l()?;
    if key_obj.is_null() {
        return Ok(false);
    }
    let key_jstr: JString = key_obj.into();
    let key_str = jstring_to_string(env, &key_jstr);
    if key_str.len() > KSU_MAX_PACKAGE_NAME - 1 {
        return Ok(false);
    }

    let current_uid = env.get_field(profile_obj, "currentUid", "I")?.i()?;
    let allow_su = env.get_field(profile_obj, "allowSu", "Z")?.z()?;
    let umount_modules = env.get_field(profile_obj, "umountModules", "Z")?.z()?;

    let mut p = AppProfile {
        version: KSU_APP_PROFILE_VER,
        allow_su,
        current_uid,
        ..AppProfile::default()
    };
    str_to_fixed(&mut p.key, &key_str);

    if allow_su {
        let root_use_default = env.get_field(profile_obj, "rootUseDefault", "Z")?.z()?;
        p.rp_config.use_default = root_use_default;

        let tmpl_obj = env
            .get_field(profile_obj, "rootTemplate", "Ljava/lang/String;")?
            .l()?;
        if !tmpl_obj.is_null() {
            let tmpl_jstr: JString = tmpl_obj.into();
            let tmpl_str = jstring_to_string(env, &tmpl_jstr);
            str_to_fixed(&mut p.rp_config.template_name, &tmpl_str);
        }

        let uid = env.get_field(profile_obj, "uid", "I")?.i()?;
        let gid = env.get_field(profile_obj, "gid", "I")?.i()?;
        p.rp_config.profile.uid = uid;
        p.rp_config.profile.gid = gid;

        // Groups
        let groups_obj = env
            .get_field(profile_obj, "groups", "Ljava/util/List;")?
            .l()?;
        let groups_count = list_size(env, &groups_obj);
        if groups_count as usize > KSU_MAX_GROUPS {
            return Ok(false);
        }
        p.rp_config.profile.groups_count = groups_count;
        for i in 0..groups_count {
            p.rp_config.profile.groups[i as usize] = list_get_int(env, &groups_obj, i);
        }

        // Capabilities
        let caps_obj = env
            .get_field(profile_obj, "capabilities", "Ljava/util/List;")?
            .l()?;
        p.rp_config.profile.capabilities.effective = cap_list_to_bits(env, &caps_obj);

        // SELinux domain
        let domain_obj = env
            .get_field(profile_obj, "context", "Ljava/lang/String;")?
            .l()?;
        let domain_jstr: JString = domain_obj.into();
        let domain_str = jstring_to_string(env, &domain_jstr);
        str_to_fixed(&mut p.rp_config.profile.selinux_domain, &domain_str);

        let ns = env.get_field(profile_obj, "namespace", "I")?.i()?;
        p.rp_config.profile.namespaces = ns;
    } else {
        let nrp = p.nrp_config_mut();
        let non_root_use_default = env.get_field(profile_obj, "nonRootUseDefault", "Z")?.z()?;
        nrp.use_default = non_root_use_default;
        nrp.profile.umount_modules = umount_modules;
    }

    Ok(set_app_profile(&p))
}
