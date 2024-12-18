//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, ArgGroup, Command};
use regex::Regex;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;
use uucore::error::{UResult, USimpleError};
use uucore::libc::{getgrnam, getpgrp, getpwnam, getsid, EXIT_FAILURE, SIGTERM};
use uucore::{format_usage, util_name};

use crate::process::{walk_process, ProcessInformation};
use crate::signals::signal_name_to_number;

///
const EXIT_NO_MATCH: i32 = 1;
///
const EXIT_USAGE: i32 = 2;

/// Config.
pub struct Config {
    ///
    pub delimiter: String,
    ///
    pub list_name: bool,
    ///
    pub list_full: bool,
    ///
    pub inverse: bool,
    ///
    pub lightweight: bool,
    ///
    pub count: bool,
    ///
    pub full: bool,
    ///
    pub pgroup: Vec<usize>,
    ///
    pub group: Vec<usize>,
    ///
    pub ignore_case: bool,
    ///
    pub newest: bool,
    ///
    pub oldest: bool,
    ///
    pub older: Option<u64>,
    ///
    pub parent: Vec<usize>,
    ///
    pub session: Vec<usize>,
    ///
    pub signal: i32,
    ///
    pub terminal: Vec<String>,
    ///
    pub euid: Vec<usize>,
    ///
    pub uid: Vec<usize>,
    ///
    pub exact: bool,
    ///
    pub pid: Option<usize>,
    ///
    pub runstates: Option<String>,
    ///
    pub ignore_ancestors: bool,
    ///
    pub cgroup: Vec<String>,
    /// TODO
    pub ns: Option<usize>,
    /// TODO
    pub nslist: Vec<String>,
    ///
    pub env: Vec<String>,
    ///
    pub pattern: String,
    ///
    pub require_handler: bool,
}

/// options.
pub mod options {
    ///
    pub static DELIMITER: &str = "delimiter";
    ///
    pub static LIST_NAME: &str = "list-name";
    ///
    pub static LIST_FULL: &str = "list-full";
    ///
    pub static INVERSE: &str = "inverse";
    ///
    pub static LIGHTWEIGHT: &str = "lightweight";
    ///
    pub static COUNT: &str = "count";
    ///
    pub static FULL: &str = "full";
    ///
    pub static PGROUP: &str = "pgroup";
    ///
    pub static GROUP: &str = "group";
    ///
    pub static IGNORE_CASE: &str = "ignore-case";
    ///
    pub static NEWEST: &str = "newest";
    ///
    pub static OLDEST: &str = "oldest";
    ///
    pub static OLDER: &str = "older";
    ///
    pub static PARENT: &str = "parent";
    ///
    pub static SESSION: &str = "session";
    ///
    pub static SIGNAL: &str = "signal";
    ///
    pub static TERMINAL: &str = "terminal";
    ///
    pub static EUID: &str = "euid";
    ///
    pub static UID: &str = "uid";
    ///
    pub static EXACT: &str = "exact";
    ///
    pub static PIDFILE: &str = "pidfile";
    ///
    pub static LOGPIDFILE: &str = "logpidfile";
    ///
    pub static RUNSTATES: &str = "runstates";
    ///
    pub static IGNORE_ANCESTORS: &str = "ignore-ancestors";
    ///
    pub static CGROUP: &str = "cgroup";
    ///
    pub static NS: &str = "ns";
    ///
    pub static NSLIST: &str = "nslist";
    ///
    pub static ENV: &str = "env";
    ///
    pub static PATTERN: &str = "pattern";
    ///
    pub static REQUIRE_HANDLER: &str = "require-handler";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let delimiter = options
            .get_one::<String>(options::DELIMITER)
            .unwrap_or(&"\n".to_string())
            .to_string();

