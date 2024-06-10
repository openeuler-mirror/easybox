//! This file is part of the uutils coreutils package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use std::fs::{read_link, File};
use std::io::Read;
use std::path::Path;
use std::process;
use std::{cell::RefCell, fs};
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;

const PROC_BASE: &str = "/proc";
const CMDLINE_NAME: &str = "cmdline";
const STAT_NAME: &str = "stat";
const EXE_NAME: &str = "exe";
const ROOT_NAME: &str = "root";
const TASK_NAME: &str = "task";
const NOT_FOUND_EXIT_CODE: i32 = 1;
const FOUND_EXIT_CODE: i32 = 0;

/// Config
#[derive(Debug)]
pub struct Config {
    ///
    pub single_shot: bool,
    ///
    pub check_root: bool,
    ///
    pub quiet: bool,
    ///
    pub with_workers: bool,
    ///
    pub scripts_too: bool,
    ///
    pub lightweight: bool,
    ///
    pub omit_pid: Vec<i32>,
    ///
    pub separator: String,
    ///
    pub program: Option<Vec<String>>,
    ///
    pub procs: RefCell<Vec<Vec<i32>>>,
    ///
    pub root_link: Option<String>,
}

/// Options.
pub mod options {
    ///
    pub static SINGLE_SHOT: &str = "single-shot";
    ///
    pub static CHECK_ROOT: &str = "check-root";
    ///
    pub static WITH_WORKERS: &str = "with-workers";
    ///
    pub static OMIT_PID: &str = "omit-pid";
    ///
    pub static SEPARATOR: &str = "separator";
    ///
    pub static QUIET: &str = "quiet";
    ///
    pub static LIGHTWEIGHT: &str = "lightweight";
    ///
    pub static SCRIPTS_TOO: &str = "scripts-too";
    ///
    pub static PROGRAM: &str = "program";
}

impl Config {
    ///
    pub fn from(args_matches: &clap::ArgMatches) -> UResult<Self> {
        //
        let mut omit: Vec<i32> = vec![];
        let omit_sets: Option<Vec<String>> = args_matches
            .get_many::<String>(options::OMIT_PID)
            .map(|v| v.map(String::from).collect());
        if !omit_sets.is_none() {
            for omit_set in omit_sets.unwrap() {
                omit.extend(omit_set.split(',').filter_map(|s| s.parse::<i32>().ok()));
            }
        }
        // set separator
        let sep: Option<String> =
            if let Some(seps) = args_matches.get_many::<String>(options::SEPARATOR) {
                seps.map(|v| v.to_owned()).last()
            } else {
                Some(" ".to_string())
            };
        let program_list: Option<Vec<String>> = args_matches
            .get_many::<String>(options::PROGRAM)
            .map(|v| v.map(String::from).collect());
        let program_cnt = program_list.as_ref().map_or_else(|| 0, |vec| vec.len());
        let root_link = pid_link(process::id() as i32, ROOT_NAME);
        Ok(Self {
            single_shot: args_matches.contains_id(options::SINGLE_SHOT),
            check_root: args_matches.contains_id(options::CHECK_ROOT),
            quiet: args_matches.contains_id(options::QUIET),
            with_workers: args_matches.contains_id(options::WITH_WORKERS),
            scripts_too: args_matches.contains_id(options::SCRIPTS_TOO),
            lightweight: args_matches.contains_id(options::LIGHTWEIGHT),
            omit_pid: omit,
            separator: sep.unwrap(),
            program: program_list,
            procs: RefCell::new(vec![Vec::new(); program_cnt]),
            root_link,
        })
    }
}

///
pub fn parse_pidof_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = pidof_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn pidof_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        // Format arguments.
        .arg(
            Arg::new(options::SINGLE_SHOT)
                .short('s')
                .long(options::SINGLE_SHOT)
                .multiple_occurrences(true)
                .help("return one PID only"),
        )
        .arg(
            Arg::new(options::CHECK_ROOT)
                .short('c')
                .long(options::CHECK_ROOT)
                .multiple_occurrences(true)
                .help("omit processes with different root"),
        )
        .arg(
            Arg::new(options::QUIET)
                .short('q')
                .multiple_occurrences(true)
                .help("quiet mode, only set the exit code"),
        )
        .arg(
            Arg::new(options::WITH_WORKERS)
                .short('w')
                .long(options::WITH_WORKERS)
                .multiple_occurrences(true)
                .help("show kernel workers too"),
        )
        .arg(
            Arg::new(options::SCRIPTS_TOO)
                .short('x')
                .multiple_occurrences(true)
                .help("also find shells running the named scripts"),
        )
        .arg(
            Arg::new(options::OMIT_PID)
                .short('o')
                .long(options::OMIT_PID)
                .multiple(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .takes_value(true)
                .help("omit processes with PID"),
        )
        .arg(
            Arg::new(options::LIGHTWEIGHT)
                .short('t')
                .long(options::LIGHTWEIGHT)
                .multiple_occurrences(true)
                .help("list threads too"),
        )
        .arg(
            Arg::new(options::SEPARATOR)
                .short('S')
                .long(options::SEPARATOR)
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .help("use SEP as separator put between PIDs"),
        )
        .arg(Arg::new(options::PROGRAM).multiple(true))
}

