//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::ffi::CString;

use nix::errno::Errno;
use nix::sched::{sched_getaffinity, sched_setaffinity, CpuSet};
use nix::unistd::{getpid, Pid};
use procfs::process::Process;
use uucore::error::{UResult, USimpleError, UUsageError};
use uucore::{format_usage, show_error};

use clap::{crate_version, Arg, ArgMatches, Command};
use libc::EXIT_FAILURE;

use crate::lib_cpuset;

const EX_EXEC_FAILED: i32 = 126; /* Program located, but not usable. */
const EX_EXEC_ENOENT: i32 = 127; /* Could not find program to exec.  */
const ADDITIONAL_HELP: &str = "The default behavior is to run a new command:
    taskset 03 sshd -b 1024
You can retrieve the mask of an existing task:
    taskset -p 700
Or set it:
    taskset -p 03 700
List format uses a comma-separated list instead of a mask:
    taskset -pc 0,3,7-11 700
Ranges in list format can take a stride argument:
    e.g. 0-31:2 is equivalent to mask 0x55555555";

/// Config
pub struct Config {
    /// The process PID to operate
    pub pid: Pid,
    /// Some: The new set we want to apply
    /// None: Just retrieve CPU affinity
    pub new_set: Option<CpuSet>,
    /// The command we want to launch
    pub command: Option<Vec<String>>,
    /// Is --all-tasks option set
    pub all_tasks: bool,
    /// Use CPU list instead of mask
    pub use_list: bool,
}

/// Command Options
pub mod options {
    ///
    pub static ALLTASKS: &str = "all-tasks";
    ///
    pub static CPULIST: &str = "cpu-list";
    ///
    pub static PID: &str = "pid";
    ///
    pub static PID_CMD: &str = "pidcmd";
    ///
    pub static MASK_CPULIST: &str = "mask-cpu-list";
}

impl Config {
    /// Generate taskset general Config
    pub fn from(args_matches: &ArgMatches) -> UResult<Self> {
        let pid_cmd_option: Option<Vec<String>> = args_matches
            .get_many::<String>(options::PID_CMD)
            .map(|v| v.map(String::from).collect());
        let mask_cpulist_option = args_matches.get_one::<String>(options::MASK_CPULIST);
        let pid_set = args_matches.contains_id(options::PID);
        let cpulist_set = args_matches.contains_id(options::CPULIST);
        let mut pid_num = 0;
        let mut command: Option<Vec<String>> = None;
        let mut new_set: Option<CpuSet> = None;
        if pid_cmd_option.is_some() {
            // Process the first positional argument.
            let mask_cpulist = mask_cpulist_option.unwrap();
            new_set = Some(Self::generate_set(mask_cpulist, cpulist_set)?);
            // Process the second positional argument.
            if pid_set {
                let pid_vec = pid_cmd_option.unwrap();
                if pid_vec.len() > 1 {
                    return Err(UUsageError::new(
                        EXIT_FAILURE,
                        "bad Usage: PID option set with too many positional argument.",
                    ));
                }
                pid_num = pid_vec[0].parse::<i32>().unwrap();
            } else {
                command = pid_cmd_option;
            }
        } else if mask_cpulist_option.is_some() && pid_set {
            pid_num = mask_cpulist_option.unwrap().parse::<i32>().unwrap();
        } else {
            return Err(UUsageError::new(EXIT_FAILURE, "bad usage"));
        }

        Ok(Self {
            pid: Pid::from_raw(pid_num),
            new_set, // We omit get_only field by judging if new_set is None
            command,
            all_tasks: args_matches.contains_id(options::ALLTASKS),
            use_list: cpulist_set,
        })
    }

    /// Parse mask or list into CpuSet
    fn generate_set(mask_or_list: &String, cpulist_set: bool) -> UResult<CpuSet> {
        let res = match cpulist_set {
            true => lib_cpuset::cpulist_parse(mask_or_list),
            false => lib_cpuset::cpumask_parse(mask_or_list),
        };
        match res {
            Ok(v) => Ok(v),
            Err(_) => {
                let mask_list_str = match cpulist_set {
                    true => "list",
                    false => "mask",
                };
                Err(USimpleError::new(
                    EXIT_FAILURE,
                    format!("failed to parse CPU {}: {}", mask_list_str, mask_or_list),
                ))
            }
        }
    }
}

