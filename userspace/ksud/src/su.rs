//! Su command implementation for granting and managing root access.
//!
//! This is the entry point for the 'su' CLI tool. It handles argument
//! parsing and coordinates with the kernel to elevate privileges.

use crate::{
    defs,
    utils::{self, umask},
};
use anyhow::{Context, Result, bail};
use getopts::Options;
use libc::c_int;
use log::error;
use std::env;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::{
    ffi::{CStr, CString},
    process::Command,
};

use crate::ksucalls::get_wrapped_fd;
use rustix::{
    process::getuid,
    thread::{Gid, Uid, set_thread_res_gid, set_thread_res_uid},
};

/// Public API for granting root to the current process and spawning a shell.
pub fn grant_root(global_mnt: bool) -> Result<()> {
    /* coordinate with LKM to elevate current task's creds */
    crate::ksucalls::grant_root()?;

    let mut command = Command::new("sh");
    // SAFETY: pre_exec runs in a forked child before exec; switch_mnt_ns is safe to call here.
    let command = unsafe {
        command.pre_exec(move || {
            if global_mnt {
                /* mount ns 1 is the global namespace on Android */
                let _ = utils::switch_mnt_ns(1);
            }
            Result::Ok(())
        })
    };
    /* inject our binary dir into PATH so 'ksud' is always available */
    add_path_to_env(defs::BINARY_DIR)?;
    Err(command.exec().into())
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("KernelSU\n\nUsage: {program} [options] [-] [user [argument...]]");
    print!("{}", opts.usage(&brief));
}

/// Swaps current thread IDs. Used when the user requests su to a specific UID.
fn set_identity(uid: u32, gid: u32, groups: &[u32]) -> std::io::Result<()> {
    /* supplementary groups must be set before UID/GID drop */
    rustix::thread::set_thread_groups(
        groups
            .iter()
            .map(|g| Gid::from_raw(*g))
            .collect::<Vec<_>>()
            .as_ref(),
    )
    .map_err(Into::<std::io::Error>::into)?;

    let gid = Gid::from_raw(gid);
    let uid = Uid::from_raw(uid);
    
    /* set real, effective, and saved IDs. failure here is fatal to security boundary. */
    set_thread_res_gid(gid, gid, gid).map_err(Into::<std::io::Error>::into)?;
    set_thread_res_uid(uid, uid, uid).map_err(Into::<std::io::Error>::into)?;
    Ok(())
}

/// Redirects TTY FDs through the kernel's io_worker wrapper to bypass
/// strict SELinux TTY access restrictions.
fn wrap_tty(fd: c_int) {
    let inner_fn = move || -> Result<()> {
        // SAFETY: fd is a valid open file descriptor; isatty is safe to call.
        if unsafe { libc::isatty(fd) != 1 } {
            return Ok(());
        }
        let new_fd = get_wrapped_fd(fd).context("get_wrapped_fd")?;
        // SAFETY: new_fd and fd are valid file descriptors.
        if unsafe { libc::dup2(new_fd, fd) } == -1 {
            // SAFETY: __errno returns a valid pointer to the thread-local errno value.
            bail!("dup {new_fd} -> {fd} errno: {}", unsafe {
                *libc::__errno()
            });
        }
        // SAFETY: new_fd is a valid open file descriptor duplicated above.
        unsafe { libc::close(new_fd) };
        Ok(())
    };

    if let Err(e) = inner_fn() {
        error!("wrap tty {fd}: {e:?}");
    }
}

