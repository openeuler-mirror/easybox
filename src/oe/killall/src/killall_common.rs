//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use nix::errno::Errno;
use nix::sys::signal::{kill, killpg, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{getpgid, Pid, User};
use regex::Regex;
use std::fs::{self, read_dir, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{process, thread};
use uucore::error::{UResult, USimpleError, UUsageError};
use uucore::format_usage;
const PROC_BASE: &str = "/proc/";
const TASK_COMM_LEN: usize = 16;
///
pub static KILLALL_CMD_PARSE_ERROR: i32 = 1;
///
pub struct Config {
    ///
    pub exact: bool,
    ///
    pub ignore_case: bool,
    ///
    pub process_group: bool,
    ///
    pub younger_than: u64,
    ///
    pub older_than: u64,
    ///
    pub interactive: bool,
    ///
    pub list: bool,
    ///
    pub quiet: bool,
    ///
    pub regexp: bool,
    ///
    pub signal: Signal,
    ///
    pub user: Option<u32>,
    ///
    pub verbose: bool,
    ///
    pub wait: bool,
    ///
    pub ns: i64,
    ///
    pub context: Option<String>,
    ///
    pub name: Vec<String>,
}

/// options.
///
pub mod options {
    ///
    pub static EXACT: &str = "exact";
    ///
    pub static IGNORE_CASE: &str = "ignore-case";
    ///
    pub static PROCESS_GROUP: &str = "process-group";
    ///
    pub static YOUNGER_THAN: &str = "younger-than";
    ///
    pub static OLDER_THAN: &str = "older-than";
    ///
    pub static INTERACTIVE: &str = "interactive";
    ///
    pub static LIST: &str = "list";
    ///
    pub static QUIET: &str = "quiet";
    ///
    pub static REGEXP: &str = "regexp";
    ///
    pub static SIGNAL: &str = "signal";
    ///
    pub static USER: &str = "user";
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static WAIT: &str = "wait";
    ///
    pub static CONTEXT: &str = "context";
    ///
    pub static NS: &str = "ns";
    ///
    pub static NAME: &str = "name";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let mut younger_than = 0;
        let mut older_than = 0;
        if options.is_present(options::YOUNGER_THAN) {
            let younger_str = options.value_of(options::YOUNGER_THAN).unwrap_or("");
            match parse_time_units(younger_str) {
                Ok(younger) => younger_than = younger,
                Err(e) => return Err(UUsageError::new(KILLALL_CMD_PARSE_ERROR, e)),
            }
        }
        if options.is_present(options::OLDER_THAN) {
            let older_str = options.value_of(options::OLDER_THAN).unwrap_or("");
            match parse_time_units(older_str) {
                Ok(older) => older_than = older,
                Err(e) => return Err(UUsageError::new(KILLALL_CMD_PARSE_ERROR, e)),
            }
        }
        if !options.is_present(options::LIST) && !options.is_present(options::NAME) {
            return Err(UUsageError::new(
                KILLALL_CMD_PARSE_ERROR,
                "invalid arguments",
            ));
        }
        let name_args: Vec<String> = options
            .get_many::<String>(options::NAME)
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect();
        let username;
        if options.is_present(options::USER) {
            let user_str = options.value_of(options::USER).unwrap_or("").to_string();
            match get_user_uid(&user_str) {
                Some(uid) => username = Some(uid),
                None => {
                    return Err(UUsageError::new(
                        KILLALL_CMD_PARSE_ERROR,
                        format!("Cannot find user {}", user_str),
                    ))
                }
            }
        } else {
            username = None;
        }
        let context;
        if options.is_present(options::CONTEXT) {
            let pattern = options.value_of(options::CONTEXT).unwrap_or("");
            if !Regex::new(pattern).is_ok() {
                return Err(UUsageError::new(
                    KILLALL_CMD_PARSE_ERROR,
                    format!("Invalid context pattern {}", pattern),
                ));
            }
            context = Some(pattern.to_string());
        } else {
            context = None;
        }
        Ok(Self {
            exact: options.is_present(options::EXACT),
            ignore_case: options.is_present(options::IGNORE_CASE),
            process_group: options.is_present(options::PROCESS_GROUP),
            younger_than: younger_than,
            older_than: older_than,
            interactive: options.is_present(options::INTERACTIVE),
            list: options.is_present(options::LIST),
            quiet: options.is_present(options::QUIET),
            regexp: options.is_present(options::REGEXP),
            signal: get_signal(options.value_of(options::SIGNAL).unwrap_or("TERM")),
            user: username,
            verbose: options.is_present(options::VERBOSE),
            wait: options.is_present(options::WAIT),
            ns: options
                .value_of(options::NS)
                .unwrap_or("")
                .parse()
                .unwrap_or(-1),
            context: context,
            name: name_args,
        })
    }
}

///
pub fn parse_base_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = killall_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn killall_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::EXACT)
                .short('e')
                .long(options::EXACT)
                .help("require exact match for very long names"),
        )
        .arg(
            Arg::new(options::IGNORE_CASE)
                .short('I')
                .long(options::IGNORE_CASE)
                .help("case insensitive process name match"),
        )
        .arg(
            Arg::new(options::PROCESS_GROUP)
                .short('g')
                .long(options::PROCESS_GROUP)
                .help("kill process group instead of process"),
        )
        .arg(
            Arg::new(options::YOUNGER_THAN)
                .short('y')
                .long(options::YOUNGER_THAN)
                .value_name("TIME")
                .takes_value(true)
                .help("kill processes younger than TIME"),
        )
        .arg(
            Arg::new(options::OLDER_THAN)
                .short('o')
                .long(options::OLDER_THAN)
                .value_name("TIME")
                .takes_value(true)
                .help("kill processes older than TIME"),
        )
        .arg(
            Arg::new(options::INTERACTIVE)
                .short('i')
                .long(options::INTERACTIVE)
                .help("ask for confirmation before killing"),
        )
        .arg(
            Arg::new(options::LIST)
                .short('l')
                .long(options::LIST)
                .help("list all known signal names"),
        )
        .arg(
            Arg::new(options::QUIET)
                .short('q')
                .long(options::QUIET)
                .help("don't print complaints"),
        )
        .arg(
            Arg::new(options::REGEXP)
                .short('r')
                .long(options::REGEXP)
                .help("interpret NAME as an extended regular expression"),
        )
        .arg(
            Arg::new(options::SIGNAL)
                .short('s')
                .long(options::SIGNAL)
                .value_name("SIGNAL")
                .takes_value(true)
                .help("send this signal instead of SIGTERM"),
        )
        .arg(
            Arg::new(options::USER)
                .short('u')
                .long(options::USER)
                .value_name("USER")
                .takes_value(true)
                .help("kill only process(es) running as USER"),
        )
        .arg(
            Arg::new(options::VERBOSE)
                .short('v')
                .long(options::VERBOSE)
                .help("report if the signal was successfully sent"),
        )
        .arg(
            Arg::new(options::WAIT)
                .short('w')
                .long(options::WAIT)
                .help("wait for processes to die"),
        )
        .arg(
            Arg::new(options::NS)
                .short('n')
                .long(options::NS)
                .value_name("NAMESPACE")
                .takes_value(true)
                .help("match processes that belong to the same namespaces\nas PID"),
        )
        .arg(
            Arg::new(options::CONTEXT)
                .short('Z')
                .long(options::CONTEXT)
                .value_name("CONTEXT")
                .takes_value(true)
                .help("kill only process(es) having context\n(must precede other arguments)"),
        )
        .arg(Arg::new(options::NAME).index(1).multiple_occurrences(true))
}

