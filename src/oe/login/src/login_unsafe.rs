//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::ffi::{CStr, CString, OsString};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::Path;
use std::ptr::null;
use std::sync::atomic::Ordering;

use libc::{
    c_char, c_int, c_void, closelog, getdtablesize, gid_t, openlog, setpriority, syslog, uid_t,
    usleep, vhangup, winsize, LOG_AUTHPRIV, LOG_ERR, LOG_ODELAY, LOG_WARNING, PRIO_PROCESS,
    STDERR_FILENO, STDIN_FILENO, TIOCGWINSZ, TIOCNOTTY, TIOCSCTTY,
};
use nix::dir::Dir;
use nix::errno::Errno;
use nix::fcntl::OFlag;
use nix::sys::signal::Signal::SIGHUP;
use nix::sys::stat::Mode;
use nix::unistd::{close, fork};
use nix::{ioctl_none_bad, ioctl_read_bad, ioctl_write_int_bad, ioctl_write_ptr_bad};
use pam::ffi::{pam_conv, pam_handle_t, pam_message, pam_response};
use pam::{get_item, PamHandle, PamItemType, PamReturnCode};
use utmpx::sys::Utmpx;

use crate::login_common::{
    fork_session_signal_child, fork_session_signal_fork_err, fork_session_signal_parent,
    handle_timedout, loginpam_err, sig_handler, CHILD_PID,
};
use nix::sys::signal::{sigaction, signal, SaFlags, SigAction, SigHandler, SigSet, Signal};

// The misc_conv function is part of libpam_misc.
#[link(name = "pam_misc")]
extern "C" {
    /// misc_conv - text based conversation function
    pub fn misc_conv(
        num_msg: c_int,
        msg: *mut *const pam_message,
        resp: *mut *mut pam_response,
        appdata_ptr: *mut libc::c_void,
    ) -> c_int;
}

extern "C" {
    fn updwtmpx(__wtmpx_file: *const c_char, __utmpx: *const Utmpx) -> c_void;
}

/// Use misc_conv to build pam_conv structure
pub const CTX_CONV: pam_conv = pam_conv {
    conv: Some(misc_conv),
    appdata_ptr: std::ptr::null_mut(),
};

ioctl_read_bad!(
    /// Read tty window size by TIOCGWINSZ code
    get_tty_windowsize,
    TIOCGWINSZ,
    winsize
);
ioctl_write_ptr_bad!(
    /// Set tty window size by TIOCGWINSZ code
    set_tty_windowsize,
    TIOCGWINSZ,
    winsize
);
ioctl_none_bad!(
    /// Set TIOCNOTTY: If the given terminal was the controlling terminal of the
    /// calling process, give up this controlling terminal. If the process was
    /// session leader, then send SIGHUP and SIGCONT to the foreground process
    /// group and all processes in the current session lose their controlling terminal.
    tiocnotty_ioctl,
    TIOCNOTTY
);
ioctl_write_int_bad!(
    /// Set TIOCSCTTY: Make the given terminal the controlling terminal of the calling process.
    tiocsctty_ioctl,
    TIOCSCTTY
);

/// Register timeout signal handler
pub fn login_prepare_signal() {
    let act = sigaction_ignore_wrapper(Signal::SIGALRM);
    unsafe {
        sigaction(
            Signal::SIGALRM,
            &SigAction::new(
                SigHandler::Handler(handle_timedout),
                act.flags().difference(SaFlags::SA_RESTART),
                act.mask(),
            ),
        )
        .ok();
        signal(Signal::SIGQUIT, SigHandler::SigIgn).ok();
        signal(Signal::SIGINT, SigHandler::SigIgn).ok();
        setpriority(PRIO_PROCESS, 0, 0);
    }
}

/// close_range syscall wrapper
#[allow(unused_assignments)]
pub fn close_range_wrapper() {
    let mut is_err = true;
    let first = STDERR_FILENO + 1;
    #[cfg(target_env = "gnu")]
    {
        is_err = unsafe { libc::close_range(first.try_into().unwrap(), libc::c_uint::MAX, 0) }
            .is_negative();
    }
    if is_err {
        // Impl ul_close_all_fds() here
        // MUSL lib does not have close_range(), so use codes below to close all fds
        if let Ok(dir) = Dir::open(
            "/proc/self/fd",
            OFlag::O_RDONLY | OFlag::O_DIRECTORY,
            Mode::empty(),
        ) {
            let dfd = dir.as_raw_fd();
            for rd in dir {
                if let Ok(d) = rd {
                    if d.file_name().to_bytes() == ['.' as u8; 1]
                        || d.file_name().to_bytes() == ['.' as u8; 2]
                    {
                        continue;
                    }
                    let fd = i32::from_str_radix(d.file_name().to_str().unwrap_or("0"), 10)
                        .unwrap_or_default();
                    if fd < first || dfd < 0 || dfd == fd {
                        continue;
                    }
                    close(fd).ok();
                }
            }
        } else {
            let tbsz = unsafe { getdtablesize() };
            for fd in 0..tbsz {
                if first <= fd {
                    close(fd).ok();
                }
            }
        }
    }
}

