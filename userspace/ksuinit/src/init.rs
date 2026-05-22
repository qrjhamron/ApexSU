use std::io::{ErrorKind, Write};

use crate::loader::load_module;
use anyhow::Result;
use rustix::fs::{Mode, symlink, unlink};
use rustix::{
    fd::AsFd,
    fs::{Access, CWD, FileType, access, makedev, mkdir, mknodat},
    mount::{
        FsMountFlags, FsOpenFlags, MountAttrFlags, MoveMountFlags, UnmountFlags, fsconfig_create,
        fsmount, fsopen, move_mount, unmount,
    },
};

/// RAII helper to ensure kernel-interface filesystems are unmounted
/// before handoff to the real init.
struct AutoUmount {
    mountpoints: Vec<String>,
}

impl Drop for AutoUmount {
    fn drop(&mut self) {
        for mountpoint in self.mountpoints.iter().rev() {
            /* Use DETACH (lazy umount) to ensure the mount point is 
             * cleared even if some kernel thread is still accessing it. */
            if let Err(e) = unmount(mountpoint.as_str(), UnmountFlags::DETACH) {
                log::error!("Cannot umount {}: {}", mountpoint, e)
            }
        }
    }
}

/// Helper for mounting essential pseudo-filesystems.
fn mount_filesystem(name: &str, mountpoint: &str) -> Result<()> {
    mkdir(mountpoint, Mode::from_raw_mode(0o755)).or_else(|err| match err.kind() {
        ErrorKind::AlreadyExists => Ok(()),
        _ => Err(err),
    })?;
    /* Use the new mount API (fsopen/fsmount) for better error reporting 
     * on modern kernels. */
    let fs_fd = fsopen(name, FsOpenFlags::FSOPEN_CLOEXEC)?;
    fsconfig_create(fs_fd.as_fd())?;
    let mount_fd = fsmount(
        fs_fd.as_fd(),
        FsMountFlags::FSMOUNT_CLOEXEC,
        MountAttrFlags::empty(),
    )?;
    move_mount(
        mount_fd.as_fd(),
        "",
        CWD,
        mountpoint,
        MoveMountFlags::MOVE_MOUNT_F_EMPTY_PATH,
    )?;
    Ok(())
}

fn prepare_mount() -> AutoUmount {
    let mut mountpoints = vec![];

    /* procfs is needed for /proc/self/exe and kernel info */
    match mount_filesystem("proc", "/proc") {
        Ok(_) => mountpoints.push("/proc".to_string()),
        Err(e) => log::error!("Cannot mount procfs: {:?}", e),
    }

    /* sysfs is needed for LKM symbol resolution */
    match mount_filesystem("sysfs", "/sys") {
        Ok(_) => mountpoints.push("/sys".to_string()),
        Err(e) => log::error!("Cannot mount sysfs: {:?}", e),
    }

    AutoUmount { mountpoints }
}

/// Initializes kernel logging. If /dev/kmsg is missing, attempts to 
/// create it as a character device node.
fn setup_kmsg() {
    const KMSG: &str = "/dev/kmsg";
    let device = match access(KMSG, Access::EXISTS) {
        Ok(_) => KMSG,
        Err(_) => {
            /* 1, 11 are the standard major/minor numbers for /dev/kmsg */
            mknodat(
                CWD,
                "/kmsg",
                FileType::CharacterDevice,
                0o666.into(),
                makedev(1, 11),
            )
            .ok();
            "/kmsg"
        }
    };

    let _ = kernlog::init_with_device(device);
}

fn unlimit_kmsg() {
    /* Disable kmsg rate limiting to ensure we see all early-boot logs. */
    if let Ok(mut rate) = std::fs::File::options()
        .write(true)
        .open("/proc/sys/kernel/printk_devkmsg")
    {
        writeln!(rate, "on").ok();
    }
}