///
pub fn handle_input(config: Config) -> UResult<i32> {
    match select_proc(&config) {
        Ok(_) => Ok(print_procs(&config)),
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}

#[derive(Debug)]
///
pub struct Proc {
    /// if kernel worder, this is empty
    cmdline: Vec<String>,
    ///
    command: String,
    exe_link: Option<String>,
    tids: Vec<i32>,
}

/// get base path name of a file
pub fn get_basename(filename: &str) -> Option<&str> {
    let path = Path::new(filename);
    path.file_name()?.to_str()
}

/// Select matched process
pub fn select_proc(config: &Config) -> UResult<()> {
    let proc_dir = match fs::read_dir(PROC_BASE) {
        Ok(dir) => dir,
        Err(e) => {
            return Err(USimpleError::new(1, format!("{}", e)));
        }
    };
    for de in proc_dir {
        let entry = de?;
        let path = entry.path();
        let file_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if path.is_dir() {
            if let Ok(pid) = file_name.parse::<i32>() {
                // read stat file
                let stat_path = path.join(STAT_NAME);
                let mut stat_file = match File::open(stat_path) {
                    Ok(file) => file,
                    _ => continue,
                };
                let mut readbuf = String::new();
                let _size = match stat_file.read_to_string(&mut readbuf) {
                    Ok(size) => size,
                    _ => continue,
                };
                //find command between()
                let command = match (readbuf.find('('), readbuf.find(')')) {
                    (Some(start), Some(end)) => readbuf[start + 1..end].to_string(),
                    _ => continue,
                };
                // read cmdline file
                let cmdline_path = path.join(CMDLINE_NAME);
                let mut cmdline_file = match File::open(cmdline_path) {
                    Ok(file) => file,
                    _ => continue,
                };
                let mut cmdline = String::new();
                let _size = match cmdline_file.read_to_string(&mut cmdline) {
                    Ok(size) => size,
                    _ => continue,
                };
                let cmdline_argv: Vec<String> = cmdline
                    .split('\0')
                    .map(String::from)
                    .filter(|s| s.len() > 0)
                    .collect();

                // read exe
                let exe_link = pid_link(pid, EXE_NAME);
                let mut tids = vec![pid];
                // get thread id
                if config.lightweight {
                    let task_path = path.join(TASK_NAME);
                    let task_dir = fs::read_dir(task_path);
                    if let Ok(task_dir) = task_dir {
                        for task_de in task_dir {
                            let task_entry = task_de?;
                            if let Ok(tid) =
                                task_entry.file_name().into_string().unwrap().parse::<i32>()
                            {
                                if !tids.contains(&tid) {
                                    tids.push(tid);
                                }
                            }
                        }
                    }
                }
                let proc = Proc {
                    cmdline: cmdline_argv,
                    command,
                    exe_link,
                    tids,
                };
                try_add_proc(&proc, config)?
            }
        }
    }
    Ok(())
}

/// Check if process match with program name
pub fn proc_match(program: &str, proc: &Proc, config: &Config) -> bool {
    let mut cmd_arg0: &str = match proc.cmdline.get(0) {
        Some(cmd) => cmd,
        _ => "",
    };
    // processes starting with '-' are login shells
    if cmd_arg0.len() > 0 && matches!(cmd_arg0.get(0..1), Some("-")) {
        cmd_arg0 = cmd_arg0.get(1..).unwrap();
    }
    // get executable file name
    let exe_link = proc.exe_link.clone().unwrap_or(String::new());
    let exe_link_base = get_basename(&exe_link).unwrap_or("");
    // get base name
    let cmd_arg0base = get_basename(cmd_arg0).unwrap_or("");
    let program_base = get_basename(program).unwrap_or("");
    if program == cmd_arg0base
        || program_base == cmd_arg0
        || program == cmd_arg0
        || (config.with_workers && program == &proc.command)
        || program == exe_link_base
        || program == &exe_link
    {
        return true;
    } else if config.scripts_too && proc.cmdline.len() > 1 {
        let cmd_arg1 = proc.cmdline.get(1).unwrap();
        let cmd_arg1base = get_basename(cmd_arg1).unwrap();
        if proc.command.len() > 0
            && proc.command == cmd_arg1base
            && (program == cmd_arg1base || program_base == cmd_arg1 || program == cmd_arg1)
        {
            return true;
        }
    }

    if let Some(loc) = cmd_arg0.find(' ') {
        if program == &cmd_arg0[loc..] {
            return true;
        }
    }

    return false;
}

/// Try add process to output list
pub fn try_add_proc(proc: &Proc, config: &Config) -> UResult<()> {
    if let Some(ref programs) = &config.program {
        for (index, program) in programs.iter().enumerate() {
            if config.check_root {
                let root_link = pid_link(proc.tids[0], ROOT_NAME);
                if !root_link.eq(&config.root_link) {
                    return Ok(());
                }
            }
            // if kernel process, cmdline.len() == 0
            if (proc.cmdline.len() > 0 || config.with_workers) && proc_match(program, proc, config)
            {
                for tid in &proc.tids {
                    if config.omit_pid.contains(tid) {
                        continue;
                    }
                    config.procs.borrow_mut()[index].push(*tid);
                }
            }
        }
    }
    Ok(())
}

/// Get the filename which the link point to
pub fn pid_link(pid: i32, base_name: &str) -> Option<String> {
    let path_filename = format!("/proc/{}/{}", pid, base_name);
    match read_link(path_filename) {
        Ok(file) => Some(file.into_os_string().into_string().unwrap()),
        _ => None,
    }
}

/// Print pid list
pub fn print_procs(config: &Config) -> i32 {
    let procs = config.procs.borrow();
    let mut first = true;
    for i in 0..procs.len() {
        for proc in procs[i].iter().rev() {
            if first {
                if config.quiet {
                    return FOUND_EXIT_CODE;
                }
                print!("{}", proc);
                first = false;
            } else {
                print!("{}{}", config.separator, proc);
            }
            if config.single_shot {
                break;
            }
        }
    }
    if procs.iter().all(|p| p.is_empty()) {
        NOT_FOUND_EXIT_CODE
    } else {
        println!();
        FOUND_EXIT_CODE
    }
}