/// syslog function wrapper
#[inline]
pub fn syslog_wrapper(pri: i32, msg: &str) {
    let cmsg = CString::new(msg).unwrap();
    unsafe {
        syslog(pri, cmsg.as_ptr());
    }
}

/// signal function wrapper
#[inline]
pub fn signal_wrapper(sig: Signal, handler: SigHandler) {
    unsafe {
        signal(sig, handler).ok();
    }
}

/// vhangup function wrapper
#[inline]
pub fn vhangup_wrapper() {
    unsafe {
        signal(SIGHUP, SigHandler::SigIgn).ok(); /* so vhangup() won't kill us */
        vhangup();
        signal(SIGHUP, SigHandler::SigDfl).ok();
    }
}

/// pam_end function wrapper (with PAM_DATA_SILENT option)
#[inline]
pub fn pam_end_data_silent_wrapper(pamh: &mut u8) {
    unsafe { pam::ffi::pam_end(pamh, pam::ffi::PAM_DATA_SILENT) };
}

/// Do ioctl() with TIOCGWINSZ
pub fn get_windowsize_wrapper() -> winsize {
    let mut ws = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        if get_tty_windowsize(STDIN_FILENO, &mut ws).is_err() {
            let msg = CString::new("TIOCGWINSZ ioctl failed: %m").unwrap();
            syslog(LOG_WARNING, msg.as_ptr());
        }
    }
    ws
}

/// Do ioctl() with TIOCSWINSZ
pub fn set_windowsize_wrapper(ws: winsize) {
    if (ws.ws_row > 0 || ws.ws_col > 0) && unsafe { set_tty_windowsize(STDIN_FILENO, &ws).is_err() }
    {
        let msg = CString::new("TIOCSWINSZ ioctl failed: %m").unwrap();
        unsafe { syslog(LOG_WARNING, msg.as_ptr()) };
    }
}

/// openlog function wrapper
/// Set login syslog title and other attribute
#[inline]
pub fn openlog_login() {
    let login_cstr = CString::new("login").unwrap();
    unsafe { openlog(login_cstr.as_ptr(), LOG_ODELAY, LOG_AUTHPRIV) };
}

/// updwtmpx function wrapper
#[inline]
pub fn updwtmpx_wrapper(file: &str, ut: Utmpx) {
    let ptr: *const Utmpx = &ut;
    unsafe { updwtmpx(file.as_ptr().cast(), ptr) };
}

/// Based on nix setresgid code
#[inline]
pub fn setregid_wrapper(rgid: gid_t, egid: gid_t) -> Result<(), Errno> {
    let res = unsafe { libc::setregid(rgid, egid) };

    Errno::result(res).map(drop)
}

/// Based on nix setresuid code
#[inline]
pub fn setreuid_wrapper(ruid: uid_t, euid: uid_t) -> Result<(), Errno> {
    let res = unsafe { libc::setreuid(ruid, euid) };

    Errno::result(res).map(drop)
}

/// Set sigaction with the given action
#[inline]
pub fn sigaction_wrapper(sig: Signal, sa: SigAction) {
    unsafe {
        sigaction(sig, &sa).ok();
    }
}

/// usleep function wrapper
#[inline]
pub fn usleep_wrapper(usec: u32) {
    unsafe { usleep(usec) };
}

/// Build OwnedFd from rawfd
#[inline]
pub fn rawfd_to_ownedfd(fd: RawFd) -> OwnedFd {
    unsafe { OwnedFd::from_raw_fd(fd) }
}

/// Ignore a signal, return the old sigaction
#[inline]
pub fn sigaction_ignore_wrapper(sig: Signal) -> SigAction {
    unsafe {
        sigaction(
            sig,
            &SigAction::new(SigHandler::SigIgn, SaFlags::empty(), SigSet::empty()),
        )
    }
    .unwrap_or(SigAction::new(
        SigHandler::SigDfl,
        SaFlags::empty(),
        SigSet::empty(),
    ))
}