/// parse time string to seconds
///
pub fn parse_time_units(age: &str) -> Result<u64, &'static str> {
    if age.is_empty() {
        return Err("Input is empty");
    }

    let (num_str, unit) = age.split_at(age.len() - 1);
    let num: u64 = match num_str.parse() {
        Ok(n) => n,
        Err(_) => return Err("Failed to parse number"),
    };

    match unit.chars().next() {
        Some('s') => Ok(num),
        Some('m') => Ok(num * 60),
        Some('h') => Ok(num * 60 * 60),
        Some('d') => Ok(num * 60 * 60 * 24),
        Some('w') => Ok(num * 60 * 60 * 24 * 7),
        Some('M') => Ok(num * 60 * 60 * 24 * 7 * 4),
        Some('y') => Ok(num * 60 * 60 * 24 * 7 * 4 * 12),
        _ => Err("Invalid time unit"),
    }
}

/// handle input
///
pub fn handle_input(config: &Config) -> UResult<()> {
    if config.list {
        list_signals();
        return Ok(());
    }
    let ret_code = kill_all(config);
    if ret_code != 0 {
        return Err(USimpleError::new(ret_code, ""));
    }
    Ok(())
}

/// list all signals
///
pub fn list_signals() {
    let mut col = 0;
    for sig in Signal::iterator() {
        let mut name = sig.to_string()[3..].to_string();
        // SIGIO and SIGPOLL are the same in Linux, but killall prints POLL
        if name == "IO" {
            name = "POLL".to_string();
        }
        if col + name.len() + 1 > 80 {
            println!();
            col = 0;
        }
        if col != 0 {
            print!(" ");
        }
        print!("{}", name);
        col += name.len() + 1;
    }
    println!();
}

