//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::cell::Cell;
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{stdin, stdout};
use std::net::IpAddr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::{self, AtomicBool, AtomicI32};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use clap::{crate_version, Arg, ArgMatches, Command};
use dns_lookup::{getaddrinfo, AddrInfoHints};
use libc::{
    AI_ADDRCONFIG, EXIT_FAILURE, EXIT_SUCCESS, LOG_ERR, LOG_INFO, LOG_NOTICE, STDERR_FILENO,
    STDIN_FILENO, STDOUT_FILENO,
};
use nix::dir::{Dir, Entry, Type};
use nix::errno::Errno;
use nix::fcntl::FcntlArg::F_SETFL;
use nix::fcntl::{fcntl, open, openat, OFlag};
use nix::sys::signal::{kill, SigHandler, Signal};
use nix::sys::stat::{fchmod, lstat, stat, Mode, SFlag};
use nix::sys::termios::{tcgetattr, tcsetattr, ControlFlags, LocalFlags, SetArg};
use nix::sys::uio::{pread, pwrite};
use nix::sys::wait::wait;
use nix::unistd::{
    access, alarm, close, dup2, fchown, gethostname, getpid, getuid, isatty, setsid, sleep,
    AccessFlags, Gid, Group, Pid, Uid, User,
};
use pam::ffi::pam_handle_t;
use pam::{
    acct_mgmt, authenticate, chauthtok, close_session, open_session, setcred, strerror, PamError,
    PamFlag, PamHandle, PamReturnCode,
};
use utmpx::sys::{UtType, Utmpx};
use utmpx::{
    close_database, find_by_id, find_by_line, read_next_entry, reset_cursor, set_filename,
    write_line,
};
use uucore::error::UResult;
use uucore::format_usage;
use uucore::msg_log::{err_c, errx_c, warn_c, warnx_c};
use uucore::version_cmp::version_cmp;

use crate::lastlog::Lastlog;
use crate::login_unsafe::{
    close_range_wrapper, get_windowsize_wrapper, login_prepare_signal, loginpam_get_username,
    loginpam_getenvlist, openlog_login, pam_set_item_wrapper, rawfd_to_ownedfd,
    set_windowsize_wrapper, sigaction_ignore_wrapper, sigaction_wrapper, signal_wrapper,
    syslog_wrapper, updwtmpx_wrapper, vhangup_wrapper, CTX_CONV,
};
use crate::logindefs::{getlogindefs_bool, getlogindefs_num, getlogindefs_str, logindefs_setenv};
use crate::utils::{cast_addr, get_terminal_name, str2memcpy, ul_copy_file};

/// Default tty device node permission
const TTY_MODE: u32 = 0620;
/// Default login exit delay time when encounters an error
const LOGIN_EXIT_TIMEOUT: u32 = 5;
/// Default login retry times
const LOGIN_MAX_TRIES: u32 = 3;
/// Default unknown username
const UNKNOWN_USER: &str = "(unknown)";
/// Default tty group name
const TTYGRPNAME: &str = "tty";
const _PATH_BTMP: &str = "/var/log/btmp";
const _PATH_WTMP: &str = "/var/log/wtmp";
const _PATH_UTMP: &str = "/var/run/utmp";
const _PATH_MAILDIR: &str = "/var/mail";
const _PATH_LASTLOG: &str = "/var/log/lastlog";
const _PATH_DEFPATH: &str = "/usr/local/bin:/usr/bin";
const _PATH_MOTDFILE: &str = "/usr/share/misc/motd:/run/motd:/etc/motd";
const _PATH_DEFPATH_ROOT: &str = "/usr/local/sbin:/usr/local/bin:/sbin:/bin:/usr/sbin:/usr/bin";
const MOTDDIR_EXT: &[u8] = ".motd".as_bytes();
/// Default shell path for user
pub const _PATH_BSHELL: &str = "/bin/sh";
const LOGIN_TIMEOUT_DEF: u32 = 60;
static GOT_SIG: AtomicBool = AtomicBool::new(false);
/// Store child pid after fork()
pub static CHILD_PID: AtomicI32 = AtomicI32::new(0);

thread_local! {
    static G_TIMEOUT: Cell<u32> = Cell::new(LOGIN_TIMEOUT_DEF);
}

/// Login control struct
pub struct LoginContext {
    /// TTY info
    pub tty_info: Option<TTYInfo>,
    /// points to PAM, pwd or cmd_username
    pub username: Option<String>,

    /// user info
    pub pwd: Option<User>,

