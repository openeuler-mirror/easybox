//! This file is part of the uutils coreutils package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use chrono::{DateTime, Datelike, NaiveDate};
use clap::{crate_version, Arg, Command};
use libc::{strerror, EXIT_SUCCESS};
use nix::fcntl::{fcntl, FcntlArg};
use nix::unistd::{chdir, chroot};
use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io::{stdin, stdout, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use users::switch::{set_both_gid, set_both_uid};
use users::{get_current_gid, get_current_uid};
use uucore::error::{UResult, USimpleError};
use uucore::{format_usage, help_section};
const E_NOPERM: i32 = 1;
const E_USAGE: i32 = 2;
const E_BAD_ARG: i32 = 3;
const E_PASSWD_NOTFOUND: i32 = 14;
const E_SHADOW_NOTFOUND: i32 = 15;

const UNIX_EPOCH_DAYS: i64 = 719163;
const DAY: i64 = 24 * 60 * 60;

const SHADOW_FILE: &str = "/etc/shadow";
const PASSWD_FILE: &str = "/etc/passwd";

const SHADOW_LOCK: &str = "/etc/shadow.lock";
const PASSWD_LOCK: &str = "/etc/passwd.lock";

const SHADOW_PASSWD_STRING: &str = "x";
const OPTIONS: &str = help_section!("Options", "chage.md");

const LOCK_TRIES: u32 = 15;
const LOCK_SLEEP: u64 = 1;

/// Config
#[derive(Debug)]
pub struct Config {
    ///
    pub last_day: i64,
    ///
    pub expire_date: i64,
    ///
    pub iso8601: bool,
    ///
    pub inactive_days: i64,
    ///
    pub list: bool,
    ///
    pub min_days: i64,
    ///
    pub max_days: i64,
    ///
    pub root: String,
    ///
    pub warn_days: i64,
    ///
    pub login: String,
    ///
    pub ruid: u32,
    ///
    pub rgid: u32,
    ///
    pub isroot: bool,
    ///
    pub nochange: bool,
    ///
    pub lstday_present: bool,
    ///
    pub expdate_present: bool,
    ///
    pub inact_present: bool,
    ///
    pub mindays_present: bool,
    ///
    pub maxdays_present: bool,
    ///
    pub warndays_present: bool,
}

/// Options.
pub mod options {
    ///
    pub static LAST_DAY: &str = "lastday";
    ///
    pub static EXPIRE_DATE: &str = "expiredate";
    ///
    pub static ISO8601: &str = "iso8601";
    ///
    pub static INACTIVE: &str = "inactive";
    ///
    pub static LIST: &str = "list";
    ///
    pub static MIN_DAYS: &str = "mindays";
    ///
    pub static MAX_DAYS: &str = "maxdays";
    ///
    pub static ROOT: &str = "root";
    ///
    pub static WARN_DAYS: &str = "warndays";
    ///
    pub static LOGIN: &str = "login";
}

impl Config {
    ///
    pub fn from(args_matches: &clap::ArgMatches) -> UResult<Config> {
        let last_day = args_matches
            .value_of(options::LAST_DAY)
            .map(|v| strtoday(v))
            .unwrap_or(-1);
        if last_day < -1 {
            return Err(USimpleError::new(
                E_USAGE,
                format!(
                    "invalid date '{}'\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",
                    args_matches.value_of(options::LAST_DAY).unwrap(),
                    OPTIONS
                ),
            ));
        }

        let expire_date = args_matches
            .value_of(options::EXPIRE_DATE)
            .map(|v| strtoday(v))
            .unwrap_or(-1);
        if expire_date < -1 {
            return Err(USimpleError::new(
                E_USAGE,
                format!(
                    "invalid date '{}'\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",
                    args_matches.value_of(options::EXPIRE_DATE).unwrap(),
                    OPTIONS
                ),
            ));
        }

        let iso8601 = args_matches.is_present(options::ISO8601);
        let inactive_days = args_matches
            .value_of(options::INACTIVE)
            .map(|v| v.parse::<i64>().unwrap_or(-2))
            .unwrap_or(-1);
        if inactive_days < -1 {
            return Err(USimpleError::new(
                E_USAGE,
                format!("invalid numeric argument '{}'\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",
                args_matches.value_of(options::INACTIVE).unwrap(),
                OPTIONS)
            ));
        }

        let list = args_matches.is_present(options::LIST);
        let min_days = args_matches
            .value_of(options::MIN_DAYS)
            .map(|v| v.parse::<i64>().unwrap_or(-2))
            .unwrap_or(-1);
        if min_days < -1 {
            return Err(USimpleError::new(
                E_USAGE,
                format!("invalid numeric argument '{}'\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",
                args_matches.value_of(options::MIN_DAYS).unwrap(),
                OPTIONS)
            ));
        }

        let max_days = args_matches
            .value_of(options::MAX_DAYS)
            .map(|v| v.parse::<i64>().unwrap_or(-2))
            .unwrap_or(-1);
        if max_days < -1 {
            return Err(USimpleError::new(
                E_USAGE,
                format!("invalid numeric argument '{}'\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",
                args_matches.value_of(options::MAX_DAYS).unwrap(),
                OPTIONS)
            ));
        }

        let root = args_matches
            .value_of(options::ROOT)
            .map(|v| v.to_string())
            .unwrap_or_default();

        let warn_days = args_matches
            .value_of(options::WARN_DAYS)
            .map(|v| v.parse::<i64>().unwrap_or(-2))
            .unwrap_or(-1);
        if warn_days < -1 {
            return Err(USimpleError::new(
                E_USAGE,
                format!("invalid numeric argument '{}'\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",
                args_matches.value_of(options::WARN_DAYS).unwrap(),
                OPTIONS)
            ));
        }
        let login = args_matches
            .value_of(options::LOGIN)
            .map(|v| v.to_string())
            .unwrap_or_default();

        let lstday_present = args_matches.is_present(options::LAST_DAY);
        let expdate_present = args_matches.is_present(options::EXPIRE_DATE);
        let inact_present = args_matches.is_present(options::INACTIVE);
        let mindays_present = args_matches.is_present(options::MIN_DAYS);
        let maxdays_present = args_matches.is_present(options::MAX_DAYS);
        let warndays_present = args_matches.is_present(options::WARN_DAYS);

        Ok(Self {
            last_day,
            expire_date,
            iso8601,
            inactive_days,
            list,
            min_days,
            max_days,
            root,
            warn_days,
            login,
            ruid: get_current_uid(),
            rgid: get_current_gid(),
            isroot: get_current_uid() == 0,
            nochange: !lstday_present
                && !expdate_present
                && !inact_present
                && !mindays_present
                && !maxdays_present
                && !warndays_present,
            lstday_present,
            expdate_present,
            inact_present,
            mindays_present,
            maxdays_present,
            warndays_present,
        })
    }
}

///
pub fn parse_chage_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = chage_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn chage_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        // Format arguments.
        .arg(
            Arg::new(options::LAST_DAY)
                .short('d')
                .long(options::LAST_DAY)
                .allow_hyphen_values(true)
                .value_name("LAST_DAY")
                .help("set date of last password change to LAST_DAY")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::EXPIRE_DATE)
                .short('E')
                .long(options::EXPIRE_DATE)
                .allow_hyphen_values(true)
                .value_name("EXPIRE_DATE")
                .help("set account expiration date to EXPIRE_DATE")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::ISO8601)
                .short('i')
                .long(options::ISO8601)
                .help("use YYYY-MM-DD when printing dates"),
        )
        .arg(
            Arg::new(options::INACTIVE)
                .short('I')
                .long(options::INACTIVE)
                .allow_hyphen_values(true)
                .value_name("INACTIVE")
                .help("set password inactive after expiration to INACTIVE")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::LIST)
                .short('l')
                .long(options::LIST)
                .help("show account aging information"),
        )
        .arg(
            Arg::new(options::MIN_DAYS)
                .short('m')
                .long(options::MIN_DAYS)
                .allow_hyphen_values(true)
                .value_name("MIN_DAYS")
                .help("set minimum number of days before password change to MIN_DAYS")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::MAX_DAYS)
                .short('M')
                .long(options::MAX_DAYS)
                .allow_hyphen_values(true)
                .value_name("MAX_DAYS")
                .help("set maximum number of days before password change to MAX_DAYS")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::ROOT)
                .short('R')
                .long(options::ROOT)
                .value_name("CHROOT_DIR")
                .help("directory to chroot into")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::WARN_DAYS)
                .short('W')
                .long(options::WARN_DAYS)
                .allow_hyphen_values(true)
                .value_name("WARN_DAYS")
                .help("set expiration warning days to WARN_DAYS")
                .takes_value(true),
        )
        .arg(Arg::new(options::LOGIN).required(true).multiple(false))
}