/// Detach the controlling terminal, fork, and create a new session and reinitialize syslog stuff.
pub fn fork_session(mut pamh: &mut pam_handle_t, tty_path: &Path) {
    let sa = SigAction::new(SigHandler::SigIgn, SaFlags::empty(), SigSet::empty());
    let sa_new = SigAction::new(SigHandler::Handler(sig_handler), sa.flags(), sa.mask());
    unsafe {
        signal(Signal::SIGALRM, SigHandler::SigDfl).ok();
        signal(Signal::SIGQUIT, SigHandler::SigDfl).ok();
        signal(Signal::SIGTSTP, SigHandler::SigIgn).ok();
        sigaction(Signal::SIGINT, &sa).ok();
    }
    let oldsa_hup = unsafe { sigaction(Signal::SIGHUP, &sa).unwrap() };
    // Detach the controlling tty.
    unsafe {
        tiocnotty_ioctl(0).ok();
        sigaction(Signal::SIGHUP, &sa_new).ok();
    }
    let oldsa_term = unsafe { sigaction(Signal::SIGTERM, &sa_new).unwrap() };
    unsafe { closelog() };

    if let Ok(child_p) = unsafe { fork() } {
        if child_p.is_parent() {
            fork_session_signal_parent(&mut pamh);
        } else {
            /*
             * child
             */
            unsafe {
                sigaction(Signal::SIGHUP, &oldsa_hup).ok();
                sigaction(Signal::SIGTERM, &oldsa_term).ok();
            }
            fork_session_signal_child(tty_path);
            unsafe {
                if tiocsctty_ioctl(0, 1).is_err() {
                    let msg = CString::new("TIOCSCTTY failed: %m").unwrap();
                    syslog(LOG_ERR, msg.as_ptr());
                }
                signal(Signal::SIGINT, SigHandler::SigDfl).ok();
            }
        }
    } else {
        CHILD_PID.store(-1, Ordering::Relaxed);
        fork_session_signal_fork_err(&mut pamh);
    }
}

/* Below are the functions about PAM API */

/// pam_set_item function wrapper
pub fn pam_set_item_wrapper<P: Sized + Clone + Into<Vec<u8>>>(
    mut handle: &mut PamHandle,
    item_type: PamItemType,
    item: Option<&P>,
) {
    let cstr_item;
    let ptr = match item {
        Some(v) => {
            cstr_item = CString::new(v.clone()).unwrap();
            cstr_item.as_ptr()
        }
        None => null(),
    };
    match unsafe { pam::ffi::pam_set_item(handle, item_type as c_int, ptr.cast()) }.into() {
        pam::PamReturnCode::Success => (),
        err => loginpam_err(&mut handle, err),
    }
}

/// pam_get_username function wrapper
/// Modified based on https://github.com/1wilkens/pam/blob/master/src/client.rs#L105
pub fn loginpam_get_username(mut pamh: &mut PamHandle, need_handle_err: bool) -> Option<String> {
    let res_get_item = get_item(pamh, PamItemType::User);
    if let Ok(result) = res_get_item {
        // Pam user is a char *
        let ptr: *const c_char = unsafe { std::mem::transmute(result) };
        let username = unsafe { CStr::from_ptr(ptr) };
        match username.to_str() {
            Err(_) => {
                if need_handle_err {
                    loginpam_err(&mut pamh, PamReturnCode::System_Err);
                }
            }
            Ok(username) => return Some(username.to_string()),
        }
    } else if need_handle_err {
        loginpam_err(&mut pamh, res_get_item.unwrap_err().0);
    }
    None
}

/// pam_getenvlist function wrapper
/// Separate the env string to (key, value)
pub fn loginpam_getenvlist(pamh: &mut PamHandle) -> Vec<(OsString, Option<OsString>)> {
    let mut result = Vec::new();
    let ptr = unsafe { pam::ffi::pam_getenvlist(pamh) };
    let mut current = ptr;
    if !current.is_null() {
        while !(unsafe { *current }).is_null() {
            let one_line = unsafe { CStr::from_ptr(*current).to_bytes() };
            if !one_line.is_empty() {
                let mut pos = one_line.split(|x| *x == b'=');
                let key = OsString::from_vec(pos.next().unwrap().to_vec());
                let value = pos.next().map(|x| OsString::from_vec(x.to_vec()));
                result.push((key, value))
            }
            current = unsafe { current.add(1) };
        }
    }
    unsafe { pam::ffi::pam_misc_drop_env(ptr) };
    result
}