        let mut pgroup = Vec::new();
        for pgroup_str in options
            .get_many::<String>(options::PGROUP)
            .unwrap_or_default()
        {
            match pgroup_str.parse::<usize>() {
                Ok(pgid) => {
                    if pgid == 0 {
                        pgroup.push(unsafe { getpgrp() } as usize);
                    } else {
                        pgroup.push(pgid)
                    }
                }
                Err(_) => {
                    return Err(USimpleError::new(
                        EXIT_USAGE,
                        &format!("invalid process group: {}", pgroup_str),
                    ));
                }
            }
        }

        let mut group = Vec::new();
        for group_str in options
            .get_many::<String>(options::GROUP)
            .unwrap_or_default()
        {
            if let Ok(gid) = group_str.parse::<usize>() {
                group.push(gid);
            } else if let Some(gid) = Self::get_gid_by_name(group_str) {
                group.push(gid as usize)
            } else {
                return Err(USimpleError::new(
                    EXIT_USAGE,
                    &format!("invalid group name: {}", group_str),
                ));
            }
        }

        let older = options
            .get_one::<String>(options::OLDER)
            .map(|o| o.parse::<u64>().unwrap_or(0));

        let mut parent = Vec::new();
        for parent_str in options
            .get_many::<String>(options::PARENT)
            .unwrap_or_default()
        {
            match parent_str.parse::<usize>() {
                Ok(ppid) => {
                    parent.push(ppid);
                }
                Err(_) => {
                    return Err(USimpleError::new(
                        EXIT_USAGE,
                        &format!("not a number: {}", parent_str),
                    ));
                }
            }
        }

        let mut session = Vec::new();
        for session_str in options
            .get_many::<String>(options::SESSION)
            .unwrap_or_default()
        {
            match session_str.parse::<usize>() {
                Ok(sid) => {
                    if sid == 0 {
                        session.push(unsafe { getsid(0) } as usize);
                    } else {
                        session.push(sid)
                    }
                }
                Err(_) => {
                    return Err(USimpleError::new(
                        EXIT_USAGE,
                        &format!("invalid session id: {}", session_str),
                    ));
                }
            }
        }

        let mut signal: i32 = -1;
        match options.get_one::<String>(options::SIGNAL) {
            Some(signal_str) => match signal_name_to_number(signal_str) {
                Some(sig_val) => signal = sig_val,
                None => {
                    let number_str: String = signal_str
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !number_str.is_empty() {
                        if let Ok(sig_val) = number_str.parse::<i32>() {
                            signal = sig_val;
                        }
                    }
                    if signal == -1 {
                        return Err(USimpleError::new(
                            EXIT_USAGE,
                            &format!("Unknown signal \"{}\".", signal_str),
                        ));
                    }
                }
            },
            None => signal = SIGTERM,
        };

        let terminal: Vec<String> = options
            .get_many::<String>(options::TERMINAL)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let mut euid = Vec::new();
        for euid_str in options
            .get_many::<String>(options::EUID)
            .unwrap_or_default()
        {
            if let Ok(e) = euid_str.parse::<usize>() {
                euid.push(e);
            } else if let Some(e) = Self::get_uid_by_name(euid_str) {
                euid.push(e as usize)
            } else {
                return Err(USimpleError::new(
                    EXIT_USAGE,
                    &format!("invalid user name: {}", euid_str),
                ));
            }
        }

        let mut uid = Vec::new();
        for uid_str in options.get_many::<String>(options::UID).unwrap_or_default() {
            if let Ok(u) = uid_str.parse::<usize>() {
                uid.push(u);
            } else if let Some(u) = Self::get_uid_by_name(uid_str) {
                uid.push(u as usize)
            } else {
                return Err(USimpleError::new(
                    EXIT_USAGE,
                    &format!("invalid user name: {}", uid_str),
                ));
            }
        }

        let mut runstates = options.get_one::<String>(options::RUNSTATES).cloned();

