//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use clap::Command;
use libc::{EXIT_FAILURE, EXIT_SUCCESS, LOG_ALERT};
use nix::unistd::{
    alarm, chdir, execvp, getuid, initgroups, setgid, setgroups, setpgid, setuid, Pid, User,
};
use nix::NixPath;
use uucore::msg_log::{err_c, warn_c};
use uucore::{error::UResult, help_section, help_usage};

use crate::login_common::{
    chown_tty, display_login_messages, init_environ, init_loginpam_first_stage,
    init_loginpam_second_stage, init_tty, initialize, log_lastlog, log_syslog, log_utmp,
    loginpam_acct, loginpam_auth, loginpam_session, session_abort_msg, LoginContext, _PATH_BSHELL,
};
use crate::login_unsafe::{
    fork_session, openlog_login, pam_end_data_silent_wrapper, syslog_wrapper,
};
use crate::logindefs::{get_hushlogin_status, getlogindefs_bool};

/// login common functions
pub mod login_common;

/// login unsafe functions
pub mod login_unsafe;

/// Operate login defs functions
pub mod logindefs;

/// Some utils
pub mod utils;

/// Lastlog Implement
pub mod lastlog;

const ABOUT: &str = help_section!("about", "login.md");
const USAGE: &str = help_usage!("login.md");

#[uucore::main]
/// This the main() of login
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let mut ctx: LoginContext = initialize(args, ABOUT, USAGE)?;
    setpgid(Pid::from_raw(0), Pid::from_raw(0)).ok(); // the same as setpgrp()
    ctx.tty_info = Some(init_tty());
    openlog_login();
    let username_clone_tmp = ctx.username.clone();
    let pamh = init_loginpam_first_stage(ctx.remote_info.is_some(), username_clone_tmp.as_deref());
    init_loginpam_second_stage(&mut ctx, pamh);
    /* login -f, then the user has already been authenticated */
    ctx.noauth = ctx.noauth && getuid().is_root();
    if !ctx.noauth {
        loginpam_auth(&mut ctx, pamh);
    }
    loginpam_acct(&mut ctx, pamh);
    // After loginpam_acct(), we are sure that ctx.username is not None
    if let Ok(user_pwd) = User::from_name(ctx.username.as_deref().unwrap()) {
        ctx.pwd = user_pwd;
    }
    if ctx.pwd.is_none() {
        session_abort_msg(
            pamh,
            &format!("Invalid user name \"{}\". Abort.", ctx.username.unwrap()),
        );
    }
    // We are sure that ctx.pwd is not None
    let pwd = ctx.pwd.as_ref().unwrap();
    ctx.username = Some(pwd.name.clone());

    // Initialize the supplementary group list.
    let retcode = match pwd.uid.is_root() {
        false => initgroups(
            &CString::new(ctx.username.as_deref().unwrap()).unwrap_or_default(),
            pwd.gid,
        ),
        true => setgroups(&[]),
    };
    if retcode.is_err() {
        session_abort_msg(pamh, "groups initialization failed: %m");
    }

    ctx.quiet = get_hushlogin_status(pwd);

    /*
     * Open PAM session (after successful authentication and account check).
     */
    loginpam_session(&ctx, pamh);

    /* committed to login -- turn off timeout */
    alarm::cancel();

    log_utmp(&ctx);
    log_lastlog(&ctx);

    chown_tty(&ctx);

    if setgid(pwd.gid).is_err() && pwd.gid.as_raw() != 0 {
        syslog_wrapper(LOG_ALERT, "setgid() failed");
        exit(EXIT_FAILURE);
    }

    if pwd.shell.is_empty() {
        ctx.pwd.as_mut().unwrap().shell = PathBuf::from_str(_PATH_BSHELL).unwrap();
    }
    let pwd = ctx.pwd.as_ref().unwrap();

    init_environ(&ctx, pamh); /* init $HOME, $TERM ... */

    // We can't modify argv variable in Rust
    // So skip this function.
    // process_title_update(ctx.username);

    log_syslog(&ctx);

    if !ctx.quiet {
        display_login_messages();
    }

    /*
     * Detach the controlling terminal, fork, and create a new session
     * and reinitialize syslog stuff.
     */
    fork_session(pamh, &ctx.tty_info.as_ref().unwrap().tty_path);

    /* discard permissions last so we can't get killed and drop core */
    if setuid(pwd.uid).is_err() && pwd.uid.as_raw() != 0 {
        syslog_wrapper(LOG_ALERT, "setuid() failed");
        exit(EXIT_FAILURE);
    }

    /* wait until here to change directory! */
    if chdir(&pwd.dir).is_err() {
        warn_c(&format!(
            "{}: change directory failed",
            pwd.dir.to_string_lossy()
        ));

        if !getlogindefs_bool("DEFAULT_HOME", true) {
            exit(EXIT_SUCCESS);
        }
        if chdir("/").is_err() {
            exit(EXIT_FAILURE);
        }
        ctx.pwd.as_mut().unwrap().dir = PathBuf::from_str("/").unwrap();
        print!("Logging in with home = \"/\".\n");
    }
    let pwd = ctx.pwd.as_ref().unwrap();
    let mut pw_shell: Vec<u8> = pwd.shell.as_os_str().as_bytes().into();

    let mut child_argv: Vec<CString> = Vec::new();
    /* if the shell field has a space: treat it like a shell script */
    if pw_shell.contains(&b' ') {
        child_argv.push(CString::new("/bin/sh").unwrap());
        child_argv.push(CString::new("-sh").unwrap());
        child_argv.push(CString::new("-c").unwrap());
        let mut buff: Vec<u8> = "exec ".into();
        buff.append(&mut pw_shell);
        child_argv.push(CString::new(buff).unwrap());
    } else {
        child_argv.push(CString::new(pw_shell.clone()).unwrap());
        let p = pw_shell.rsplit(|v| *v == b'/').next();
        let mut tbuf = vec![b'-'];
        tbuf.extend_from_slice(p.unwrap_or(pw_shell.as_slice()));
        child_argv.push(CString::new(tbuf).unwrap());
    }

    pam_end_data_silent_wrapper(pamh);

    let file_name = child_argv.remove(0);
    execvp(&file_name, child_argv.as_slice()).ok();

    if file_name == CString::new("/bin/sh").unwrap() {
        warn_c("couldn't exec shell script");
    } else {
        err_c(EXIT_SUCCESS, "no shell");
    }

    Ok(())
}

/// This the oe_app of login
///
pub fn oe_app<'a>() -> Command<'a> {
    login_common::login_app(ABOUT, USAGE)
}