/// Core initialization routine for the ksuinit loader.
pub fn init() -> Result<()> {
    setup_kmsg();

    log::info!("Hello, KernelSU!");

    let _dontdrop = prepare_mount();

    unlimit_kmsg();

    if has_kernelsu() {
        log::info!("KernelSU may be already loaded in kernel, skip!");
    } else {
        log::info!("Loading kernelsu.ko..");
        if let Err(e) = load_module("/kernelsu.ko") {
            log::error!("Cannot load kernelsu.ko: {:?}", e);
        }
    }

    /* Prepare for init handoff. We must unlink the symlink we created
     * in the previous boot session or the raw loader binary. */
    unlink("/init")?;

    let real_init = match access("/init.real", Access::EXISTS) {
        Ok(_) => select_init_target(true),
        Err(_) => select_init_target(false),
    };

    log::info!("init is {}", real_init);
    symlink(real_init, "/init")?;

    Ok(())
}

fn select_init_target(init_real_exists: bool) -> &'static str {
    if init_real_exists {
        "init.real"
    } else {
        "/system/bin/init"
    }
}

/// Legacy check for older KernelSU versions using prctl.
fn has_kernelsu_legacy() -> bool {
    use syscalls::{Sysno, syscall};
    let mut version = 0;
    const CMD_GET_VERSION: i32 = 2;
    /* SAFETY: syscall 0xDEADBEEF is the custom KSU prctl bridge. */
    unsafe {
        let _ = syscall!(
            Sysno::prctl,
            0xDEADBEEF,
            CMD_GET_VERSION,
            std::ptr::addr_of_mut!(version)
        );
    }

    log::info!("KernelSU version (legacy): {}", version);

    version != 0
}

#[repr(C)]
#[derive(Default)]
struct GetInfoCmd {
    version: u32,
    flags: u32,
    features: u32,
}

/// Detects modern KernelSU via the reboot-bridge covert channel.
fn has_kernelsu_v2() -> bool {
    use syscalls::{Sysno, syscall};
    const KSU_INSTALL_MAGIC1: u32 = 0xDEADBEEF;
    const KSU_INSTALL_MAGIC2: u32 = 0xCAFEBABE;
    const KSU_IOCTL_GET_INFO: u32 = 0x80004b02; // _IOC(_IOC_READ, 'K', 2, 0)

    let mut fd: i32 = -1;
    /* SAFETY: reboot syscall with magic numbers is our stealth IOCTL channel. */
    unsafe {
        let _ = syscall!(
            Sysno::reboot,
            KSU_INSTALL_MAGIC1,
            KSU_INSTALL_MAGIC2,
            0,
            std::ptr::addr_of_mut!(fd)
        );
    }

    let version = if fd >= 0 {
        let mut cmd = GetInfoCmd::default();
        let version = unsafe {
            /* SAFETY: fd is a valid driver handle returned by the reboot bridge. */
            let ret = syscall!(Sysno::ioctl, fd, KSU_IOCTL_GET_INFO, &mut cmd as *mut _);

            match ret {
                Ok(_) => cmd.version,
                Err(_) => 0,
            }
        };

        unsafe {
            let _ = syscall!(Sysno::close, fd);
        }

        version
    } else {
        0
    };

    log::info!("KernelSU version: {}", version);

    version != 0
}

/// Check if any version of KernelSU is currently active in the kernel.
pub fn has_kernelsu() -> bool {
    has_kernelsu_v2() || has_kernelsu_legacy()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_target_prefers_init_real_over_system_init() {
        assert_eq!(select_init_target(true), "init.real");
    }

    #[test]
    fn init_target_falls_back_to_system_init_when_init_real_missing() {
        assert_eq!(select_init_target(false), "/system/bin/init");
    }

    #[test]
    fn get_info_cmd_matches_kernel_payload_size() {
        assert_eq!(std::mem::size_of::<GetInfoCmd>(), 12);
        assert_eq!(std::mem::align_of::<GetInfoCmd>(), 4);
    }
}