/// Struct representing the spwd structure
#[derive(Debug, Clone)]
pub struct Spwd {
    ///
    pub sp_namp: String,
    ///
    pub sp_pwdp: String,
    ///
    pub sp_lstchg: i64,
    ///
    pub sp_min: i64,
    ///
    pub sp_max: i64,
    ///
    pub sp_warn: i64,
    ///
    pub sp_inact: i64,
    ///
    pub sp_expire: i64,
    ///
    pub sp_flag: u64,
}

/// A record in the user database.
#[derive(Debug, Clone)]
pub struct Passwd {
    ///
    pub pw_name: String,
    ///
    pub pw_passwd: String,
    ///
    pub pw_uid: u32,
    ///
    pub pw_gid: u32,
    ///
    pub pw_gecos: String,
    ///
    pub pw_dir: String,
    ///
    pub pw_shell: String,
}

///
struct OpenedFiles {
    shadow_file: Option<File>,
    shadow_islock: bool,
    passwd_file: Option<File>,
    passwd_islock: bool,
}

///
fn strtoday(day_str: &str) -> i64 {
    if day_str.is_empty() {
        return -1;
    }

    let mut s = day_str.trim_start();
    if s.starts_with('-') {
        s = s.trim_start_matches('-');
    }

    let isnum = s.chars().all(|c| c.is_digit(10));
    if isnum {
        if let Ok(retdate) = day_str.parse::<i64>() {
            return retdate;
        } else {
            return -2;
        }
    }

    match NaiveDate::parse_from_str(day_str, "%Y-%m-%d") {
        Ok(date_time) => date_time.num_days_from_ce() as i64 - UNIX_EPOCH_DAYS,
        Err(_) => -2,
    }
}

