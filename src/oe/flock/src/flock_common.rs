//! This file is part of the easybox package.
//
// (c)  Allen Xu <xubo3006@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use uucore::display::Quotable;
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;

use std::ffi::CString;
use std::path::Path;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use nix::errno::Errno;
use nix::fcntl::{flock, open, FlockArg, OFlag};
use nix::sys::signal::{
    sigaction, signal, SaFlags, SigAction, SigEvent, SigHandler, SigSet, SigevNotify, Signal,
};
use nix::sys::stat::Mode;
use nix::sys::time::TimeSpec;
use nix::sys::timer::{Expiration, Timer, TimerSetTimeFlags};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::time::{clock_gettime, ClockId};
use nix::unistd::{access, close, execvp, fork, AccessFlags, ForkResult};

use clap::{crate_version, Arg, Command};

///
pub static FLOCK_CMD_PARSE_ERROR: i32 = 1;
///
pub static FLOCK_CONFILCT_EXIT_CODE: i32 = 2;
///
pub static FLOCK_OSERR_EXIT_CODE: i32 = 3;
///
pub static FLOCK_CANTCREAT_EXIT_CODE: i32 = 4;
///
pub static FLOCK_NOINPUT_EXIT_CODE: i32 = 5;

const EX_USAGE: i32 = 64;
const EX_DATAERR: i32 = 65;
const EX_NOINPUT: i32 = 66;
const EX_UNAVAILABLE: i32 = 69;
const EX_OSERR: i32 = 71;
const EX_CANTCREAT: i32 = 73;
const EXIT_FAILURE: i32 = 1;

static mut TIMEOUT_EXPIRED: AtomicBool = AtomicBool::new(false);

///
pub struct Config {
    ///
    pub shared: bool,
    ///
    pub exclusive: bool,
    ///
    pub unlock: bool,
    ///
    pub nonblocking: bool,
    ///
    pub timeout_secs: Option<u64>,
    ///
    pub conflict_num: Option<i32>,
    ///
    pub close: bool,
    ///
    pub nofork: bool,
    ///
    pub verbose: bool,
    ///
    pub file_to_lock: Option<String>,
    ///
    pub fd_to_lock: Option<i32>,
    ///
    pub cmd_normal: Option<Vec<String>>,
    ///
    pub cmd_shell: Option<String>,
}

///
pub mod options {
    ///
    pub static SHARED: &str = "shared";
    ///
    pub static EXCLUSIVE: &str = "exclusive";
    ///
    pub static UNLOCK: &str = "unlock";
    ///
    pub static NONBLOCKING: &str = "nonblock";
    ///
    pub static TIMEOUT: &str = "timeout";
    ///
    pub static CONFLICT: &str = "conflict-exit-code";
    ///
    pub static CLOSE: &str = "close";
    ///
    pub static NOFORK: &str = "no-fork";
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static FILE: &str = "file";
    ///
    pub static COMMAND_SHELL: &str = "command-shell";
    ///
    pub static COMMAND_NORMAL: &str = "command-normal";
    ///
    pub static HELP: &str = "help";
    ///
    pub static VERSION: &str = "version";
}