        let cgroup: Vec<String> = options
            .get_many::<String>(options::CGROUP)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let ns = match options.get_one::<String>(options::NS) {
            Some(pid_str) => match pid_str.parse::<usize>() {
                Ok(pid) => Some(pid),
                Err(_) => {
                    // Fall-through to case 'r' in C.
                    runstates = Some(pid_str.to_string());
                    None
                }
            },
            None => None,
        };

        let nslist = options
            .get_many::<String>(options::NSLIST)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let env: Vec<String> = options
            .get_many::<String>(options::ENV)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let logpidfile = options.get_flag(options::LOGPIDFILE);
        let pidfile = options.get_one::<String>(options::PIDFILE).cloned();
        if logpidfile && pidfile.is_none() {
            return Err(USimpleError::new(
                EXIT_USAGE,
                &format!(
                    "-L without -F makes no sense\nTry `{} --help' for more information.",
                    util_name()
                ),
            ));
        }

        let mut pid = None;
        if let Some(file) = pidfile {
            let mut fp: Box<dyn BufRead> = if file == "-" {
                let f = io::stdin().lock();
                Box::new(f)
            } else {
                match File::open(file) {
                    Ok(f) => Box::new(io::BufReader::new(f)),
                    Err(_) => {
                        return Err(USimpleError::new(EXIT_FAILURE, "Unable to open pidfile"));
                    }
                }
            };

            // TODO: read lock

            let mut buf = String::new();
            if fp.read_line(&mut buf).is_ok() {
                buf = buf.trim_end_matches('\n').to_string();
                let parts: Vec<&str> = buf
                    .split(|c: char| c == '\0' || c.is_whitespace())
                    .collect();
                if let Ok(p) = parts[0].parse::<usize>() {
                    pid = Some(p)
                }
            }

            if pid.is_none() {
                return Err(USimpleError::new(
                    EXIT_FAILURE,
                    &format!(
                        "pidfile not valid\nTry `{} --help' for more information.",
                        util_name()
                    ),
                ));
            }
        }

        let pattern_list: Vec<String> = options
            .get_many::<String>(options::PATTERN)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();
        let pattern = match pattern_list.first() {
            Some(p) => p.clone(),
            None => String::new(),
        };

        if pattern_list.len() > 1 {
            return Err(USimpleError::new(
                EXIT_USAGE,
                &format!(
                    "only one pattern can be provided\nTry `{} --help' for more information.",
                    util_name()
                ),
            ));
        }

        if pattern_list.is_empty()
            && (!options.contains_id(options::PIDFILE)
                && !options.contains_id(options::GROUP)
                && !options.contains_id(options::PARENT)
                && !options.contains_id(options::UID)
                && !options.contains_id(options::PGROUP)
                && !options.get_flag(options::NEWEST)
                && !options.get_flag(options::OLDEST)
                && !options.contains_id(options::OLDER)
                && !options.contains_id(options::SESSION)
                && !options.contains_id(options::TERMINAL)
                && !options.contains_id(options::EUID)
                && !options.contains_id(options::RUNSTATES)
                && !options.contains_id(options::NS)
                && !options.contains_id(options::CGROUP)
                && !options.contains_id(options::ENV)
                && !options.get_flag(options::REQUIRE_HANDLER))
        {
            return Err(USimpleError::new(
                EXIT_USAGE,
                &format!(
                    "no matching criteria specified\nTry `{} --help' for more information.",
                    util_name()
                ),
            ));
        }