///
fn interactive_get_field(config: &mut Config) -> bool {
    let mut buf = String::new();

    println!(
        "{}",
        "Enter the new value, or press ENTER for the default\n"
    );

    //
    print!("\tMinimum Password Age [{}]: ", config.min_days);
    stdout().flush().unwrap();
    if stdin().read_line(&mut buf).is_err() {
        return false;
    }
    let input = buf.trim();
    if !input.is_empty() {
        config.min_days = match input.parse() {
            Ok(v) => {
                if v < -1 {
                    return false;
                }
                v
            }
            Err(_) => {
                return false;
            }
        };
    }

    //
    print!("\tMaximum Password Age [{}]: ", config.max_days);
    stdout().flush().unwrap();
    buf.clear();
    if stdin().read_line(&mut buf).is_err() {
        println!("hhhhh");
        return false;
    }
    let input = buf.trim();

    if !input.is_empty() {
        config.max_days = match input.parse() {
            Ok(v) => {
                if v < -1 {
                    println!("ghhhh");
                    return false;
                }
                v
            }
            Err(_) => {
                println!("{}", input);
                return false;
            }
        };
    }

    //
    let default_lstdate = if config.last_day == -1 || config.last_day > i64::MAX / DAY {
        "-1".to_string()
    } else {
        print_day_as_date(config.last_day, true)
    };
    print!(
        "\tLast Password Change (YYYY-MM-DD) [{}]: ",
        default_lstdate
    );
    stdout().flush().unwrap();
    buf.clear();
    if stdin().read_line(&mut buf).is_err() {
        return false;
    }
    let input = buf.trim();

    if !input.is_empty() {
        config.last_day = if input == "-1" {
            -1
        } else {
            match strtoday(input) {
                v if v < 0 => {
                    return false;
                }
                v => v,
            }
        };
    }

    //
    print!("\tPassword Expiration Warning [{}]: ", config.warn_days);
    stdout().flush().unwrap();
    buf.clear();
    if stdin().read_line(&mut buf).is_err() {
        return false;
    }
    let input = buf.trim();

    if !input.is_empty() {
        config.warn_days = match input.parse() {
            Ok(v) => {
                if v < -1 {
                    return false;
                }
                v
            }
            Err(_) => {
                return false;
            }
        };
    }

    //
    print!("\tPassword Inactive [{}]: ", config.inactive_days);
    stdout().flush().unwrap();
    buf.clear();
    if stdin().read_line(&mut buf).is_err() {
        return false;
    }
    let input = buf.trim();

    if !input.is_empty() {
        config.inactive_days = match input.parse() {
            Ok(v) => {
                if v < -1 {
                    return false;
                }
                v
            }
            Err(_) => {
                return false;
            }
        };
    }

    //
    let default_expdate = if config.expire_date == -1 || config.expire_date > i64::MAX / DAY {
        "-1".to_string()
    } else {
        print_day_as_date(config.expire_date, true)
    };
    print!(
        "\tAccount Expiration Date (YYYY-MM-DD) [{}]: ",
        default_expdate
    );
    stdout().flush().unwrap();
    buf.clear();
    if stdin().read_line(&mut buf).is_err() {
        return false;
    }
    let input = buf.trim();

    if !input.is_empty() {
        config.expire_date = if input == "-1" {
            -1
        } else {
            match strtoday(input) {
                v if v < 0 => {
                    return false;
                }
                v => v,
            }
        };
    }
    true
}