impl Config {
    ///
    pub fn from(args_matches: &clap::ArgMatches) -> UResult<Self> {
        let mut file_descriptor: Option<i32> = None;
        let file: Option<String> = match args_matches.get_one::<String>(options::FILE) {
            Some(file_path) => {
                if file_path == "-" {
                    None
                } else {
                    let mut file_path_res: Option<String> = Some(file_path.to_owned());
                    if !Path::exists(Path::new(file_path)) {
                        match file_path.parse::<i32>() {
                            Ok(file_fd) => {
                                file_descriptor = Some(file_fd);
                                file_path_res = None;
                            }
                            Err(_e) => {}
                        }
                    }
                    file_path_res
                }
            }
            None => None,
        };

        let command_shell: Option<String> = args_matches
            .get_one::<String>(options::COMMAND_SHELL)
            .map(|v| v.to_owned());

        let command_normal: Option<Vec<String>> = args_matches
            .get_many::<String>(options::COMMAND_NORMAL)
            .map(|v| v.map(String::from).collect());

        let timeout_secs = args_matches
            .get_one::<String>(options::TIMEOUT)
            .map(|sec: &String| {
                sec.parse::<u64>().map_err(|_| {
                    USimpleError::new(
                        FLOCK_CMD_PARSE_ERROR,
                        format!("invalid timeout value: {}", sec.quote()),
                    )
                })
            })
            .transpose()?;

        let conflict_num = args_matches
            .get_one::<String>(options::CONFLICT)
            .map(|num| {
                num.parse::<i32>().map_err(|_| {
                    USimpleError::new(
                        FLOCK_CMD_PARSE_ERROR,
                        format!("invalid exit code: {}", num.quote()),
                    )
                })
            })
            .transpose()?;

        Ok(Self {
            shared: args_matches.contains_id(options::SHARED),
            exclusive: args_matches.contains_id(options::EXCLUSIVE),
            unlock: args_matches.contains_id(options::UNLOCK),
            nonblocking: args_matches.contains_id(options::NONBLOCKING),
            timeout_secs,
            conflict_num,
            close: args_matches.contains_id(options::CLOSE),
            nofork: args_matches.contains_id(options::NOFORK),
            verbose: args_matches.contains_id(options::VERBOSE),
            file_to_lock: file,
            fd_to_lock: file_descriptor,
            cmd_shell: command_shell,
            cmd_normal: command_normal,
        })
    }
}

///
pub fn parse_flock_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = flock_app(about, usage);
    let arg_list = args.collect_lossy();
    // Parse the valid command in arg_list
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn flock_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        // Format arguments.
        .arg(
            Arg::new(options::SHARED)
                .short('s')
                .long(options::SHARED)
                .help("test get a shared lock"),
        )
        .arg(
            Arg::new(options::EXCLUSIVE)
                .short('x')
                .short('e')
                .long(options::EXCLUSIVE)
                .help("get an exclusive lock (default)"),
        )
        .arg(
            Arg::new(options::UNLOCK)
                .short('u')
                .long(options::UNLOCK)
                .help("remove a lock"),
        )
        .arg(
            Arg::new(options::NONBLOCKING)
                .alias("nb")
                .short('n')
                .long(options::NONBLOCKING)
                .help("fail rather than wait"),
        )
        .arg(
            Arg::new(options::TIMEOUT)
                .alias("wait")
                .short('w')
                .long(options::TIMEOUT)
                .value_name("secs")
                .takes_value(true)
                .help("wait for a limited amount of time"),
        )
        .arg(
            Arg::new(options::CONFLICT)
                .short('E')
                .long(options::CONFLICT)
                .value_name("number")
                .takes_value(true)
                .help("exit code after conflict or timeout"),
        )
        .arg(
            Arg::new(options::CLOSE)
                .short('o')
                .long(options::CLOSE)
                .help("close file descriptor before running command"),
        )
        .arg(
            Arg::new(options::NOFORK)
                .short('F')
                .long(options::NOFORK)
                .help("execute command without forking"),
        )
        .arg(
            Arg::new(options::VERBOSE)
                .long(options::VERBOSE)
                .help("increase verbosity"),
        )
        .arg(Arg::new(options::HELP).short('h').long(options::HELP))
        .arg(Arg::new(options::VERSION).short('v').long(options::VERSION))
        .arg(
            Arg::new(options::FILE)
                .index(1)
                .value_hint(clap::ValueHint::FilePath),
        )
        .arg(
            Arg::new(options::COMMAND_SHELL)
                .short('c')
                .long(options::COMMAND_SHELL)
                .value_name("command")
                .takes_value(true)
                .help("run a single command string through the shell"),
        )
        .arg(
            Arg::new(options::COMMAND_NORMAL)
                .action(clap::ArgAction::Append)
                .index(2)
                .multiple_values(true)
                .value_hint(clap::ValueHint::CommandName),
        )
        .trailing_var_arg(true)
}