/// Core su logic. Assumes the kernel has already granted root to the caller.
#[allow(clippy::similar_names)]
pub fn root_shell() -> Result<()> {
    use anyhow::anyhow;
    let env_args: Vec<String> = env::args().collect();
    let program = env_args[0].clone();
    
    /* handle '-c' special case where everything following it is a single command string */
    let args = env_args.iter().position(|arg| arg == "-c").map_or_else(
        || env_args.clone(),
        |i| {
            let rest = env_args[i + 1..].to_vec();
            let mut new_args = env_args[..i].to_vec();
            new_args.push("-c".to_string());
            if !rest.is_empty() {
                new_args.push(rest.join(" "));
            }
            new_args
        },
    );

    let mut opts = Options::new();
    opts.optopt("c", "command", "pass COMMAND to the invoked shell", "COMMAND");
    opts.optflag("h", "help", "display this help message and exit");
    opts.optflag("l", "login", "pretend the shell to be a login shell");
    opts.optflag("p", "preserve-environment", "preserve the entire environment");
    opts.optopt("s", "shell", "use SHELL instead of the default /system/bin/sh", "SHELL");
    opts.optflag("v", "version", "display version number and exit");
    opts.optflag("V", "", "display version code and exit");
    opts.optflag("M", "mount-master", "force run in the global mount namespace");
    opts.optopt("g", "group", "Specify the primary group", "GROUP");
    opts.optmulti("G", "supp-group", "supplementary group list", "GROUP");
    opts.optflag("W", "no-wrapper", "don't use ksu fd wrapper");

    let args = args
        .into_iter()
        .map(|e| {
            if e == "-mm" { "-M".to_string() } else if e == "-cn" { "-z".to_string() } else { e }
        })
        .collect::<Vec<String>>();

    let matches = match opts.parse(&args[1..]) {
        Result::Ok(m) => m,
        Err(f) => {
            println!("{f}");
            print_usage(&program, &opts);
            std::process::exit(-1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return Ok(());
    }

    if matches.opt_present("v") {
        println!("{}:KernelSU", defs::VERSION_NAME);
        return Ok(());
    }

    if matches.opt_present("V") {
        println!("{}", defs::VERSION_CODE);
        return Ok(());
    }

    let shell = matches.opt_str("s").unwrap_or_else(|| "/system/bin/sh".to_string());
    let mut is_login = matches.opt_present("l");
    let preserve_env = matches.opt_present("p");
    let mount_master = matches.opt_present("M");
    let use_fd_wrapper = !matches.opt_present("W");

    let groups = matches
        .opt_strs("G")
        .into_iter()
        .map(|g| g.parse::<u32>().map_err(|_| anyhow!("Invalid GID: {g}")))
        .collect::<Result<Vec<_>, _>>()?;

    let mut gid = matches
        .opt_str("g")
        .map(|g| g.parse::<u32>().map_err(|_| anyhow!("Invalid GID: {g}")))
        .transpose()?;

    if gid.is_none() && !groups.is_empty() {
        gid = Some(groups[0]);
    }

    let args = matches
        .opt_str("c")
        .map(|cmd| vec!["-c".to_string(), cmd])
        .unwrap_or_default();

    let mut free_idx = 0;
    if !matches.free.is_empty() && matches.free[free_idx] == "-" {
        is_login = true;
        free_idx += 1;
    }

    let mut uid = getuid().as_raw();
    if free_idx < matches.free.len() {
        let name = &matches.free[free_idx];
        /* SAFETY: getpwnam(3) is called on a valid C string. We use it to resolve 
         * human-readable usernames to UIDs. */
        uid = unsafe {
            let pw = CString::new(name.as_str())
                .ok()
                .and_then(|c_name| libc::getpwnam(c_name.as_ptr()).as_ref());

            pw.map_or_else(|| name.parse::<u32>().unwrap_or(0), |pw| pw.pw_uid)
        }
    }

    let gid = gid.unwrap_or(uid);
    let arg0 = if is_login { "-" } else { &shell };

    let mut command = &mut Command::new(&shell);

    if !preserve_env {
        /* SAFETY: getpwuid(3) returns a pointer to a thread-local static struct. */
        let pw = unsafe { libc::getpwuid(uid).as_ref() };

        if let Some(pw) = pw {
            // SAFETY: pw fields are guaranteed valid by getpwuid.
            let home = unsafe { CStr::from_ptr(pw.pw_dir) };
            let pw_name = unsafe { CStr::from_ptr(pw.pw_name) };

            command = command
                .env("HOME", home.to_string_lossy().as_ref())
                .env("USER", pw_name.to_string_lossy().as_ref())
                .env("LOGNAME", pw_name.to_string_lossy().as_ref())
                .env("SHELL", &shell);
        }
    }

    add_path_to_env(defs::BINARY_DIR)?;

    if PathBuf::from(defs::KSURC_PATH).exists() && env::var("ENV").is_err() {
        command = command.env("ENV", defs::KSURC_PATH);
    }

    // SAFETY: pre_exec runs after fork() but before execve(). 
    // This is where we drop the kernel-granted root to the target user identity.
    command = unsafe {
        command.pre_exec(move || {
            umask(0o22);
            utils::switch_cgroups();

            if mount_master {
                let _ = utils::switch_mnt_ns(1);
            }

            if use_fd_wrapper {
                /* Wrap standard FDs to bypass SELinux TTY checks */
                wrap_tty(0);
                wrap_tty(1);
                wrap_tty(2);
            }

            /* Drop to target identity. Failure here must abort to prevent root escape. */
            set_identity(uid, gid, &groups)?;

            std::io::Result::Ok(())
        })
    };

    command = command.args(args).arg0(arg0);
    Err(command.exec().into())
}

fn add_path_to_env(path: &str) -> Result<()> {
    let mut paths =
        env::var_os("PATH").map_or(Vec::new(), |val| env::split_paths(&val).collect::<Vec<_>>());
    let new_path = PathBuf::from(path.trim_end_matches('/'));
    paths.push(new_path);
    let new_path_env = env::join_paths(paths)?;
    // SAFETY: set_var is called in a single-threaded daemon initialization context.
    unsafe { env::set_var("PATH", new_path_env) };
    Ok(())
}