///
pub fn get_signal(signal: &str) -> Signal {
    let sig_num;
    if let Ok(signal_num) = signal.parse::<i32>() {
        sig_num = signal_num;
    } else {
        sig_num = -1;
    }
    for sig in Signal::iterator() {
        if sig.to_string()[3..].to_string() == signal {
            return sig;
        }
        if sig.to_string() == signal {
            return sig;
        }
        if sig_num < 0 {
            continue;
        }
        if sig as i32 == sig_num {
            return sig;
        }
    }
    return Signal::SIGTERM;
}

///
pub fn kill_all(c: &Config) -> i32 {
    let de = read_dir(PROC_BASE);
    if de.is_err() {
        return 1;
    }
    let de = de.unwrap();
    let mut pid_table: Vec<u32> = Vec::new();
    let mut found: u64 = 0;
    let mut p_killed = 0;
    let mut pid_killed: Vec<Pid> = Vec::new();
    let mut err_ret = 0;
    for entry in de {
        let pid = entry
            .unwrap()
            .file_name()
            .into_string()
            .unwrap_or("".to_string());
        if let Ok(pid) = pid.parse::<u32>() {
            if pid == process::id() {
                continue;
            }
            pid_table.push(pid);
        }
    }
    let mut pgids = pid_table.clone();
    for i in 0..pid_table.len() {
        let pid = pid_table[i];
        if c.user.is_some() && !match_process_uid(pid, c.user.unwrap()) {
            continue;
        }
        if c.ns > 0 {
            let namespaces = ["pid", "net", "mnt", "uts", "ipc", "cgroup"];
            let mut ns_same = true;
            for ns in namespaces {
                if read_namespace_link(pid, ns) != read_namespace_link(c.ns as u32, ns) {
                    ns_same = false;
                    break;
                }
            }
            if !ns_same {
                continue;
            }
        }
        let path = format!("{}/{}/stat", PROC_BASE, pid);
        let file_path = std::path::Path::new(&path);
        let comm;
        let mut process_age_sec = -1.0;
        let mut get_long = 0;
        let mut command = "".to_string();
        if let Ok(mut file) = File::open(file_path) {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            comm = {
                let start = content.find('(').unwrap_or(0);
                let end = content[start..].find(')').unwrap_or(0);
                content[start + 1..start + end].to_string()
            };
            if comm.is_empty() {
                continue;
            }
            if c.younger_than > 0 || c.older_than > 0 {
                let process_stt_jf = {
                    let fields: Vec<&str> = content.split_whitespace().collect();
                    if fields.len() < 22 {
                        continue;
                    }
                    let proc_stt_jf: i64 = fields[21].parse().unwrap_or(-1);
                    proc_stt_jf
                };
                if process_stt_jf < 0 {
                    continue;
                } else {
                    process_age_sec = process_age(process_stt_jf as u64).unwrap();
                }
            }
            if comm.len() >= TASK_COMM_LEN || c.exact {
                command = read_cmdline(pid).unwrap_or("".to_string());
                if command.is_empty() {
                    continue;
                } else {
                    get_long = 1;
                }
            }
            if c.exact && get_long == 0 {
                continue;
            }
        } else {
            continue;
        }
        let mut found_name = -1;
        for j in 0..c.name.len() {
            let p2kill = &c.name[j];
            if c.regexp {
                let re = match Regex::new(p2kill) {
                    Ok(regex) => regex,
                    Err(err) => {
                        eprintln!("Failed to compile regex: {}", err);
                        return 1;
                    }
                };
                if (get_long == 1 && !re.is_match(&command))
                    || (get_long == 0 && !re.is_match(&comm))
                {
                    continue;
                }
            } else {
                if c.younger_than > 0 && process_age_sec > c.younger_than as f64 {
                    continue;
                }
                if c.older_than > 0
                    && process_age_sec > 0.0
                    && process_age_sec < c.older_than as f64
                {
                    continue;
                }
                if c.ignore_case {
                    if get_long == 1 && !str::eq_ignore_ascii_case(p2kill, &command) {
                        continue;
                    } else if get_long == 0 && !str::eq_ignore_ascii_case(p2kill, &comm) {
                        continue;
                    }
                } else {
                    if get_long == 1 && p2kill != &command {
                        continue;
                    } else if get_long == 0 && !p2kill.eq(&comm) {
                        continue;
                    }
                }
            }
            found_name = j as isize;
            break;
        }
        if found_name < 0 {
            continue;
        }
        let idx = found_name as usize;
        let id;
        if !c.process_group {
            id = Pid::from_raw(pid as i32);
        } else {
            id = getpgid(Some(Pid::from_raw(pid as i32))).unwrap();
            let id_u32 = id.as_raw() as u32;
            pgids[i] = id_u32;
            let mut index = 0;
            for j in 0..i {
                if pgids[j] == id_u32 {
                    break;
                }
                index = j + 1;
            }
            if index < i {
                continue;
            }
        }
        if c.interactive && !ask(&c.name[idx], pid, c.signal, c) {
            continue;
        }
        if let Err(err) = oe_kill(c.process_group, id, c.signal) {
            if err != Errno::ESRCH || c.interactive {
                let process_name = if get_long == 1 { command } else { comm };
                eprintln!("{}({}): {}", process_name, id.as_raw(), err);
            }
        } else {
            if c.verbose {
                let process_name = if get_long == 1 { command } else { comm };
                let group_info = if c.process_group { "pgid" } else { "" };
                let signal_info = c.signal as i32;

                if group_info.is_empty() {
                    eprintln!(
                        "Killed {}({}) with signal {}",
                        process_name,
                        id.as_raw(),
                        signal_info
                    );
                } else {
                    eprintln!(
                        "Killed {}({}{}) with signal {}",
                        process_name,
                        group_info,
                        id.as_raw(),
                        signal_info
                    );
                }
            }
            if found_name >= 0 {
                found |= 1 << found_name;
                p_killed += 1;
                pid_killed.push(id);
            }
        }
    }

    if !c.quiet {
        for i in 0..c.name.len() {
            if (found & (1 << i)) == 0 {
                eprintln!("{}: no process found", &c.name[i]);
            }
        }
    }
    if c.name.len() > 0 {
        if p_killed <= 0 {
            err_ret = 1;
        }
    }
    while p_killed > 0 && c.wait {
        for i in 0..p_killed {
            if !check_process_exists(pid_killed[i]) {
                p_killed -= 1;
                pid_killed[i] = pid_killed[p_killed];
                continue;
            } else {
                println!("{} is still running", pid_killed[i].as_raw());
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
    return err_ret;
}

fn match_process_uid(pid: u32, uid: u32) -> bool {
    let path = format!("{}/{}/status", PROC_BASE, pid);

    let f = File::open(&path);
    let reader;
    if f.is_err() {
        return false;
    } else {
        reader = BufReader::new(f.unwrap());
    }

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("Uid:\t") {
            let mut parts = line.split_whitespace();
            parts.next(); // Skip "Uid:"
            if let Some(puid_str) = parts.next() {
                if let Ok(puid) = puid_str.parse::<u32>() {
                    return uid == puid;
                }
            }
        }
    }

    eprintln!("Cannot get UID from process status");
    return false;
}

fn read_namespace_link(pid: u32, namespace: &str) -> Option<String> {
    let path = PathBuf::from(format!("/proc/{}/ns/{}", pid, namespace));
    fs::read_link(path)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

fn get_user_uid(username: &str) -> Option<u32> {
    match User::from_name(username) {
        Ok(Some(user)) => Some(user.uid.as_raw()),
        Ok(None) => None,
        Err(err) => {
            eprintln!("Error fetching user information: {}", err);
            None
        }
    }
}

/// Read the/proc/uptime file to obtain the system runtime
fn uptime() -> Result<f64, Box<dyn std::error::Error>> {
    let file = File::open("/proc/uptime")?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let uptime: f64 = content
        .split_whitespace()
        .next()
        .ok_or("Failed to read uptime")?
        .parse()?;
    Ok(uptime)
}

/// Reading the/proc/stat file to obtain the system clock ticks
fn clk_tck() -> Result<f64, Box<dyn std::error::Error>> {
    let file = File::open("/proc/stat")?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    for line in content.lines() {
        if line.starts_with("btime") {
            return Ok(100.0);
        }
    }

    Err("Failed to read clock tick from /proc/stat".into())
}

/// Calculate the age of the process
fn process_age(jf: u64) -> Result<f64, Box<dyn std::error::Error>> {
    let sc_clk_tck = clk_tck()?;
    if sc_clk_tck <= 0.0 {
        return Err("Invalid _SC_CLK_TCK value".into());
    }
    let uptime = uptime()?;
    Ok(uptime - (jf as f64 / sc_clk_tck))
}

fn read_cmdline(pid: u32) -> io::Result<String> {
    let path = format!("/proc/{}/cmdline", pid);
    let path = Path::new(&path);

    let mut file = File::open(&path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let cmdline = buffer
        .iter()
        .map(|&c| if c == 0 { b' ' } else { c })
        .collect::<Vec<u8>>();

    let cmdline_str = String::from_utf8_lossy(&cmdline).into_owned();

    Ok(cmdline_str.trim_end().to_string())
}

fn ask(name: &str, pid: u32, signal: Signal, c: &Config) -> bool {
    loop {
        let prompt = if signal == Signal::SIGTERM {
            format!(
                "Kill {}({}{}) ? (y/N) ",
                name,
                if c.process_group { "pgid " } else { "" },
                pid
            )
        } else {
            format!(
                "Signal {}({}{}) ? (y/N) ",
                name,
                if c.process_group { "pgid " } else { "" },
                pid
            )
        };

        print!("{}", prompt);
        io::stdout().flush().expect("Failed to flush stdout");

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        // Check for default (empty line)
        if line.trim().is_empty() {
            return false;
        }

        let response = line.trim().to_lowercase();

        match response.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                println!("Invalid response. Please enter 'y' or 'n'.");
                continue;
            }
        }
    }
}

fn oe_kill(process_group: bool, id: Pid, signal: Signal) -> Result<(), nix::Error> {
    let result = if process_group {
        killpg(id, signal)
    } else {
        kill(id, signal)
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn check_process_exists(pid: Pid) -> bool {
    match waitpid(pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => true,
        Ok(_) => false,
        Err(_) => false,
    }
}