///
pub fn handle_input(mut config: Config) -> UResult<i32> {
    let mut status: i32 = 0;
    let mut fd: i32;
    let mut open_flags: OFlag = OFlag::from_bits_truncate(0);
    let mut cmd_args: Vec<String> = Vec::new();
    let mut flock_type = FlockArg::LockExclusive;
    let mut flock_block_bool: bool = false;
    let mut conflict_exit_code: i32 = 1;
    let mut timer: Option<Timer> = None;
    let mut time_start: Option<TimeSpec> = None;

    if config.shared {
        flock_type = FlockArg::LockShared;
    }
    if config.exclusive {
        flock_type = FlockArg::LockExclusive;
    }
    if config.unlock {
        flock_type = FlockArg::Unlock;
    }
    if config.nonblocking {
        flock_block_bool = true;
    }

    if config.conflict_num.is_some() {
        conflict_exit_code = config.conflict_num.unwrap();
        if !(0..=255).contains(&conflict_exit_code) {
            return Err(USimpleError::new(
                EX_USAGE,
                "exit code out of range (expected 0 to 255)",
            ));
        }
    }

    if config.nofork && config.close {
        return Err(USimpleError::new(
            EX_USAGE,
            "the --no-fork and --close options are incompatible",
        ));
    }

    if config.cmd_shell.is_some() {
        if config.cmd_normal.is_some() {
            let command_tmp = config.cmd_normal.take().unwrap()[1].clone();
            return Err(USimpleError::new(
                EX_USAGE,
                format!("{} requires exactly one command argument", command_tmp),
            ));
        }
        let shell_env = match std::env::var("SHELL") {
            Ok(res) => res,
            Err(_e) => "/bin/sh".to_string(),
        };
        cmd_args.push(shell_env);
        cmd_args.push("-c".to_string());
        cmd_args.push(config.cmd_shell.clone().unwrap());
    } else if config.cmd_normal.is_some() {
        cmd_args = config.cmd_normal.clone().unwrap();
    }

    if config.file_to_lock.is_some() {
        fd = open_file(config.file_to_lock.clone().unwrap(), &mut open_flags)?;
    } else if config.fd_to_lock.is_some() {
        fd = config.fd_to_lock.unwrap();
    } else {
        return Err(USimpleError::new(
            EX_USAGE,
            "requires file descriptor, file or directory",
        ));
    }

    if config.timeout_secs.is_some() {
        if config.timeout_secs.unwrap() == 0 {
            /* -w 0 is equivalent to -n; this has to be
             * special-cased because setting an itimer to zero
             * means disabled!
             */
            config.timeout_secs = None;
            flock_block_bool = true;
        } else {
            timer = match setup_timer(config.timeout_secs.unwrap()) {
                Ok(timer) => Some(timer),
                Err(_e) => {
                    return Err(USimpleError::new(
                        EX_OSERR,
                        "cannot set up timer".to_string(),
                    ));
                }
            }
        }
    }
    if config.verbose {
        time_start = Some(gettime_monotonic());
    }

    let flock_arg = match flock_type {
        FlockArg::LockShared => {
            if flock_block_bool {
                FlockArg::LockSharedNonblock
            } else {
                flock_type
            }
        }
        FlockArg::LockExclusive => {
            if flock_block_bool {
                FlockArg::LockExclusiveNonblock
            } else {
                flock_type
            }
        }
        FlockArg::Unlock => {
            if flock_block_bool {
                FlockArg::UnlockNonblock
            } else {
                flock_type
            }
        }
        _ => flock_type,
    };

    loop {
        match flock(fd, flock_arg) {
            Ok(_) => {
                // Jump out of the while loop
                break;
            }
            Err(e) => {
                match e {
                    // Operation would block
                    Errno::EWOULDBLOCK => {
                        // -n option set and failed to lock.
                        if config.verbose {
                            eprintln!("failed to get lock");
                        }

                        return Err(USimpleError::new(
                            conflict_exit_code,
                            "failed to get lock".to_string(),
                        ));
                    }
                    // Interrupted system call
                    Errno::EINTR => {
                        // Signal received
                        if unsafe { TIMEOUT_EXPIRED.load(Ordering::Relaxed) } {
                            // -w option set and failed to lock.
                            if config.verbose {
                                eprintln!("timeout while waiting to get lock");
                            }
                            return Err(USimpleError::new(conflict_exit_code, ""));
                        }
                        // Otherwise try again
                        continue;
                    }
                    // Bad file number || I/O error
                    Errno::EBADF | Errno::EIO => {
                        if !open_flags.intersects(OFlag::O_RDWR)
                            && flock_type != FlockArg::LockShared
                            && (config).file_to_lock.is_some()
                            && access(
                                (config).file_to_lock.clone().unwrap().as_str(),
                                AccessFlags::R_OK | AccessFlags::W_OK,
                            )
                            .is_ok()
                        {
                            let _ = close(fd);
                            open_flags = OFlag::O_RDWR;
                            fd =
                                open_file((config).file_to_lock.clone().unwrap(), &mut open_flags)?;

                            if open_flags.intersects(OFlag::O_RDWR) {
                                continue;
                            }
                        }
                        return Err(USimpleError::new(EX_DATAERR, ""));
                    }

                    _ => {
                        if e == Errno::ENOLCK || e == Errno::ENOMEM {
                            return Err(USimpleError::new(EX_OSERR, ""));
                        }
                        return Err(USimpleError::new(EX_DATAERR, ""));
                    }
                }
            }
        }
    }

    if config.timeout_secs.is_some() {
        cancel_timer(timer.take().unwrap());
    }
    if config.verbose {
        let time_done: TimeSpec = gettime_monotonic();
        let delta = time_done - time_start.take().unwrap();
        println!(
            "flock: getting lock took {}.{} seconds",
            delta.tv_sec(),
            delta.tv_nsec()
        );
    }

    if !cmd_args.is_empty() {
        /* Clear any inherited settings */
        let _sig_res = unsafe { signal(Signal::SIGCHLD, SigHandler::SigDfl) };

        if config.verbose {
            println!("flock: executing {}", cmd_args[0]);
        }

        if !config.nofork {
            let f = unsafe { fork() };

            let fork_res = match f {
                Ok(fork_res) => fork_res,
                Err(_e) => {
                    return Err(USimpleError::new(EX_OSERR, "fork failed".to_string()));
                }
            };

            match fork_res {
                // child
                ForkResult::Child => {
                    if config.close {
                        let _ = close(fd);
                    }
                    run_program(cmd_args);
                }
                // parent
                ForkResult::Parent { child } => loop {
                    let w_res = waitpid(child, Some(WaitPidFlag::empty()));
                    match w_res {
                        Ok(w_res) => match w_res {
                            WaitStatus::Exited(_pid, sta) => {
                                status = sta;
                                break;
                            }
                            WaitStatus::Signaled(_pid, sig, _bool) => {
                                status = sig as i32 + 128;
                                break;
                            }
                            WaitStatus::StillAlive => {}
                            _ => {
                                status = EX_OSERR;
                                break;
                            }
                        },
                        Err(e) => {
                            if e.ne(&Errno::EINTR) {
                                eprintln!("waitpid failed");
                                status = EXIT_FAILURE;
                                break;
                            }
                        }
                    }
                },
            }
        } else {
            // no-fork execution
            run_program(cmd_args);
        }
    }

    Ok(status)
}