        Ok(Self {
            delimiter,
            list_name: options.get_flag(options::LIST_NAME),
            list_full: options.get_flag(options::LIST_FULL),
            inverse: options.get_flag(options::INVERSE),
            lightweight: options.get_flag(options::LIGHTWEIGHT),
            count: options.get_flag(options::COUNT),
            full: options.get_flag(options::FULL),
            pgroup,
            group,
            ignore_case: options.contains_id(options::IGNORE_CASE),
            newest: options.get_flag(options::NEWEST),
            oldest: options.get_flag(options::OLDEST),
            older,
            parent,
            session,
            signal,
            terminal,
            euid,
            uid,
            exact: options.get_flag(options::EXACT),
            pid,
            runstates,
            ignore_ancestors: options.get_flag(options::IGNORE_ANCESTORS),
            cgroup,
            ns,
            nslist,
            env,
            pattern,
            require_handler: options.get_flag(options::REQUIRE_HANDLER),
        })
    }

    /// Find group's gid with the given group name.
    fn get_gid_by_name(groupname: &str) -> Option<u32> {
        let groupname = match CString::new(groupname) {
            Ok(g) => g,
            Err(_) => return None,
        };

        let result = unsafe { getgrnam(groupname.as_ptr()) };

        if result.is_null() {
            None
        } else {
            let result = unsafe { &*result };
            Some(result.gr_gid)
        }
    }

    /// Find user's uid with the given user name.
    fn get_uid_by_name(username: &str) -> Option<u32> {
        let username = match CString::new(username) {
            Ok(u) => u,
            Err(_) => return None,
        };

        let result = unsafe { getpwnam(username.as_ptr()) };

        if result.is_null() {
            None
        } else {
            let result = unsafe { &*result };
            Some(result.pw_uid)
        }
    }
}

