#![cfg_attr(not(test), no_main)]

mod init;
mod loader;

use std::ffi::CStr;

use rustix::{cstr, runtime::execve};
/// # Safety
/// This is the entry point of the program
/// We cannot use the main because rust will abort if we don't have std{in/out/err}
/// https://github.com/rust-lang/rust/blob/3071aefdb2821439e2e6f592f41a4d28e40c1e79/library/std/src/sys/unix/mod.rs#L80
/// So we use the C main function and call rust code from there
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(_argc: i32, argv: *const *const u8, envp: *const *const u8) -> i32 {
    if let Err(e) = init::init() {
        log::error!("ksuinit initialization failed: {e:#}");
    }

    unsafe { exec_init_fallbacks(argv, envp) }
}

fn init_handoff_candidates() -> [&'static CStr; 3] {
    [
        cstr!("/init"),
        cstr!("/init.real"),
        cstr!("/system/bin/init"),
    ]
}

unsafe fn exec_init_fallbacks(argv: *const *const u8, envp: *const *const u8) -> i32 {
    // Never silently ignore exec failures for PID1 handoff.
    for candidate in init_handoff_candidates() {
        let err = unsafe { execve(candidate, argv, envp) };
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

        assert_eq!(candidates[0].to_bytes(), b"/init");
        assert_eq!(candidates[1].to_bytes(), b"/init.real");
        assert_eq!(candidates[2].to_bytes(), b"/system/bin/init");
    }
}