    /// this machine
    pub thishost: Option<String>,
    /// this machine's domain
    pub thisdomain: Option<String>,
    /// Remote host info used in utmp logging
    pub remote_info: Option<RemoteInfo>,

    /// getpid() value
    pub pid: Pid,

    /// hush file exists
    pub quiet: bool,
    /// suppress hostname
    pub nohost: bool,
    /// skip a login authentication
    pub noauth: bool,
    /// do not destroy the environment
    pub keep_env: bool,
}

/// Remote host info
pub struct RemoteInfo {
    /// Hostname for the remote host to be recorded
    pub hostname: String,
    /// Addresses corresponding to the hostname
    pub hostaddress: Option<IpAddr>,
}

/// TTY node info
pub struct TTYInfo {
    /// chmod() mode
    pub tty_mode: Mode,
    /// ttyname() return value
    pub tty_path: PathBuf,
    /// tty_path without /dev prefix
    pub tty_name: String,
    /// end of the tty_path
    pub tty_number: String,
}

/// Command Options
pub mod options {
    ///
    pub static KEEPENV: &str = "keep-env";
    ///
    pub static SKIPAUTH: &str = "skip-auth";
    ///
    pub static HOSTNAME: &str = "hostname";
    ///
    pub static SUPPRESSHOST: &str = "suppress-hostname";
    ///
    pub static USERNAME: &str = "username";
}

impl LoginContext {
    /// Generate Login Context
    /// Based on initialize() in login.c
    pub fn from(args_matches: &ArgMatches) -> UResult<Self> {
        // Default value
        let mut ctx = Self {
            pid: getpid(),
            pwd: None,
            tty_info: None,
            thishost: None,
            thisdomain: None,
            username: None,
            remote_info: None,
            quiet: false,
            noauth: args_matches.contains_id(options::SKIPAUTH),
            nohost: args_matches.contains_id(options::SUPPRESSHOST),
            keep_env: args_matches.contains_id(options::KEEPENV),
        };

        let timeout = getlogindefs_num("LOGIN_TIMEOUT", LOGIN_TIMEOUT_DEF);
        G_TIMEOUT.with(|v| v.set(timeout));
        login_prepare_signal();
        if timeout > 0 {
            alarm::set(timeout);
        }

        if let Some(hostname) = args_matches.get_one::<String>(options::HOSTNAME) {
            if !getuid().is_root() {
                errx_c(EXIT_FAILURE, "-h is for superuser only");
            }
            init_remote_info(&mut ctx, hostname.to_string());
        }

        if let Some(v) = args_matches.get_one::<String>(options::USERNAME) {
            ctx.username = Some(v.to_string());
        }

        close_range_wrapper();

        Ok(ctx)
    }
}

/// Generate login general Config
pub fn initialize(args: impl uucore::Args, about: &str, usage: &str) -> UResult<LoginContext> {
    let command = login_app(about, usage);
    let arg_list = args.collect_lossy();
    LoginContext::from(&command.try_get_matches_from(arg_list)?)
}

/// Command arguments setting
pub fn login_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        // Format arguments.
        .arg(
            Arg::new(options::KEEPENV)
                .short('p')
                .help("do not destroy the environment"),
        )
        .arg(
            Arg::new(options::SKIPAUTH)
                .short('f')
                .help("skip a login authentication"),
        )
        .arg(
            Arg::new(options::HOSTNAME)
                .short('h')
                .takes_value(true)
                .help("hostname to be used for utmp logging"),
        )
        .arg(
            Arg::new(options::SUPPRESSHOST)
                .short('H')
                .help("suppress hostname in the login prompt"),
        )
        .arg(Arg::new(options::USERNAME).index(1).hide(true))
}