///
pub fn parse_pgrep_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = pgrep_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn pgrep_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .group(ArgGroup::new("oldest_inverse_newest")
        .args(&[options::OLDEST, options::INVERSE, options::NEWEST]))
        .arg(
            Arg::new(options::DELIMITER)
                .short('d')
                .long(options::DELIMITER)
                .value_name("string")
                .default_value("\n")
                .hide_default_value(true)
                .action(clap::ArgAction::Set)
                .help("specify output delimiter")
                .display_order(10),
        )
        .arg(
            Arg::new(options::LIST_NAME)
                .short('l')
                .long(options::LIST_NAME)
                .action(clap::ArgAction::SetTrue)
                .help("list PID and process name")
                .display_order(20),
        )
        .arg(
            Arg::new(options::LIST_FULL)
                .short('a')
                .long(options::LIST_FULL)
                .action(clap::ArgAction::SetTrue)
                .help("list PID and full command line")
                .display_order(30),
        )
        .arg(
            Arg::new(options::INVERSE)
                .short('v')
                .long(options::INVERSE)
                .action(clap::ArgAction::SetTrue)
                .help("negates the matching")
                .display_order(40),
        )
        .arg(
            Arg::new(options::LIGHTWEIGHT)
                .short('w')
                .long(options::LIGHTWEIGHT)
                .action(clap::ArgAction::SetTrue)
                .help("list all TID")
                .display_order(50),
        )
        .arg(
            Arg::new(options::COUNT)
                .short('c')
                .long(options::COUNT)
                .action(clap::ArgAction::SetTrue)
                .help("count of matching processes")
                .display_order(60),
        )
        .arg(
            Arg::new(options::FULL)
                .short('f')
                .long(options::FULL)
                .action(clap::ArgAction::SetTrue)
                .help("use full process name to match")
                .display_order(70),
        )
        .arg(
            Arg::new(options::PGROUP)
                .short('g')
                .long(options::PGROUP)
                .value_name("PGID")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match listed process group IDs")
                .display_order(80),
        )
        .arg(
            Arg::new(options::GROUP)
                .short('G')
                .long(options::GROUP)
                .value_name("GID")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match real group IDs")
                .display_order(90),
        )
        .arg(
            Arg::new(options::IGNORE_CASE)
                .short('i')
                .long(options::IGNORE_CASE)
                .help("match case insensitively")
                .display_order(100),
        )
        .arg(
            Arg::new(options::NEWEST)
                .short('n')
                .long(options::NEWEST)
                .action(clap::ArgAction::SetTrue)
                .help("select most recently started")
                .display_order(110),
        )
        .arg(
            Arg::new(options::OLDEST)
                .short('o')
                .long(options::OLDEST)
                .action(clap::ArgAction::SetTrue)
                .help("select least recently started")
                .display_order(120),
        )
        .arg(
            Arg::new(options::OLDER)
                .short('O')
                .long(options::OLDER)
                .takes_value(true)
                .value_name("seconds")
                .action(clap::ArgAction::Set)
                .help("select where older than seconds")
                .display_order(130),
        )
        .arg(
            Arg::new(options::PARENT)
                .short('P')
                .long(options::PARENT)
                .value_name("PPID")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match only child processes of the given parent")
                .display_order(140),
        )
        .arg(
            Arg::new(options::SESSION)
                .short('s')
                .long(options::SESSION)
                .value_name("SID")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match session IDs")
                .display_order(150),
        )
        .arg(
            Arg::new(options::SIGNAL)
                .long(options::SIGNAL)
                .takes_value(true)
                .value_name("sig")
                .action(clap::ArgAction::Set)
                .help("signal to send (either number or name)")
                .display_order(160),
        )
        .arg(
            Arg::new(options::TERMINAL)
                .short('t')
                .long(options::TERMINAL)
                .value_name("tty")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match by controlling terminal")
                .display_order(170),
        )
        .arg(
            Arg::new(options::EUID)
                .short('u')
                .long(options::EUID)
                .value_name("ID")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match by effective IDs")
                .display_order(180),
        )
        .arg(
            Arg::new(options::UID)
                .short('U')
                .long(options::UID)
                .value_name("ID")
                .value_delimiter(',')
                .action(clap::ArgAction::Set)
                .help("match by real IDs")
                .display_order(190),
        )
        .arg(
            Arg::new(options::EXACT)
                .short('x')
                .long(options::EXACT)
                .action(clap::ArgAction::SetTrue)
                .help("match exactly with the command name")
                .display_order(200),
        )
        .arg(
            Arg::new(options::PIDFILE)
                .short('F')
                .long(options::PIDFILE)
                .takes_value(true)
                .value_name("file")
                .action(clap::ArgAction::Set)
                .help("read PIDs from file")
                .display_order(210),
        )
        .arg(
            Arg::new(options::LOGPIDFILE)
                .short('L')
                .long(options::LOGPIDFILE)
                .action(clap::ArgAction::SetTrue)
                .help("fail if PID file is not locked")
                .display_order(220),
        )
        .arg(
            Arg::new(options::RUNSTATES)
                .short('r')
                .long(options::RUNSTATES)
                .takes_value(true)
                .value_name("state")
                .action(clap::ArgAction::Set)
                .help("match runstates [D,S,Z,...]")
                .display_order(230),
        )
        .arg(
            Arg::new(options::IGNORE_ANCESTORS)
                .short('A')
                .long(options::IGNORE_ANCESTORS)
                .action(clap::ArgAction::SetTrue)
                .help("exclude our ancestors from results")
                .display_order(240),
        )
        .arg(
            Arg::new(options::CGROUP)
                .long(options::CGROUP)
                .value_name("grp")
                .multiple_values(true)
                .action(clap::ArgAction::Append)
                .help("match by cgroup v2 names")
                .display_order(250),
        )
        .arg(
            Arg::new(options::NS)
                .long(options::NS)
                .takes_value(true)
                .value_name("PID")
                .action(clap::ArgAction::Set)
                .help("match the processes that belong to the same\nnamespace as <pid>")
                .display_order(260),
        )
        .arg(
            Arg::new(options::NSLIST)
                .long(options::NSLIST)
                .value_name("ns")
                .multiple_values(true)
                .action(clap::ArgAction::Append)
                .help("list which namespaces will be considered for\nthe --ns option.\nAvailable namespaces: ipc, mnt, net, pid, user, uts")
                .display_order(270),
        )
        .arg(
            Arg::new(options::ENV)
                .long(options::ENV)
                .value_name("name=val")
                .multiple_values(true)
                .action(clap::ArgAction::Append)
                .help("match on environment variable")
                .display_order(280),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .short_alias('?')
                .long("help")
                .help("display this help and exit")
                .display_order(290),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .help("output version information and exit")
                .display_order(300),
        )
        .arg(
            Arg::new(options::REQUIRE_HANDLER)
                .short('H')
                .long(options::REQUIRE_HANDLER)
                .action(clap::ArgAction::SetTrue)
                .hide(true)
        )
        .arg(Arg::new(options::PATTERN).hide(true).multiple_values(true).action(clap::ArgAction::Append))
}