///
pub fn open_file(file_path: String, flags: &mut OFlag) -> UResult<i32> {
    let mut fl: OFlag = if flags.bits() == 0 {
        OFlag::O_RDONLY
    } else {
        *flags
    };

    fl |= OFlag::O_NOCTTY | OFlag::O_CREAT;

    let handle_openfd_error = |e: &Errno| {
        eprintln!("cannot open lock file {}", file_path);
        match *e {
            Errno::ENOMEM | Errno::EMFILE | Errno::ENFILE => exit(EX_OSERR),
            Errno::EROFS | Errno::ENOSPC => exit(EX_CANTCREAT),
            _ => exit(EX_NOINPUT),
        }
    };

    let fd = match open(
        file_path.as_str(),
        fl,
        Mode::S_IRUSR
            | Mode::S_IWUSR
            | Mode::S_IRGRP
            | Mode::S_IWGRP
            | Mode::S_IROTH
            | Mode::S_IWOTH,
    ) {
        Ok(fd) => fd,
        Err(e) => {
            // Linux doesn't like O_CREAT on a directory, even though it
            // should be a no-op; POSIX doesn't allow O_RDWR or O_WRONLY
            match e {
                Errno::EISDIR => {
                    fl = OFlag::O_RDONLY | OFlag::O_NOCTTY;
                    let fd_tmp = open(
                        file_path.as_str(),
                        fl,
                        Mode::S_IRUSR
                            | Mode::S_IWUSR
                            | Mode::S_IRGRP
                            | Mode::S_IWGRP
                            | Mode::S_IROTH
                            | Mode::S_IWOTH,
                    );
                    match fd_tmp {
                        Ok(fd) => fd,
                        Err(e) => handle_openfd_error(&e),
                    }
                }
                _ => handle_openfd_error(&e),
            }
        }
    };
    *flags = fl;
    Ok(fd)
}

