//! This file is part of the easybox package.
//
// (c) Zhihua Zhao <YuukaC@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::file_unsafe::{is_wide_print, wide_char_width};

#[cfg(not(feature = "wide"))]
pub fn isprint(c: u8) -> bool {
    c.is_ascii_graphic() || c == b' '
}

pub fn fname_print(name: &str) {
    #[cfg(feature = "wide")]
    for c in name.chars() {
        if is_wide_print(c) {
            print!("{}", c);
        } else {
            print!("\\u{:04x}", c as u32);
        }
    }
    #[cfg(not(feature = "wide"))]
    for c in name.bytes() {
        if isprint(c) {
            print!("{}", c as char);
        } else {
            print!("\\{:03o}", c);
        }
    }
}

pub fn file_mbswidth(s: &str, raw: bool) -> usize {
    #[cfg(feature = "wide")]
    {
        s.chars().fold(0, |width, c| {
            width
                + if raw || is_wide_print(c) {
                    wide_char_width(c).max(1) as usize
                } else {
                    4
                }
        })
    }

    #[cfg(not(feature = "wide"))]
    s.bytes()
        .fold(0, |width, c| width + if raw || isprint(c) { 1 } else { 4 })
}

#[cfg(feature = "sandbox")]
pub fn enable_sandbox_full() -> bool {
    use libseccomp::*;

    if !crate::file_unsafe::prctl() {
        return false;
    }

    let ctx = ScmpFilterContext::new_filter(ScmpAction::KillThread);
    if ctx.is_err() {
        return false;
    }
    let mut ctx = ctx.unwrap();

    let syscalls = [
        "access",
        "brk",
        "close",
        "dup2",
        "exit",
        "exit_group",
        "faccessat",
        "fcntl",
        "fcntl64",
        "fstat",
        "fstat64",
        "fstatat64",
        "futex",
        "getdents",
        "getdents64",
        "lseek",
        "_llseek",
        "lstat",
        "lstat64",
        "madvise",
        "mmap",
        "mmap2",
        "mprotect",
        "mremap",
        "munmap",
        "newfstatat",
        "open",
        "openat",
        "pread64",
        "read",
        "readlink",
        "readlinkat",
        "rt_sigaction",
        "rt_sigprocmask",
        "rt_sigreturn",
        "select",
        "stat",
        "statx",
        "stat64",
        "sysinfo",
        "umask",
        "getpid",
        "unlink",
        "utimes",
        "write",
        "writev",
        "sigaltstack", // for rust to work
    ];

    for syscall in syscalls {
        let syscall = ScmpSyscall::from_name(syscall);
        if syscall.is_err() || ctx.add_rule(ScmpAction::Allow, syscall.unwrap()).is_err() {
            return false;
        }
    }

    let syscall = ScmpSyscall::from_name("ioctl");
    if syscall.is_err() {
        return false;
    }
    let syscall = syscall.unwrap();
    let params = [libc::FIONREAD, libc::TIOCGWINSZ, libc::TCGETS];

    for param in params {
        if ctx
            .add_rule_conditional(ScmpAction::Allow, syscall, &[scmp_cmp!($arg1 == param)])
            .is_err()
        {
            return false;
        }
    }

    let syscall = ScmpSyscall::from_name("prctl");
    if syscall.is_err() {
        return false;
    }
    let syscall = syscall.unwrap();

    if ctx
        .add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[
                scmp_cmp!($arg0 == libc::PR_SET_VMA as u64),
                scmp_cmp!($arg1 == libc::PR_SET_VMA_ANON_NAME as u64),
            ],
        )
        .is_err()
    {
        return false;
    }

    ctx.load().is_ok()
}