fn get_ancestors(pid: usize) -> Vec<usize> {
    let mut search_pid = pid;
    let mut done = false;
    let mut result = Vec::new();

    while !done {
        done = true;
        for mut process in walk_process(false) {
            if process.pid == search_pid {
                result.push(process.ppid().unwrap_or_default());
                search_pid = process.ppid().unwrap_or_default();
                done = false;
                break;
            }
        }
    }

    result
}

fn cgroup_match(cgroups_to_find: &[String], process_cgroups_str: String) -> bool {
    // '\n's in process_cgroups_str have been replaced to whitespace.
    let process_cgroups: Vec<&str> = process_cgroups_str.split_whitespace().collect();
    for cgroup_to_find in cgroups_to_find.iter() {
        for process_cgroup in process_cgroups.iter() {
            if process_cgroup.starts_with("0::") && cgroup_to_find.eq(&process_cgroup[3..]) {
                return true;
            }
        }
    }

    false
}

fn environ_match(envs_to_find: &[String], process_envs_str: String) -> bool {
    // '\0's in process_env_str have been replaced to whitespace.
    let process_envs: Vec<&str> = process_envs_str.split_whitespace().collect();
    for env_to_find in envs_to_find.iter() {
        for process_env in process_envs.iter() {
            if env_to_find.contains('=') {
                if env_to_find.eq(process_env) {
                    return true;
                }
            } else if process_env.starts_with(env_to_find) {
                return true;
            }
        }
    }

    false
}

fn signal_handler_match(signal_to_find: i32, process_sigcgt_str: String) -> bool {
    let process_sigcgt_unhex = match u64::from_str_radix(&process_sigcgt_str, 16) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("{}: not a hex string: {}", util_name(), process_sigcgt_str);
            0
        }
    };

    if signal_to_find <= 0 || signal_to_find > 64 {
        false
    } else {
        ((1u64 << (signal_to_find - 1)) & process_sigcgt_unhex) != 0
    }
}

