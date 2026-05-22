#![cfg_attr(not(test), no_main)]

mod init;
mod loader;

use std::ffi::CStr;

use rustix::{cstr, runtime::execve};

/// # Safety
/// This is the entry point of the program. 
/// Standard Rust main() is avoided to bypass CRT initialization checks for std{in/out/err},
/// which are not guaranteed to be valid during early PID1 initialization.
/// https://github.com/rust-lang/rust/blob/3071aefdb2821439e2e6f592f41a4d28e40c1e79/library/std/src/sys/unix/mod.rs#L80
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(_argc: i32, argv: *const *const u8, envp: *const *const u8) -> i32 {
    /* Perform early setup: mount kernel filesystems and load the LKM. 
     * Failures here are logged but not fatal to prevent total device bricking. */
    if let Err(e) = init::init() {
        log::error!("ksuinit initialization failed: {e:#}");
    }

    /* Transfer control to the real Android init. This never returns. */
    unsafe { exec_init_fallbacks(argv, envp) }
}

/// Potential paths for the original system init binary.
fn init_handoff_candidates() -> [&'static CStr; 3] {
    [
        cstr!("/init.real"),      /* Stock init renamed by boot_patch */
        cstr!("/system/bin/init"), /* GKI fallback */
        cstr!("/init"),           /* Last resort; likely ourselves if unlinking failed */
    ]
}

/// Attempts to execve into a valid init candidate.
/// # Safety
/// This function relies on raw pointers passed from the C entry point.
/// argv and envp must be null-terminated arrays of pointers to null-terminated strings.
unsafe fn exec_init_fallbacks(argv: *const *const u8, envp: *const *const u8) -> i32 {
    /* Cache self-exe path to avoid accidental recursive exec loops. */
    let self_exe = std::fs::read_link("/proc/self/exe").ok();

    /* Never silently ignore exec failures for PID1 handoff. 
     * A silent failure here results in a kernel panic (init exited). */
    for candidate in init_handoff_candidates() {
        let candidate_path = std::path::Path::new(candidate.to_str().unwrap_or_default());
        if let Some(ref self_path) = self_exe {
            if candidate_path == self_path {
                log::debug!("Skipping self-execve for {:?}", candidate);
                continue;
            }
        }

        let err = unsafe { execve(candidate, argv, envp) };
        /* If we are here, execve failed. Log the errno for the serial console. */
        log::error!("execve({:?}) failed: {}", candidate, err);
    }

    log::error!("FATAL: unable to exec any init candidate; refusing to continue");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_handoff_order_is_stable() {
        let candidates = init_handoff_candidates();

        assert_eq!(candidates[0].to_bytes(), b"/init.real");
        assert_eq!(candidates[1].to_bytes(), b"/system/bin/init");
        assert_eq!(candidates[2].to_bytes(), b"/init");
    }
}