///
fn print_day_as_date(day: i64, iso8601: bool) -> String {
    if day < 0 {
        return "never".to_string();
    }
    let sec_date = match day.checked_mul(DAY) {
        Some(s) => s,
        None => return "future".to_string(),
    };

    let date_time = DateTime::from_timestamp(sec_date, 0);
    if let Some(date_time) = date_time {
        if iso8601 {
            date_time.format("%Y-%m-%d").to_string()
        } else {
            date_time.format("%b %d, %Y").to_string()
        }
    } else {
        format!("time_t: {}\n", sec_date)
    }
}

///
fn list_fields(config: &Config) {
    println!(
        "Last password change\t\t\t\t\t: {}",
        if config.last_day == 0 {
            "password must be changed".to_string()
        } else {
            print_day_as_date(config.last_day, config.iso8601)
        }
    );

    println!(
        "Password expires\t\t\t\t\t: {}",
        if config.last_day == 0 {
            "password must be changed".to_string()
        } else if config.last_day < 0
            || config.max_days >= 10000
            || config.max_days < 0
            || (std::i64::MAX - config.last_day) < config.max_days
        {
            "never".to_string()
        } else {
            print_day_as_date(config.last_day + config.max_days, config.iso8601)
        }
    );

    println!(
        "Password inactive\t\t\t\t\t: {}",
        if config.last_day == 0 {
            "password must be changed".to_string()
        } else if config.last_day < 0
            || config.inactive_days < 0
            || config.max_days >= 10000
            || config.max_days < 0
            || (std::i64::MAX - config.inactive_days) < config.max_days
            || (std::i64::MAX - config.last_day) < (config.max_days + config.inactive_days)
        {
            "never".to_string()
        } else {
            print_day_as_date(
                config.last_day + config.max_days + config.inactive_days,
                config.iso8601,
            )
        }
    );

    println!(
        "Account expires\t\t\t\t\t\t: {}",
        print_day_as_date(config.expire_date, config.iso8601)
    );

    println!(
        "Minimum number of days between password change\t\t: {}",
        config.min_days
    );
    println!(
        "Maximum number of days between password change\t\t: {}",
        config.max_days
    );
    println!(
        "Number of days of warning before password expires\t: {}",
        config.warn_days
    );
}

///
fn check_perms(config: &Config) -> bool {
    config.isroot || config.list
}

///
fn close_files(opened_files: &mut OpenedFiles) -> UResult<()> {
    let mut spw_succ = !opened_files.shadow_islock;
    let mut pw_succ = !opened_files.passwd_islock;
    if opened_files.shadow_islock {
        if Path::new(SHADOW_LOCK).exists() {
            if fs::remove_file(SHADOW_LOCK).is_ok() {
                spw_succ = true;
            }
        }
    }

    if opened_files.passwd_islock {
        if Path::new(PASSWD_LOCK).exists() {
            if fs::remove_file(PASSWD_LOCK).is_ok() {
                pw_succ = true;
            }
        }
    }

    if !spw_succ {
        return Err(USimpleError::new(
            E_NOPERM,
            format!("failed to unlock {}", SHADOW_FILE),
        ));
    }
    if !pw_succ {
        return Err(USimpleError::new(
            E_NOPERM,
            format!("failed to unlock {}", PASSWD_FILE),
        ));
    }

    Ok(())
}