/// Collect pids with filter construct from command line arguments.
fn collect_matched_processes(regex: Regex, config: &Config) -> Vec<ProcessInformation> {
    let pgrep_pid = process::id();
    let mut matched_pid_list = Vec::new();

    let mut saved_start_time = if config.newest {
        0u64 as f64
    } else {
        !0u64 as f64
    };

    let mut saved_pid = if config.newest {
        0
    } else if config.oldest {
        usize::MAX
    } else {
        0
    };

    let ancestors = if config.ignore_ancestors {
        get_ancestors(pgrep_pid as usize)
    } else {
        Vec::new()
    };

    // TODO: ns nslist
    for mut process in walk_process(config.lightweight) {
        let mut matched = true;

        if process.pid == pgrep_pid as usize {
            continue;
        }

        if config.ignore_ancestors && ancestors.contains(&process.pid) {
            matched = false;
        }

        if matched && config.newest && process.starttime().unwrap_or_default() < saved_start_time {
            matched = false;
        }

        if matched && config.oldest && process.starttime().unwrap_or_default() > saved_start_time {
            matched = false;
        }

        if matched
            && !config.parent.is_empty()
            && !config.parent.contains(&process.ppid().unwrap_or_default())
        {
            matched = false;
        }

        if matched && matches!(config.pid, Some(p) if p != process.tgid().unwrap_or_default()) {
            matched = false;
        }

        if matched
            && !config.pgroup.is_empty()
            && !config.pgroup.contains(&process.pgrp().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && !config.euid.is_empty()
            && !config.euid.contains(&process.euid().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && !config.uid.is_empty()
            && !config.uid.contains(&process.ruid().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && !config.group.is_empty()
            && !config.group.contains(&process.rgid().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && !config.session.is_empty()
            && !config
                .session
                .contains(&process.session().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && matches!(config.older, Some(o) if process.elapsed().unwrap_or_default() < o as f64)
        {
            matched = false;
        }

        if matched && !config.terminal.is_empty() && !&config.terminal.contains(&process.ttyname())
        {
            matched = false;
        }

        if matched
            && matches!(config.runstates.clone(), Some(r) if !r.contains(&process.sta().unwrap_or_default()))
        {
            matched = false;
        }

        if matched
            && !config.cgroup.is_empty()
            && !cgroup_match(&config.cgroup, process.cgroup().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && !config.env.is_empty()
            && !environ_match(&config.env, process.environ().unwrap_or_default())
        {
            matched = false;
        }

        if matched
            && config.require_handler
            && !signal_handler_match(config.signal, process.sigcatch().unwrap_or_default())
        {
            matched = false;
        }

        if matched && !config.pattern.is_empty() {
            let cmd_search = if config.full {
                process.cmdline().unwrap_or_default()
            } else {
                process.cmd().unwrap_or_default()
            };

            if regex.find(&cmd_search).is_none() {
                matched = false;
            }
        }

        if (matched && !config.inverse) || (!matched && config.inverse) {
            if config.newest {
                if saved_start_time == process.starttime().unwrap_or_default()
                    && saved_pid > process.pid
                {
                    continue;
                }
                saved_start_time = process.starttime().unwrap_or_default();
                saved_pid = process.pid;
                matched_pid_list.clear();
            }
            if config.oldest {
                if saved_start_time == process.starttime().unwrap_or_default()
                    && saved_pid < process.pid
                {
                    continue;
                }
                saved_start_time = process.starttime().unwrap_or_default();
                saved_pid = process.pid;
                matched_pid_list.clear();
            }
            matched_pid_list.push(process)
        }
    }

    if matched_pid_list.is_empty()
        && !config.full
        && config.pattern.len() > 15
        && !config.pattern.contains('|')
        && !config.pattern.contains('[')
    {
        eprintln!(
            "{}: pattern that searches for process name longer than 15 characters will result in zero matches\nTry `{} -f' option to match against the complete command line.", util_name(), util_name()
        );
    }

    matched_pid_list
}

///
pub fn handle_input(config: Config) -> UResult<()> {
    // Process the pattern.
    let mut pattern = config.pattern.clone();
    if config.exact {
        pattern = format!("^{}$", pattern);
    }
    if config.ignore_case {
        pattern = format!("(?i){}", pattern);
    }
    let regex = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(USimpleError::new(
                EXIT_USAGE,
                &format!("regex error: {}", e),
            ))
        }
    };

    // Select procs.
    let matched_processes = collect_matched_processes(regex, &config);

    // Process output.
    let output = if config.count {
        format!("{}", matched_processes.len())
    } else {
        let formatted: Vec<_> = if config.list_full {
            matched_processes
                .clone()
                .into_iter()
                .map(|mut it| format!("{} {}", it.pid.clone(), it.cmdline().unwrap_or_default()))
                .collect()
        } else if config.list_name {
            matched_processes
                .clone()
                .into_iter()
                .map(|mut it| format!("{} {}", it.pid.clone(), it.cmd().unwrap_or_default()))
                .collect()
        } else {
            matched_processes
                .clone()
                .into_iter()
                .map(|it| format!("{}", it.pid))
                .collect()
        };
        formatted.join(&config.delimiter)
    };

    if !output.is_empty() {
        println!("{}", output);
    };

    if matched_processes.is_empty() {
        Err(EXIT_NO_MATCH.into())
    } else {
        Ok(())
    }
}