/// Open and set correct permission to the current tty device node
pub fn init_tty() -> TTYInfo {
    let tty_mode = Mode::from_bits_truncate(getlogindefs_num("TTYPERM", TTY_MODE));
    let mut st = None;
    let res_tty_path = get_terminal_name();
    if res_tty_path.is_ok() {
        if let Ok(fstat) = lstat(&res_tty_path.as_ref().unwrap().0) {
            st = Some(fstat);
        }
    }
    let mut ac_flags = AccessFlags::R_OK;
    ac_flags.set(AccessFlags::W_OK, true);
    if st.is_none()
        || SFlag::from_bits_truncate(st.unwrap().st_mode).contains(SFlag::S_IFCHR) == false
        || (st.unwrap().st_nlink > 1
            && res_tty_path.as_ref().unwrap().0.starts_with("/dev/") == false)
        || access(&res_tty_path.as_ref().unwrap().0, ac_flags).is_err()
    {
        syslog_wrapper(LOG_ERR, "FATAL: bad tty");
        sleepexit(EXIT_FAILURE);
    }

    let (tty_path, tty_name, tty_number) = res_tty_path.unwrap();
    let ws = get_windowsize_wrapper();
    let tt = tcgetattr(stdin()).unwrap();
    let mut ttt = tt.clone();
    let fres = (fchown(0, None, None), fchmod(0, tty_mode));
    if (fres.0.is_err() || fres.1.is_err()) && fres.0.err() != Some(Errno::EROFS) {
        syslog_wrapper(
            LOG_ERR,
            &format!(
                "FATAL: {}: change permissions failed: %m",
                tty_path.display()
            ),
        );
        sleepexit(EXIT_FAILURE);
    }
    /* Kill processes left on this tty */
    ttt.control_flags.set(ControlFlags::HUPCL, false);
    tcsetattr(stdin(), SetArg::TCSANOW, &ttt).ok();
    /*  Let's close file descriptors before vhangup */
    close(STDIN_FILENO).ok();
    close(STDOUT_FILENO).ok();
    close(STDERR_FILENO).ok();

    vhangup_wrapper();
    /* open stdin,stdout,stderr to the tty */
    open_tty(&tty_path);
    /* restore tty modes */
    tcsetattr(stdin(), SetArg::TCSAFLUSH, &tt).ok();
    /* Restore tty size */
    set_windowsize_wrapper(ws);
    TTYInfo {
        tty_mode,
        tty_path,
        tty_name,
        tty_number,
    }
}

/// Get a PamHandle
/// Using pam_start()
pub fn init_loginpam_first_stage(
    ctx_remote_info_is_some: bool,
    username: Option<&str>,
) -> &mut PamHandle {
    let res_pam_start = pam::start(
        match ctx_remote_info_is_some {
            false => "login",
            true => "remote",
        },
        username,
        &CTX_CONV,
    );
    if let Err(e) = &res_pam_start {
        warnx_c(&format!("PAM failure, aborting: {}", e));
        syslog_wrapper(LOG_ERR, &format!("Couldn't initialize PAM: {}", e));
        sleepexit(EXIT_FAILURE);
    }
    res_pam_start.unwrap()
}

/// Set some items for the given PamHandle
/// Using pam_set_item()
pub fn init_loginpam_second_stage(ctx: &mut LoginContext, pamh: &mut PamHandle) {
    if let Some(v) = &ctx.remote_info {
        pam_set_item_wrapper(pamh, pam::PamItemType::RHost, Some(&v.hostname));
    }
    if let Some(v) = &ctx.tty_info {
        pam_set_item_wrapper(
            pamh,
            pam::PamItemType::TTY,
            Some(&v.tty_path.as_os_str().as_bytes()),
        );
    }
    pam_set_item_wrapper(
        pamh,
        pam::PamItemType::User_Prompt,
        Some(&loginpam_get_prompt(ctx)),
    );
    if let Some(v) = &ctx.username {
        pam_set_item_wrapper(pamh, pam::PamItemType::User, Some(&v.as_bytes()));
    }
    ctx.username = None;
}

/// This is called for the -h option, initializes cxt->RemoteInfo
fn init_remote_info(mut ctx: &mut LoginContext, remotehost: String) {
    get_thishost(&mut ctx);
    let mut hostaddress = None;
    let mut hints = AddrInfoHints::default();
    hints.flags = AI_ADDRCONFIG;
    if let Ok(mut info_it) = getaddrinfo(Some(&remotehost), None, Some(hints)) {
        if let Some(Ok(info)) = info_it.next() {
            hostaddress = Some(info.sockaddr.ip());
        }
    }
    ctx.remote_info = Some(RemoteInfo {
        hostname: remotehost,
        hostaddress,
    });
}

fn get_thishost(ctx: &mut LoginContext) -> &Option<String> {
    if ctx.thishost.is_none() {
        if let Ok(val) = gethostname() {
            let valstr = val.to_string_lossy();
            ctx.thishost = Some(valstr.to_string());
            ctx.thisdomain = match valstr.split_once('.') {
                None => None,
                Some(v) => Some(String::from(v.1)),
            };
        }
    }
    return &ctx.thishost;
}

fn sleepexit(eval: i32) {
    sleep(getlogindefs_num("FAIL_DELAY", LOGIN_EXIT_TIMEOUT));
    exit(eval);
}