///
fn pw_locate(login: &str, passwd_file: &File) -> Option<Passwd> {
    let reader = BufReader::new(passwd_file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 7 && fields[0] == login {
                let pw_name = fields[0].to_string();
                let pw_passwd = fields[1].to_string();
                let pw_uid = match fields[2].parse() {
                    Ok(uid) => uid,
                    Err(_) => return None,
                };
                let pw_gid = match fields[3].parse() {
                    Ok(gid) => gid,
                    Err(_) => return None,
                };
                let pw_gecos = fields[4].to_string();
                let pw_dir = fields[5].to_string();
                let pw_shell = fields[6].to_string();

                return Some(Passwd {
                    pw_name,
                    pw_passwd,
                    pw_uid,
                    pw_gid,
                    pw_gecos,
                    pw_dir,
                    pw_shell,
                });
            }
        }
    }

    None
}

///
fn spw_locate(login: &str, shadow_file: &File) -> Option<Spwd> {
    let reader = BufReader::new(shadow_file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 9 && fields[0] == login {
                let sp_namp = fields[0].to_string();
                let sp_pwdp = fields[1].to_string();
                let sp_lstchg = fields[2].parse().unwrap_or(-1);
                let sp_min = fields[3].parse().unwrap_or(-1);
                let sp_max = fields[4].parse().unwrap_or(-1);
                let sp_warn = fields[5].parse().unwrap_or(-1);
                let sp_inact = fields[6].parse().unwrap_or(-1);
                let sp_expire = fields[7].parse().unwrap_or(-1);
                let sp_flag = fields[8].parse().unwrap_or(0);

                return Some(Spwd {
                    sp_namp,
                    sp_pwdp,
                    sp_lstchg,
                    sp_min,
                    sp_max,
                    sp_warn,
                    sp_inact,
                    sp_expire,
                    sp_flag,
                });
            }
        }
    }

    None
}

///
fn open_files(opened_files: &mut OpenedFiles, read_only: bool) -> UResult<()> {
    if !Path::new(SHADOW_FILE).exists() {
        return Err(USimpleError::new(
            E_SHADOW_NOTFOUND,
            "the shadow password file is not present.",
        ));
    }

    if !read_only {
        // lock shadow file
        for i in 0..LOCK_TRIES {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(SHADOW_LOCK)
            {
                Ok(_) => {
                    break;
                }
                Err(_) => {
                    if i == LOCK_TRIES - 1 {
                        return Err(USimpleError::new(
                            E_NOPERM,
                            format!("cannot lock {}; try again later.", SHADOW_FILE),
                        ));
                    }
                    sleep(Duration::from_secs(LOCK_SLEEP));
                }
            }
        }
        opened_files.shadow_islock = true;
    }

    let shadow_file = match OpenOptions::new()
        .read(true)
        .write(!read_only)
        .create(!read_only)
        .open(SHADOW_FILE)
    {
        Ok(file) => file,
        Err(_) => {
            close_files(opened_files)?;
            return Err(USimpleError::new(
                E_SHADOW_NOTFOUND,
                format!("cannot open {}.", SHADOW_FILE),
            ));
        }
    };

    opened_files.shadow_file = Some(shadow_file);

    if !Path::new(PASSWD_FILE).exists() {
        return Err(USimpleError::new(
            E_PASSWD_NOTFOUND,
            "the password file is not present.",
        ));
    }

    if !read_only {
        // lock passwd file
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(PASSWD_LOCK)
        {
            Ok(_) => {}
            Err(_) => {
                return Err(USimpleError::new(
                    E_NOPERM,
                    format!("cannot lock {}; try again later.", PASSWD_FILE),
                ));
            }
        }
        opened_files.passwd_islock = true;
    }

    let passwd_file = match OpenOptions::new()
        .read(true)
        .write(!read_only)
        .create(!read_only)
        .open(PASSWD_FILE)
    {
        Ok(file) => file,
        Err(_) => {
            close_files(opened_files)?;
            return Err(USimpleError::new(
                E_PASSWD_NOTFOUND,
                format!("cannot open {}.", PASSWD_FILE),
            ));
        }
    };

    opened_files.passwd_file = Some(passwd_file);

    Ok(())
}

