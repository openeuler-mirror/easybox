//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use uucore::error::UResult;
use uucore::format_usage;
use uucore::msg_log::{err_c, errtryhelp_c, warnx_c};

use clap::{crate_version, Arg, Command};
use errno::errno;
use fork::{fork, Fork};
use nix::{
    libc::{ioctl, wait, ENOENT, EXIT_FAILURE, STDIN_FILENO, TIOCSCTTY, WEXITSTATUS, WIFEXITED},
    unistd::{execvp, getpgrp, getpid, setsid},
};
use std::ffi::CString;

/// Exit code when execvp failed and errno is not ENOENT.
pub static EX_EXEC_FAILED: i32 = 126;
/// Exit code when execvp failed and errno is ENOENT.
pub static EX_EXEC_ENOENT: i32 = 127;

/// Config.
pub struct Config {
    ///
    pub ctty: bool,
    ///
    pub fork: bool,
    ///
    pub wait: bool,
    ///
    pub command: Vec<String>,
}

/// options.
pub mod options {
    ///
    pub static CTTY: &str = "ctty";
    ///
    pub static FORK: &str = "fork";
    ///
    pub static WAIT: &str = "wait";
    ///
    pub static COMMAND: &str = "command";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        Ok(Self {
            ctty: options.contains_id(options::CTTY),
            fork: options.contains_id(options::FORK),
            wait: options.contains_id(options::WAIT),
            command: options
                .values_of(options::COMMAND)
                .unwrap_or_default()
                .map(std::string::ToString::to_string)
                .collect(),
        })
    }
}

///
pub fn parse_setsid_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = setsid_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn setsid_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::CTTY)
                .short('c')
                .long(options::CTTY)
                .help("set the controlling terminal to the current one")
                .display_order(0),
        )
        .arg(
            Arg::new(options::FORK)
                .short('f')
                .long(options::FORK)
                .help("always fork")
                .display_order(1),
        )
        .arg(
            Arg::new(options::WAIT)
                .short('w')
                .long(options::WAIT)
                .help("wait program to exit, and use the same return")
                .display_order(2),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .help("display this help")
                .display_order(3),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .help("display version")
                .display_order(4),
        )
        .arg(
            Arg::new(options::COMMAND)
                // Leave required check later to print warn message.
                .required(false)
                // Use CommandName for better compatibility.
                // REF: https://docs.rs/clap/latest/clap/enum.ValueHint.html
                .value_hint(clap::ValueHint::CommandName)
                .multiple_values(true)
                .hide(true),
        )
        .trailing_var_arg(true)
}

///
pub fn handle_input(config: Config) -> UResult<()> {
    if config.command.is_empty() {
        warnx_c("no command specified");
        errtryhelp_c(EXIT_FAILURE);
    }

    if config.fork || getpgrp() == getpid() {
        match fork() {
            Ok(Fork::Parent(child)) => {
                let mut status: i32 = 0;
                if !config.wait {
                    return Ok(());
                }
                if unsafe { wait(&mut status) } != child {
                    err_c(EXIT_FAILURE, "wait");
                }
                if WIFEXITED(status) {
                    return Err(WEXITSTATUS(status).into());
                }
                err_c(status, &format!("child {} did not exit normally", child));
            }
            Ok(Fork::Child) => {}
            Err(_) => err_c(EXIT_FAILURE, "fork"),
        }
    }

    match setsid() {
        Ok(_) => {}
        Err(_) => err_c(EXIT_FAILURE, "setsid failed"),
    }

    if config.ctty && unsafe { ioctl(STDIN_FILENO, TIOCSCTTY, 1) != 0 } {
        err_c(EXIT_FAILURE, "failed to set the controlling terminal");
    }

    let c_args: Vec<CString> = config
        .command
        .iter()
        .map(|s| CString::new(s.as_bytes()).unwrap())
        .collect();
    match execvp(&c_args[0], &c_args) {
        Ok(_) => {}
        Err(_) => err_c(
            if errno().0 == ENOENT {
                EX_EXEC_ENOENT
            } else {
                EX_EXEC_FAILED
            },
            &format!("failed to execute {}", config.command[0]),
        ),
    }

    Ok(())
}