fn open_tty(tty: &Path) {
    let res_fd = open(tty, OFlag::O_RDWR | OFlag::O_NONBLOCK, Mode::empty());
    if res_fd.is_err() {
        syslog_wrapper(LOG_ERR, "FATAL: can't reopen tty: %m");
        sleepexit(EXIT_FAILURE);
    }
    let fd = res_fd.unwrap();
    if isatty(fd) != Ok(true) {
        close(fd).ok();
        syslog_wrapper(
            LOG_ERR,
            &format!("FATAL: {} is not a terminal", tty.display()),
        );
        sleepexit(EXIT_FAILURE);
    }
    let mut flags =
        OFlag::from_bits_truncate(fcntl(fd, nix::fcntl::FcntlArg::F_GETFL).unwrap_or_default());
    flags.set(OFlag::O_NONBLOCK, false);
    fcntl(fd, F_SETFL(flags)).ok();
    for i in 0..fd {
        close(i).ok();
    }
    for i in 0..3 {
        if fd != i {
            dup2(fd, i).ok();
        }
    }
    if fd >= 3 {
        close(fd).ok();
    }
}

/// Implementation of the authenticate process
/// Using pam_authenticate()
pub fn loginpam_auth(ctx: &mut LoginContext, pamh: &mut PamHandle) {
    let mut failcount: u32 = 0;
    let hostname = match &ctx.remote_info {
        Some(v) => v.hostname.clone(),
        None => match &ctx.tty_info {
            Some(v) => v.tty_name.clone(),
            None => "<unknown>".to_string(),
        },
    };

    let show_unknown = getlogindefs_bool("LOG_UNKFAIL_ENAB", false);
    let retries = getlogindefs_num("LOGIN_RETRIES", LOGIN_MAX_TRIES);
    let keep_username = getlogindefs_bool("LOGIN_KEEP_USERNAME", false);

    let mut rc = authenticate(pamh, PamFlag::None);

    ctx.username = loginpam_get_username(pamh, false);

    failcount += 1;
    while (failcount < retries)
        && (rc == PamReturnCode::Auth_Err
            || rc == PamReturnCode::User_Unknown
            || rc == PamReturnCode::Cred_Insufficient
            || rc == PamReturnCode::Authinfo_Unavail)
    {
        if rc == PamReturnCode::User_Unknown && !show_unknown {
            ctx.username = None;
        } else {
            ctx.username = loginpam_get_username(pamh, false);
        }

        syslog_wrapper(
            libc::LOG_NOTICE,
            &format!(
                "FAILED LOGIN {} FROM {} FOR {}, {}",
                failcount,
                hostname,
                ctx.username.as_deref().unwrap_or(UNKNOWN_USER),
                strerror(pamh, rc)
            ),
        );

        log_btmp(ctx);

        if !keep_username || rc == PamReturnCode::User_Unknown {
            pam_set_item_wrapper::<String>(pamh, pam::PamItemType::User, None);
            eprint!("Login incorrect\n\n");
        } else {
            eprint!("Password incorrect\n\n");
        }

        rc = authenticate(pamh, PamFlag::None);

        failcount += 1;
    }

    if is_pam_failure(rc) {
        if rc == PamReturnCode::User_Unknown && !show_unknown {
            ctx.username = None;
        } else {
            ctx.username = loginpam_get_username(pamh, false);
        }

        if rc == PamReturnCode::MaxTries {
            syslog_wrapper(
                LOG_NOTICE,
                &format!(
                    "TOO MANY LOGIN TRIES ({}) FROM {} FOR {}, {}",
                    failcount,
                    hostname,
                    ctx.username.as_deref().unwrap_or(UNKNOWN_USER),
                    PamError::from(rc)
                ),
            );
        } else {
            syslog_wrapper(
                LOG_NOTICE,
                &format!(
                    "FAILED LOGIN SESSION FROM {} FOR {}, {}",
                    hostname,
                    ctx.username.as_deref().unwrap_or(UNKNOWN_USER),
                    PamError::from(rc)
                ),
            );
        }

        log_btmp(ctx);

        eprint!("\nLogin incorrect\n");
        pam::end(pamh, rc);
        sleepexit(EXIT_SUCCESS);
    }
}

/// Determine if the user's account is valid
/// Using pam_acct_mgmt()
pub fn loginpam_acct(ctx: &mut LoginContext, pamh: &mut PamHandle) {
    let mut rc = acct_mgmt(pamh, PamFlag::None);

    if rc == PamReturnCode::New_Authtok_Reqd {
        rc = chauthtok(pamh, PamFlag::Change_Expired_AuthTok);
    }

    if is_pam_failure(rc) {
        loginpam_err(pamh, rc);
    }

    // First get the username that we are actually using, though.
    ctx.username = loginpam_get_username(pamh, true);

    if ctx.username.is_none() || ctx.username.as_ref().unwrap().is_empty() {
        session_abort_msg(pamh, "NULL user name. Abort.");
    }
}