///
fn drop_privileges(config: &Config, opened_files: &mut OpenedFiles) -> UResult<()> {
    if config.list {
        if let Err(_) = set_both_uid(config.ruid, config.ruid) {
            close_files(opened_files)?;
            return Err(USimpleError::new(
                E_NOPERM,
                format!("failed to drop privileges: ({})", get_recent_err_msg()),
            ));
        };

        if let Err(_) = set_both_gid(config.rgid, config.rgid) {
            close_files(opened_files)?;
            return Err(USimpleError::new(
                E_NOPERM,
                format!("failed to drop privileges: ({})", get_recent_err_msg()),
            ));
        };
    }

    Ok(())
}

///
fn get_default(config: &mut Config, spwd: &Option<Spwd>) {
    if let Some(spwd) = spwd {
        if !config.lstday_present {
            config.last_day = spwd.sp_lstchg;
        }
        if !config.mindays_present {
            config.min_days = spwd.sp_min;
        }
        if !config.maxdays_present {
            config.max_days = spwd.sp_max;
        }
        if !config.warndays_present {
            config.warn_days = spwd.sp_warn;
        }
        if !config.inact_present {
            config.inactive_days = spwd.sp_inact;
        }
        if !config.expdate_present {
            config.expire_date = spwd.sp_expire;
        }
    } else {
        if !config.lstday_present {
            config.last_day = -1;
        }
        if !config.mindays_present {
            config.min_days = -1;
        }
        if !config.maxdays_present {
            config.max_days = -1;
        }
        if !config.warndays_present {
            config.warn_days = -1;
        }
        if !config.inact_present {
            config.inactive_days = -1;
        }
        if !config.expdate_present {
            config.expire_date = -1;
        }
    }
}

///
fn pw_update(pw: &Passwd, file: &mut File) -> Result<(), i32> {
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0)).map_err(|_| E_NOPERM)?;
    file.read_to_string(&mut contents).map_err(|_| E_NOPERM)?;

    let mut lines: Vec<&str> = contents.lines().collect();
    let mut updated_lines: Vec<String> = Vec::new();
    let mut updated = false;

    for line in lines.iter_mut() {
        if line.starts_with(format!("{}{}", pw.pw_name, ":").as_str()) {
            updated_lines.push(format!(
                "{}:{}:{}:{}:{}:{}:{}",
                pw.pw_name, pw.pw_passwd, pw.pw_uid, pw.pw_gid, pw.pw_gecos, pw.pw_dir, pw.pw_shell,
            ));
            updated = true;
        } else {
            updated_lines.push(line.to_string());
        }
    }

    if !updated {
        updated_lines.push(format!(
            "{}:{}:{}:{}:{}:{}:{}",
            pw.pw_name, pw.pw_passwd, pw.pw_uid, pw.pw_gid, pw.pw_gecos, pw.pw_dir, pw.pw_shell,
        ));
    }

    file.seek(SeekFrom::Start(0)).map_err(|_| E_NOPERM)?;
    file.set_len(0).map_err(|_| E_NOPERM)?;

    for line in updated_lines {
        file.write_all(line.as_bytes()).map_err(|_| E_NOPERM)?;
        file.write_all(b"\n").map_err(|_| E_NOPERM)?;
    }

    Ok(())
}

///
fn i64tostr(i: i64) -> String {
    if i == -1 {
        "".to_string()
    } else {
        i.to_string()
    }
}

///
fn u64tostr(i: u64) -> String {
    if i == 0 {
        "".to_string()
    } else {
        i.to_string()
    }
}

