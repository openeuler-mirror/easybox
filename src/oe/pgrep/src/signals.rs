//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use uucore::libc;

const SIGTABLE: &[(&str, i32)] = &[
    ("HUP", libc::SIGHUP),
    ("INT", libc::SIGINT),
    ("QUIT", libc::SIGQUIT),
    ("ILL", libc::SIGILL),
    ("TRAP", libc::SIGTRAP),
    ("ABRT", libc::SIGABRT),
    ("IOT", libc::SIGIOT), // possibly = ABRT
    ("BUS", libc::SIGBUS),
    ("FPE", libc::SIGFPE),
    ("KILL", libc::SIGKILL),
    ("USR1", libc::SIGUSR1),
    ("SEGV", libc::SIGSEGV),
    ("USR2", libc::SIGUSR2),
    ("PIPE", libc::SIGPIPE),
    ("ALRM", libc::SIGALRM),
    ("TERM", libc::SIGTERM),
    ("STKFLT", libc::SIGSTKFLT),
    ("CHLD", libc::SIGCHLD),
    ("CLD", libc::SIGCHLD), // possibly = CHLD
    ("CONT", libc::SIGCONT),
    ("STOP", libc::SIGSTOP),
    ("TSTP", libc::SIGTSTP),
    ("TTIN", libc::SIGTTIN),
    ("TTOU", libc::SIGTTOU),
    ("URG", libc::SIGURG),
    ("XCPU", libc::SIGXCPU),
    ("XFSZ", libc::SIGXFSZ),
    ("VTALRM", libc::SIGVTALRM),
    ("PROF", libc::SIGPROF),
    ("WINCH", libc::SIGWINCH),
    ("POLL", libc::SIGPOLL),
    ("IO", libc::SIGIO), // possibly = POLL
    ("PWR", libc::SIGPWR),
    ("SYS", libc::SIGSYS),
];

/// Convert a signal name to the corresponding signal value.
pub fn signal_name_to_number(name: &str) -> Option<i32> {
    if let Ok(n) = name.parse::<i32>() {
        if (0..=i32::MAX).contains(&n) {
            return Some(n);
        } else {
            return None;
        }
    }

    let name = if let Some(stripped_name) = name.strip_prefix("SIG") {
        stripped_name
    } else {
        name
    };

    for (n, v) in SIGTABLE.iter() {
        if name.to_uppercase().eq(n) {
            return Some(*v);
        }
    }

    let add: bool;
    if name.starts_with("RTMIN") {
        if name == "RTMIN" {
            return Some(libc::SIGRTMIN());
        } else if name.starts_with("RTMIN+") {
            add = true;
        } else {
            return None;
        }
    } else if name.starts_with("RTMAX") {
        if name == "RTMAX" {
            return Some(libc::SIGRTMAX());
        } else if name.starts_with("RTMAX-") {
            add = false;
        } else {
            return None;
        }
    } else {
        return None;
    }
    let name = &name[6..];
    if let Ok(n) = name.parse::<i32>() {
        if n >= 0 && n <= libc::SIGRTMAX() - libc::SIGRTMIN() {
            if add {
                return Some(libc::SIGRTMIN() + n);
            } else {
                return Some(libc::SIGRTMAX() - n);
            }
        }
    }

    None
}