/// Start PAM session management
/// Using pam_setcred() and pam_open_session()
pub fn loginpam_session(ctx: &LoginContext, pamh: &mut PamHandle) {
    let mut rc = setcred(pamh, PamFlag::Establish_Cred);
    if is_pam_failure(rc) {
        loginpam_err(pamh, rc);
    }

    rc = open_session(pamh, ctx.quiet);
    if is_pam_failure(rc) {
        setcred(pamh, PamFlag::Delete_Cred);
        loginpam_err(pamh, rc);
    }

    rc = setcred(pamh, PamFlag::Reinitialize_Cred);
    if is_pam_failure(rc) {
        close_session(pamh, false);
        loginpam_err(pamh, rc);
    }
}

/// Set tty node permission
pub fn chown_tty(ctx: &LoginContext) {
    let uid = ctx.pwd.as_ref().unwrap().uid;
    let mut gid = ctx.pwd.as_ref().unwrap().gid;
    let tty_name = &ctx.tty_info.as_ref().unwrap().tty_name;
    let tty_mode = ctx.tty_info.as_ref().unwrap().tty_mode;
    let grname = getlogindefs_str("TTYGROUP", TTYGRPNAME);
    if !grname.is_empty() {
        if let Ok(Some(gr)) = Group::from_name(&grname) {
            gid = gr.gid;
        } else {
            gid = Gid::from_raw(getlogindefs_num("TTYGROUP", gid.as_raw()));
        }
    }
    if fchown(0, Some(uid), Some(gid)).is_err() {
        chown_err(tty_name, uid, gid);
    }
    if fchmod(0, tty_mode).is_err() {
        chmod_err(tty_name, tty_mode);
    }
}

/// Initialize $TERM, $HOME, ...
pub fn init_environ(ctx: &LoginContext, pamh: &mut PamHandle) {
    let pwd = ctx.pwd.as_ref().unwrap();
    let termenv = env::var("TERM");
    /* destroy environment unless user has requested preservation (-p) */
    if !ctx.keep_env {
        for (k, _) in env::vars_os() {
            env::remove_var(k);
        }
    }

    if env::var_os("HOME").is_none() {
        env::set_var("HOME", &pwd.dir);
    }
    env::set_var("USER", &pwd.name);
    env::set_var("SHELL", &pwd.shell);
    env::set_var(
        "TERM",
        match &termenv {
            Ok(v) => v,
            _ => "dumb",
        },
    );

    const PATH_ENV: &str = "PATH";
    if !pwd.uid.is_root() {
        if logindefs_setenv(PATH_ENV, "ENV_PATH", Some(_PATH_DEFPATH)) == false {
            err_c(EXIT_FAILURE, "failed to set the PATH environment variable");
        }
    } else if logindefs_setenv(PATH_ENV, "ENV_ROOTPATH", None) == false
        && logindefs_setenv(PATH_ENV, "ENV_SUPATH", Some(_PATH_DEFPATH_ROOT)) == false
    {
        err_c(EXIT_FAILURE, "failed to set the PATH environment variable");
    }

    if env::var_os("MAIL").is_none() {
        env::set_var("MAIL", format!("{}/{}", _PATH_MAILDIR, pwd.name));
    }
    env::set_var("LOGNAME", &pwd.name);
    for (k, v) in loginpam_getenvlist(pamh) {
        if let Some(vv) = v {
            env::set_var(k, vv);
        } else {
            env::remove_var(k);
        }
    }
}

/// PAM error handler
pub fn loginpam_err(pamh: &mut PamHandle, rc: PamReturnCode) {
    let msg = pam::strerror(pamh, rc);
    eprintln!("\n{}", msg);
    pam::end(pamh, rc);
    sleepexit(EXIT_FAILURE);
}

#[inline]
fn is_pam_failure(rc: PamReturnCode) -> bool {
    rc != PamReturnCode::Success
}

/// Called when session setup failed
#[inline]
pub fn session_abort_msg(pamh: &mut PamHandle, msg: &str) {
    warnx_c("\nSession setup problem, abort.");
    syslog_wrapper(LOG_ERR, msg);
    pam::end(pamh, PamReturnCode::System_Err);
    sleepexit(EXIT_FAILURE);
}
/*
 * Composes "<host> login: " string; or returns "login: " if -H is given or
 * LOGIN_PLAIN_PROMPT=yes configured.
 */