fn run_program(cmd_args: Vec<String>) {
    let args: Vec<CString> = cmd_args
        .iter()
        .map(|s| CString::new(s.as_bytes()).unwrap())
        .collect();
    let res = execvp(&args[0], &args);

    match res {
        Ok(_res) => {}
        Err(e) => {
            eprintln!("failed to execute {}", cmd_args[0]);
            match e {
                Errno::ENOMEM => {
                    exit(EX_OSERR);
                }
                _ => {
                    exit(EX_UNAVAILABLE);
                }
            }
        }
    }
}

extern "C" fn timeout_handler(_sig: i32, _info: *mut libc::siginfo_t, _context: *mut libc::c_void) {
    unsafe { TIMEOUT_EXPIRED.store(true, Ordering::Relaxed) };
}

fn setup_timer(timeout_secs: u64) -> UResult<Timer> {
    let timeout_handler: SigHandler = SigHandler::SigAction(timeout_handler);
    let sa = SigAction::new(
        timeout_handler,
        SaFlags::SA_SIGINFO | SaFlags::SA_RESETHAND,
        SigSet::empty(),
    );
    let res = unsafe { sigaction(Signal::SIGALRM, &sa) };
    match res {
        Ok(_) => {}
        Err(_e) => return Err(USimpleError::new(0, "")),
    }

    let se = SigEvent::new(SigevNotify::SigevSignal {
        signal: Signal::SIGALRM,
        si_value: 0,
    });
    let clockid = ClockId::CLOCK_REALTIME;
    let timer_res = Timer::new(clockid, se);
    let mut timer = match timer_res {
        Ok(timer) => timer,
        Err(_e) => return Err(USimpleError::new(0, "")),
    };
    let expiration = Expiration::OneShot(Duration::from_secs(timeout_secs).into());
    let timerset_res = timer.set(expiration, TimerSetTimeFlags::empty());
    match timerset_res {
        Ok(_) => {}
        Err(_e) => return Err(USimpleError::new(0, "")),
    }
    Ok(timer)
}

fn cancel_timer(mut timer: Timer) {
    let exp_zero = Expiration::OneShot(Duration::from_millis(0).into());
    let _ = timer.set(exp_zero, TimerSetTimeFlags::empty());

    let default_action = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());
    let _res = unsafe { sigaction(Signal::SIGALRM, &default_action) };
}

fn gettime_monotonic() -> TimeSpec {
    let mut clock_time = TimeSpec::new(0, 0);
    let clock_time_res = clock_gettime(ClockId::CLOCK_REALTIME);
    match clock_time_res {
        Ok(time_spec) => clock_time = time_spec,
        Err(_e) => {}
    }
    clock_time
}