///
fn spw_update(spw: &Spwd, file: &mut File) -> Result<(), i32> {
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0)).map_err(|_| E_NOPERM)?;
    file.read_to_string(&mut contents).map_err(|_| E_NOPERM)?;

    let mut lines: Vec<&str> = contents.lines().collect();
    let mut updated_lines: Vec<String> = Vec::new();
    let mut updated = false;

    for line in lines.iter_mut() {
        if line.starts_with(format!("{}{}", spw.sp_namp, ":").as_str()) {
            updated_lines.push(format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}",
                spw.sp_namp,
                spw.sp_pwdp,
                i64tostr(spw.sp_lstchg),
                i64tostr(spw.sp_min),
                i64tostr(spw.sp_max),
                i64tostr(spw.sp_warn),
                i64tostr(spw.sp_inact),
                i64tostr(spw.sp_expire),
                u64tostr(spw.sp_flag),
            ));
            updated = true;
        } else {
            updated_lines.push(line.to_string());
        }
    }

    if !updated {
        updated_lines.push(format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}",
            spw.sp_namp,
            spw.sp_pwdp,
            i64tostr(spw.sp_lstchg),
            i64tostr(spw.sp_min),
            i64tostr(spw.sp_max),
            i64tostr(spw.sp_warn),
            i64tostr(spw.sp_inact),
            i64tostr(spw.sp_expire),
            u64tostr(spw.sp_flag)
        ));
    }

    file.seek(SeekFrom::Start(0)).map_err(|_| E_NOPERM)?;
    file.set_len(0).map_err(|_| E_NOPERM)?;

    for line in updated_lines {
        file.write_all(line.as_bytes()).map_err(|_| E_NOPERM)?;
        file.write_all(b"\n").map_err(|_| E_NOPERM)?;
    }
    file.flush().map_err(|_| E_NOPERM)?;
    Ok(())
}

///
fn update_age(
    spwd: &Option<Spwd>,
    pw: &Passwd,
    config: &Config,
    opened_files: &mut OpenedFiles,
) -> UResult<()> {
    let mut pwent = (*pw).clone();
    let spwent = match spwd {
        Some(sp) => Spwd {
            sp_namp: sp.sp_namp.clone(),
            sp_pwdp: sp.sp_pwdp.clone(),
            sp_lstchg: config.last_day,
            sp_min: config.min_days,
            sp_max: config.max_days,
            sp_warn: config.warn_days,
            sp_inact: config.inactive_days,
            sp_expire: config.expire_date,
            sp_flag: sp.sp_flag,
        },
        None => {
            pwent.pw_passwd = SHADOW_PASSWD_STRING.to_string();
            if let Err(_) = pw_update(&pwent, opened_files.passwd_file.as_mut().unwrap()) {
                return Err(USimpleError::new(
                    E_NOPERM,
                    format!(
                        "failed to prepare the new {} entry '{}'",
                        PASSWD_FILE, pwent.pw_name
                    ),
                ));
            }
            Spwd {
                sp_namp: pw.pw_name.clone(),
                sp_pwdp: pw.pw_passwd.clone(),
                sp_lstchg: config.last_day,
                sp_min: config.min_days,
                sp_max: config.max_days,
                sp_warn: config.warn_days,
                sp_inact: config.inactive_days,
                sp_expire: config.expire_date,
                sp_flag: 0,
            }
        }
    };

    if spw_update(&spwent, opened_files.shadow_file.as_mut().unwrap()).is_err() {
        return Err(USimpleError::new(
            E_NOPERM,
            format!(
                "failed to prepare the new {} entry '{}'",
                SHADOW_FILE, spwent.sp_namp
            ),
        ));
    }
    Ok(())
}

///
fn sanitize_env() {
    let forbid = [
        "_RLD_=",
        "BASH_ENV=",
        "ENV=",
        "HOME=",
        "IFS=",
        "KRB_CONF=",
        "LD_",
        "LIBPATH=",
        "MAIL=",
        "NLSPATH=",
        "PATH=",
        "SHELL=",
        "SHLIB_PATH=",
    ];

    let noslash = ["LANG=", "LANGUAGE=", "LC_"];

    let mut envp = std::env::vars().collect::<Vec<_>>();

    let mut to_removed = Vec::new();

    for cur in &mut envp {
        for bad in &forbid {
            if cur.0.starts_with(bad) {
                to_removed.push(cur.0.clone());
                break;
            }
        }
    }

    for cur in &mut envp {
        for bad in &noslash {
            if cur.0.starts_with(bad) && cur.0.contains('/') {
                to_removed.push(cur.0.clone());
                break;
            }
        }
    }

    for var in &to_removed {
        std::env::remove_var(var);
    }
}