#[allow(non_upper_case_globals)]
fn loginpam_get_prompt(mut ctx: &mut LoginContext) -> String {
    let dflt_prompt = String::from("login: ");
    if ctx.nohost {
        return dflt_prompt;
    }
    if getlogindefs_bool("LOGIN_PLAIN_PROMPT", false) {
        return dflt_prompt;
    }
    let host = match get_thishost(&mut ctx) {
        Some(v) => v,
        None => return dflt_prompt,
    };
    format!("{} {}", host, dflt_prompt)
}

/*
 * Logs failed login attempts in _PATH_BTMP, if it exists.
 */
fn log_btmp(ctx: &LoginContext) {
    let mut ut = Utmpx::default();
    if let Some(v) = &ctx.tty_info {
        str2memcpy(&mut ut.ut_id, &v.tty_number);
        str2memcpy(&mut ut.ut_line, &v.tty_name);
    }
    let tv = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ut.ut_tv.tv_sec = tv.as_secs() as i64;
    ut.ut_tv.tv_usec = (tv.as_nanos() - tv.as_secs() as u128 * 1000000) as i64;

    ut.ut_type = UtType::LOGIN_PROCESS;
    ut.ut_pid = ctx.pid.as_raw();

    if let Some(v) = &ctx.remote_info {
        str2memcpy(&mut ut.ut_host, &v.hostname);
        if let Some(ip) = &v.hostaddress {
            ut.ut_addr_v6 = cast_addr(ip);
        }
    }

    updwtmpx_wrapper(_PATH_BTMP, ut);
}

///Update wtmp and utmp logs.
pub fn log_utmp(ctx: &LoginContext) {
    let mut ut = Utmpx::default();
    let mut utp = None;

    set_filename(&CString::new(_PATH_UTMP).unwrap()).ok(); // utmpxname(_PATH_UTMP) in C code
    reset_cursor(); // setutxent() in C code

    /* Find pid in utmp. */
    while let Ok(ut_tmp) = read_next_entry() {
        let ut_type_judge = match ut_tmp.ut_type {
            UtType::INIT_PROCESS
            | UtType::LOGIN_PROCESS
            | UtType::USER_PROCESS
            | UtType::DEAD_PROCESS => true,
            _ => false,
        };
        if ut_tmp.ut_pid == ctx.pid.as_raw() && ut_type_judge {
            utp = Some(ut_tmp);
            break;
        }
    }

    /* If we can't find a pre-existing entry by pid, try by line.*/
    if utp.is_none() && ctx.tty_info.is_some() {
        reset_cursor();
        ut.ut_type = UtType::LOGIN_PROCESS;
        str2memcpy(&mut ut.ut_line, &ctx.tty_info.as_ref().unwrap().tty_name);
        utp = find_by_line(&ut).ok(); // getutxline(&ut) in C code
    }

    /* If we can't find a pre-existing entry by pid and line, try it by id.*/
    if utp.is_none() && ctx.tty_info.is_some() {
        reset_cursor();
        ut.ut_type = UtType::DEAD_PROCESS;
        str2memcpy(&mut ut.ut_id, &ctx.tty_info.as_ref().unwrap().tty_number);
        utp = find_by_id(&ut).ok(); // getutxid(&ut) in C code
    }

    if let Some(v) = utp {
        ut = v;
    } else {
        ut = Utmpx::default();
    }

    if let Some(v) = &ctx.tty_info {
        if ut.ut_id[0] == 0 {
            str2memcpy(&mut ut.ut_id, &v.tty_number);
        }
        str2memcpy(&mut ut.ut_line, &v.tty_name);
    }
    if let Some(v) = &ctx.username {
        str2memcpy(&mut ut.ut_user, v);
    }

    let tv = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ut.ut_tv.tv_sec = tv.as_secs() as i64;
    ut.ut_tv.tv_usec = (tv.as_nanos() - tv.as_secs() as u128 * 1000000) as i64;
    ut.ut_type = UtType::USER_PROCESS;
    ut.ut_pid = ctx.pid.as_raw();
    if let Some(v) = &ctx.remote_info {
        str2memcpy(&mut ut.ut_host, &v.hostname);
        if let Some(ip) = &v.hostaddress {
            ut.ut_addr_v6 = cast_addr(ip);
        }
    }

    write_line(&ut).ok(); // pututxline(&ut) in C code
    close_database(); // endutxent() in C code

    updwtmpx_wrapper(_PATH_WTMP, ut);
}