/// Get cpu affinity of ts.pid then set a new affinity if need
pub fn do_taskset(ts: &mut Config) -> UResult<()> {
    let pid_is_valid = ts.pid.as_raw() > 0;
    /* read the current mask */
    if pid_is_valid {
        let set = match sched_getaffinity(ts.pid) {
            Ok(v) => v,
            Err(e) => {
                return Err(USimpleError::new(
                    EXIT_FAILURE,
                    err_affinity(ts.pid, false, e),
                ))
            }
        };
        print_affinity(ts, set, false)?;
    }

    if ts.new_set.is_none() {
        return Ok(());
    }

    /* set new mask */
    if let Err(mut e) = sched_setaffinity(ts.pid, ts.new_set.as_ref().unwrap()) {
        if pid_is_valid {
            let pc_stat = Process::new(ts.pid.as_raw()).unwrap().stat().unwrap();
            if e != Errno::EPERM && (pc_stat.flags & libc::PF_NO_SETAFFINITY as u32) > 0 {
                // Based on taskset 2.39.3 logic
                show_error!("affinity cannot be set due to PF_NO_SETAFFINITY flag set");
                e = Errno::EINVAL;
            }
        }
        return Err(USimpleError::new(
            EXIT_FAILURE,
            err_affinity(ts.pid, true, e),
        ));
    }

    /* re-read the current mask */
    if pid_is_valid {
        let set = match sched_getaffinity(ts.pid) {
            Ok(v) => v,
            Err(e) => {
                return Err(USimpleError::new(
                    EXIT_FAILURE,
                    err_affinity(ts.pid, false, e),
                ))
            }
        };
        print_affinity(ts, set, true)?;
    }
    Ok(())
}

/// Parse raw CpuSet into string then print it
fn print_affinity(ts: &Config, set: CpuSet, isnew: bool) -> UResult<()> {
    if ts.use_list {
        let list = lib_cpuset::cpulist_create(set)?;
        if isnew {
            println!("pid {}'s new affinity list: {}", ts.pid, list);
        } else {
            println!("pid {}'s current affinity list: {}", ts.pid, list);
        }
    } else {
        let mask = lib_cpuset::cpumask_create(set)?;
        if isnew {
            println!("pid {}'s new affinity mask: {}", ts.pid, mask);
        } else {
            println!("pid {}'s current affinity mask: {}", ts.pid, mask);
        }
    }
    Ok(())
}

/// Called if an error produced, format error string
fn err_affinity(mut pid: Pid, is_set: bool, err: Errno) -> String {
    if pid.as_raw() == 0 {
        pid = getpid();
    };
    let mut errmsg = format!("{}", err);
    // Remove leading extra characters to keep consistent of strerror()
    errmsg.replace_range(..errmsg.find(' ').unwrap(), "");
    match is_set {
        true => format!("failed to set pid {}'s affinity:{}", pid, errmsg),
        false => format!("failed to get pid {}'s affinity:{}", pid, errmsg),
    }
}

/// Generate taskset general Config
pub fn parse_taskset_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
) -> UResult<Config> {
    let command = taskset_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

/// Execute a new program
pub fn run_program(cmd_args: Vec<String>) -> UResult<()> {
    let args: Vec<CString> = cmd_args
        .iter()
        .map(|s| CString::new(s.as_bytes()).unwrap())
        .collect();
    let res = nix::unistd::execvp(&args[0], &args);

    match res {
        Ok(_res) => Ok(()),
        Err(e) => Err(USimpleError::new(
            match e {
                Errno::ENOENT => EX_EXEC_ENOENT,
                _ => EX_EXEC_FAILED,
            },
            format!("failed to execute {}", cmd_args[0]),
        )),
    }
}

/// Command arguments setting
pub fn taskset_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg_required_else_help(true)
        // Format arguments.
        .arg(
            Arg::new(options::ALLTASKS)
                .short('a')
                .long(options::ALLTASKS)
                .help("operate on all the tasks (threads) for a given pid"),
        )
        .arg(
            Arg::new(options::PID)
                .short('p')
                .long(options::PID)
                .help("operate on existing given pid"),
        )
        .arg(
            Arg::new(options::CPULIST)
                .short('c')
                .long(options::CPULIST)
                .help("display and specify cpus in list format"),
        )
        .after_help(ADDITIONAL_HELP)
        .arg(Arg::new(options::MASK_CPULIST).index(1).hide(true))
        .arg(
            Arg::new(options::PID_CMD)
                .index(2)
                .multiple_values(true)
                .hide(true),
        )
        .trailing_var_arg(true)
}