///
fn check_fd(fd: i32) {
    let flags = fcntl(fd, FcntlArg::F_GETFL).unwrap_or(-1);
    if flags != -1 {
        return;
    }

    let devnull = OpenOptions::new().read(true).write(true).open("/dev/null");

    match devnull {
        Ok(file) => {
            let devnull_fd = file.as_raw_fd();
            if devnull_fd != fd {
                panic!("Failed to open /dev/null with the same file descriptor");
            }
        }
        Err(err) => {
            panic!("Failed to open /dev/null: {}", err);
        }
    }
}

///
fn check_fds() {
    for fd in 0..3 {
        check_fd(fd);
    }
}

///
fn get_recent_err_msg() -> String {
    unsafe { CStr::from_ptr(strerror(*libc::__errno_location())) }
        .to_string_lossy()
        .to_string()
}

///
fn change_root(chroot_dir: &str) -> UResult<()> {
    if chroot_dir.is_empty() || !chroot_dir.starts_with('/') {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!(
                "invalid chroot path '{}', only absolute paths are supported.",
                chroot_dir
            ),
        ));
    }

    if !Path::new(chroot_dir).exists() {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!(
                "cannot access chroot directory {}: {}",
                chroot_dir,
                get_recent_err_msg()
            ),
        ));
    }

    if let Err(_) = chroot(chroot_dir) {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!(
                "unable to chroot to directory {}: {}",
                chroot_dir,
                get_recent_err_msg()
            ),
        ));
    };

    if let Err(_) = chdir("/") {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!(
                "cannot chdir in chroot directory {}: {}",
                chroot_dir,
                get_recent_err_msg()
            ),
        ));
    }

    Ok(())
}

///
pub fn check_flag(config: &Config) -> UResult<()> {
    if config.list && !config.nochange {
        return Err(USimpleError::new(
            E_USAGE,
            format!("do not include \"l\" with other flags\nUsage: chage [options] LOGIN\n\nOptions:\n  {}\n",OPTIONS),
        ));
    }
    Ok(())
}

///
pub fn handle_input(mut config: Config) -> UResult<i32> {
    check_flag(&config)?;
    sanitize_env();
    check_fds();

    if config.root != "" {
        change_root(&config.root)?;
    }

    if !check_perms(&config) {
        return Err(USimpleError::new(E_NOPERM, "Permission denied."));
    }

    let mut opened_files = OpenedFiles {
        shadow_file: None,
        shadow_islock: false,
        passwd_file: None,
        passwd_islock: false,
    };

    if let Err(e) = open_files(&mut opened_files, config.list) {
        close_files(&mut opened_files)?;
        return Err(e);
    }
    // Drop privileges
    if let Err(e) = drop_privileges(&config, &mut opened_files) {
        close_files(&mut opened_files)?;
        return Err(e);
    }

    let passwd = match pw_locate(&config.login, opened_files.passwd_file.as_ref().unwrap()) {
        Some(pw) => pw,
        None => {
            close_files(&mut opened_files)?;
            return Err(USimpleError::new(
                E_NOPERM,
                format!("user '{}' does not exist in {}", config.login, PASSWD_FILE),
            ));
        }
    };

    let shadow = spw_locate(&config.login, opened_files.shadow_file.as_ref().unwrap());

    get_default(&mut config, &shadow);

    if config.list {
        if !config.isroot && (config.ruid != passwd.pw_uid) {
            close_files(&mut opened_files)?;
            return Err(USimpleError::new(E_NOPERM, "Permission denied."));
        }

        list_fields(&config);
        return Ok(EXIT_SUCCESS);
    }

    if config.nochange {
        println!("Changing the aging information for {}", config.login);
        if !interactive_get_field(&mut config) {
            close_files(&mut opened_files)?;
            return Err(USimpleError::new(E_NOPERM, "error changing fields"));
        }
    }

    if let Err(e) = update_age(&shadow, &passwd, &config, &mut opened_files) {
        close_files(&mut opened_files)?;
        return Err(e);
    }

    close_files(&mut opened_files)?;
    Ok(0)
}