/// Record login log into lastlog database
pub fn log_lastlog(ctx: &LoginContext) {
    if ctx.pwd.is_none() {
        return;
    }

    let pwd = ctx.pwd.as_ref().unwrap();
    if pwd.uid.as_raw() > getlogindefs_num("LASTLOG_UID_MAX", u32::MAX) {
        return;
    }

    let mut ll_buf = [0; Lastlog::size()];

    /* lastlog is huge on systems with large UIDs, ignore SIGXFSZ */
    let oldsa_xfsz = sigaction_ignore_wrapper(Signal::SIGXFSZ);

    let offset = pwd.uid.as_raw() as i64 * ll_buf.len() as i64;
    let path_lastlog = PathBuf::from_str(_PATH_LASTLOG).unwrap();
    let res_fd = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(path_lastlog);
    if let Ok(fd) = res_fd {
        /*
         * Print last log message.
         */
        if !ctx.quiet {
            if Ok(ll_buf.len()) == pread(&fd, &mut ll_buf, offset) {
                let ll = Lastlog::from(ll_buf);
                if ll.ll_time != 0 {
                    let ll_time =
                        DateTime::from_timestamp(ll.ll_time.into(), 0).unwrap_or_default();
                    print!("Last login: {} ", ll_time.format("%a %b %e %T"));

                    if !ll.ll_host.is_empty() {
                        print!("from {}\n", ll.ll_host);
                    } else {
                        print!("on {}\n", ll.ll_line);
                    }
                }
            }
        }

        let new_ll_line = match &ctx.tty_info {
            Some(v) => v.tty_name.clone(),
            None => String::new(),
        };
        let new_ll_host = match &ctx.remote_info {
            Some(v) => v.hostname.clone(),
            None => String::new(),
        };
        let ll = Lastlog {
            ll_time: Utc::now().timestamp() as u32,
            ll_line: new_ll_line,
            ll_host: new_ll_host,
        };
        if Ok(Lastlog::size()) != pwrite(fd, &ll.as_bytes(), offset) {
            warn_c("write lastlog failed");
        }
    }
    sigaction_wrapper(Signal::SIGXFSZ, oldsa_xfsz);
}

/// Record login log into syslog database
pub fn log_syslog(ctx: &LoginContext) {
    if ctx.tty_info.is_none() || ctx.pwd.is_none() {
        return;
    }
    let pwd = ctx.pwd.as_ref().unwrap();
    let tty_info = ctx.tty_info.as_ref().unwrap();
    if tty_info.tty_name.starts_with("ttyS") {
        syslog_wrapper(
            LOG_INFO,
            &format!("DIALUP AT {} BY {}", tty_info.tty_name, pwd.name),
        );
    }

    if pwd.uid.is_root() {
        if let Some(r) = ctx.remote_info.as_ref() {
            syslog_wrapper(
                LOG_NOTICE,
                &format!("ROOT LOGIN ON {} FROM {}", tty_info.tty_name, r.hostname),
            );
        } else {
            syslog_wrapper(LOG_NOTICE, &format!("ROOT LOGIN ON {}", tty_info.tty_name));
        }
    } else {
        if let Some(r) = ctx.remote_info.as_ref() {
            syslog_wrapper(
                LOG_INFO,
                &format!(
                    "LOGIN ON {} BY {} FROM {}",
                    tty_info.tty_name, pwd.name, r.hostname
                ),
            );
        } else {
            syslog_wrapper(
                LOG_INFO,
                &format!("LOGIN ON {} BY {}", tty_info.tty_name, pwd.name),
            );
        }
    }
}

/// Iterate through the motd directory and output its contents
pub fn display_login_messages() {
    let mut done = 0;
    let firstonly = getlogindefs_bool("MOTD_FIRSTONLY", false);

    let mb = getlogindefs_str("MOTD_FILE", _PATH_MOTDFILE);
    if mb.is_empty() {
        return;
    }
    let list = mb.split(':');

    for file in list {
        if let Ok(st) = stat(file) {
            let st_mode = SFlag::from_bits_truncate(st.st_mode);
            if st_mode.contains(SFlag::S_IFDIR) {
                done += motddir(file);
            }
            if st_mode.contains(SFlag::S_IFREG) && st.st_size > 0 {
                if let Ok(fd) = File::options().read(true).open(file) {
                    ul_copy_file(fd, stdout());
                }
                done += 1;
            }
            if firstonly && done > 0 {
                break;
            }
        }
    }
}

fn motddir(dirname: &str) -> i32 {
    let dd = open(
        dirname,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_DIRECTORY,
        Mode::empty(),
    )
    .unwrap_or(-1);
    if dd < 0 {
        return 0;
    }
    let mut done = 0;
    if let Ok(mut all_files) = Dir::from_fd(dd) {
        let mut nfiles: Vec<Entry> = all_files.iter().filter_map(motddir_filter).collect();
        nfiles.sort_by(|a, b| {
            version_cmp(
                &a.file_name().to_string_lossy(),
                &b.file_name().to_string_lossy(),
            )
        });
        for d in nfiles {
            let fd = openat(
                dd,
                d.file_name(),
                OFlag::O_RDONLY | OFlag::O_CLOEXEC,
                Mode::empty(),
            )
            .unwrap_or(-1);
            if fd >= 0 {
                ul_copy_file(rawfd_to_ownedfd(fd), stdout());
                done += 1;
            }
        }
    }
    close(dd).ok();
    return done;
}

fn motddir_filter(d: Result<Entry, Errno>) -> Option<Entry> {
    if let Ok(dire) = d {
        let typ = dire.file_type();
        if typ != None && typ != Some(Type::File) && typ != Some(Type::Symlink) {
            return None;
        }
        let name = dire.file_name().to_bytes();
        if name.get(0) == Some(&b'.') {
            return None;
        }
        if name.len() < MOTDDIR_EXT.len() + 1 || name.ends_with(MOTDDIR_EXT) == false {
            return None;
        }
        return Some(dire);
    }
    None
}

#[inline]
fn chown_err(what: &str, uid: Uid, gid: Gid) {
    syslog_wrapper(
        LOG_ERR,
        &format!("chown ({}, {}, {}) failed: %m", what, uid, gid),
    );
}

#[inline]
fn chmod_err(what: &str, mode: Mode) {
    syslog_wrapper(
        LOG_ERR,
        &format!("chmod ({}, {}) failed: %m", what, mode.bits()),
    );
}

/// Called when fork() failed
#[inline]
pub fn fork_session_signal_fork_err(pamh: &mut pam_handle_t) {
    warn_c("fork failed");
    pam::setcred(pamh, pam::PamFlag::Delete_Cred);
    let status = pam::close_session(pamh, false);
    pam::end(pamh, status);
    sleepexit(EXIT_FAILURE);
}

/// The parent process after forking a session
pub fn fork_session_signal_parent(pamh: &mut pam_handle_t) {
    close(STDIN_FILENO).ok();
    close(STDOUT_FILENO).ok();
    close(STDERR_FILENO).ok();

    sigaction_ignore_wrapper(Signal::SIGQUIT);
    sigaction_ignore_wrapper(Signal::SIGINT);

    /* wait as long as any child is there */
    while wait() == Err(Errno::EINTR) {}
    openlog_login();

    pam::setcred(pamh, PamFlag::Delete_Cred);
    let status = pam::close_session(pamh, false);
    pam::end(pamh, status);
    exit(EXIT_SUCCESS);
}

/// The child process after forking a session (safe part)
#[inline]
pub fn fork_session_signal_child(tty_path: &Path) {
    if GOT_SIG.load(atomic::Ordering::Relaxed) {
        exit(EXIT_FAILURE);
    }
    setsid().ok();
    open_tty(tty_path);
    openlog_login();
}

/// based on timedout() in login.c
pub extern "C" fn handle_timedout(_signal: libc::c_int) {
    signal_wrapper(Signal::SIGALRM, SigHandler::Handler(handle_timedout2));
    alarm::set(10);
    let t = G_TIMEOUT.with(|v| v.get());
    warnx_c(&format!("timed out after {} seconds", t));
    signal_wrapper(Signal::SIGALRM, SigHandler::SigIgn);
    alarm::cancel();
    handle_timedout2(0);
}

/// based on timedout2() in login.c
extern "C" fn handle_timedout2(signal: libc::c_int) {
    let _ = signal;
    if let Ok(mut ti) = tcgetattr(std::io::stdin()) {
        ti.local_flags.set(LocalFlags::ECHO, true);
        tcsetattr(std::io::stdin(), SetArg::TCSANOW, &ti).ok();
    }
    exit(EXIT_SUCCESS);
}

/// This handler can be used to inform a shell about signals to login.
pub extern "C" fn sig_handler(signal: libc::c_int) {
    let child_p = CHILD_PID.load(atomic::Ordering::Relaxed);
    if child_p > 0 {
        let pid = Pid::from_raw(-child_p);
        kill(pid, Signal::try_from(signal).unwrap()).ok();
        if signal == libc::SIGTERM {
            kill(pid, Signal::SIGHUP).ok();
        }
    } else {
        GOT_SIG.store(true, atomic::Ordering::Relaxed);
    }
}
