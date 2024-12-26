//! This file is part of the easybox package.
//! //
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use errno::{errno, set_errno, Errno};
use glob::{glob, Pattern};
use libc::{
    EACCES, EIO, EISDIR, ENOENT, EPERM, EROFS, EXIT_FAILURE, EXIT_SUCCESS, S_IRUSR, S_IWUSR,
};
use regex::Regex;
use std::fs::{canonicalize, metadata, read_dir, File};
use std::io::{BufRead, BufReader, Write};
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use uucore::error::UResult;
use uucore::msg_log::{err_c, errx_c, warn_c, warnx_c};
use uucore::{format_usage, util_name};

///
const PROC_PATH: &str = "/proc/sys/";
///
const DEFAULT_PRELOAD: &str = "/etc/sysctl.conf";
///
const DEPRECATED: [&str; 2] = ["base_reachable_time", "retrans_time"];
/// Verboten parameters must never be read as they cause side-effects.
const VERBOTEN: [&str; 1] = ["stat_refresh"];

///
const GLOB_CHARS: &str = "*?[";

/// Config.
pub struct Config {
    ///
    pub all: bool,
    ///
    pub deprecated: bool,
    ///
    pub dry_run: bool,
    ///
    pub binary: bool,
    ///
    pub ignore: bool,
    ///
    pub names: bool,
    ///
    pub values: bool,
    ///
    pub load: Vec<String>,
    ///
    pub system: bool,
    ///
    pub pattern: Option<String>,
    ///
    pub quiet: bool,
    ///
    pub write: bool,
    ///
    pub variable: Vec<String>,
}

/// options.
pub mod options {
    ///
    pub static ALL: &str = "all";
    ///
    pub static DEPRECATED: &str = "deprecated";
    ///
    pub static DRY_RUN: &str = "dry-run";
    ///
    pub static BINARY: &str = "binary";
    ///
    pub static IGNORE: &str = "ignore";
    ///
    pub static NAMES: &str = "names";
    ///
    pub static VALUES: &str = "values";
    ///
    pub static LOAD: &str = "load";
    ///
    pub static SYSTEM: &str = "system";
    ///
    pub static PATTERN: &str = "pattern";
    ///
    pub static QUIET: &str = "quiet";
    ///
    pub static WRITE: &str = "write";
    ///
    pub static VARIABLE: &str = "variable";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let load = options
            .get_many::<String>(options::LOAD)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let pattern: Option<String> = options.get_one::<String>(options::PATTERN).cloned();

        let variable = options
            .get_many::<String>(options::VARIABLE)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        Ok(Self {
            all: options.contains_id(options::ALL),
            deprecated: options.contains_id(options::DEPRECATED),
            dry_run: options.contains_id(options::DRY_RUN),
            binary: options.contains_id(options::BINARY),
            ignore: options.contains_id(options::IGNORE),
            names: options.contains_id(options::NAMES),
            values: options.contains_id(options::VALUES),
            load,
            system: options.contains_id(options::SYSTEM),
            pattern,
            quiet: options.contains_id(options::QUIET),
            write: options.contains_id(options::WRITE),
            variable,
        })
    }
}

///
struct InternalConfig {
    ///
    pub display_all_opt: bool,
    ///
    pub ignore_deprecated: bool,
    ///
    pub dry_run: bool,
    ///
    pub ignore_error: bool,
    ///
    pub name_only: bool,
    ///
    pub preload_file_opt: bool,
    ///
    pub pattern: Option<String>,
    ///
    pub quiet: bool,
    ///
    pub write_mode: bool,
    ///
    pub print_name: bool,
    ///
    pub print_new_line: bool,
}

impl InternalConfig {
    ///
    pub fn from(config: &Config) -> UResult<Self> {
        Ok(Self {
            display_all_opt: config.all,
            ignore_deprecated: !config.deprecated,
            dry_run: config.dry_run,
            ignore_error: config.ignore || config.system,
            name_only: config.names,
            preload_file_opt: !config.load.is_empty(),
            pattern: config.pattern.clone(),
            quiet: config.quiet,
            write_mode: config.write,
            print_name: !(config.binary || config.values),
            print_new_line: !config.binary,
        })
    }
}

///
pub fn parse_sysctl_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = sysctl_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn sysctl_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new(options::ALL)
                .short('a')
                .visible_short_aliases(&['A', 'X'])
                .long(options::ALL)
                .help("display all variables")
                .display_order(0),
        )
        .arg(
            Arg::new(options::DEPRECATED)
                .long(options::DEPRECATED)
                .help("include deprecated parameters to listing")
                .display_order(1),
        )
        .arg(
            Arg::new(options::DRY_RUN)
                .long(options::DRY_RUN)
                .help("Print the key and values but do not write")
                .display_order(2),
        )
        .arg(
            Arg::new(options::BINARY)
                .short('b')
                .long(options::BINARY)
                .help("print value without new line")
                .display_order(3),
        )
        .arg(
            Arg::new(options::IGNORE)
                .short('e')
                .long(options::IGNORE)
                .help("ignore unknown variables errors")
                .display_order(4),
        )
        .arg(
            Arg::new(options::NAMES)
                .short('N')
                .long(options::NAMES)
                .help("print variable names without values")
                .display_order(5),
        )
        .arg(
            Arg::new(options::VALUES)
                .short('n')
                .long(options::VALUES)
                .help("print only values of the given variable(s)")
                .display_order(6),
        )
        .arg(
            Arg::new(options::LOAD)
                .short('p')
                .visible_short_alias('f')
                .long(options::LOAD)
                .help("read values from file")
                .display_order(7)
                .multiple_values(true)
                .action(clap::ArgAction::Append)
                .default_missing_value(DEFAULT_PRELOAD),
        )
        .arg(
            Arg::new(options::SYSTEM)
                .long(options::SYSTEM)
                .help("read values from all system directories")
                .display_order(8),
        )
        .arg(
            Arg::new(options::PATTERN)
                .short('r')
                .long(options::PATTERN)
                .help("select setting that match expression")
                .display_order(9)
                .takes_value(true)
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new(options::QUIET)
                .short('q')
                .long(options::QUIET)
                .help("do not echo variable set")
                .display_order(10),
        )
        .arg(
            Arg::new(options::WRITE)
                .short('w')
                .long(options::WRITE)
                .help("enable writing a value to variable")
                .display_order(11),
        )
        .arg(
            Arg::new("o does nothing")
                .short('o')
                .help("does nothing")
                .display_order(12),
        )
        .arg(
            Arg::new("x does nothing")
                .short('x')
                .help("does nothing")
                .display_order(13),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .visible_short_alias('d')
                .long("help")
                .help("display this help and exit")
                .display_order(14),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .help("output version information and exit")
                .display_order(15),
        )
        .arg(Arg::new(options::VARIABLE).hide(true).multiple_values(true))
        .trailing_var_arg(true)
}

///
struct SysctlSetting {
    key: String,
    path: String,
    value: String,
    ignore_failure: bool,
    glob_exclude: bool,
}

impl SysctlSetting {
    ///
    pub fn new(key: &str, value: &str, ignore_failure: bool, glob_exclude: bool) -> Self {
        let mut path = String::from(PROC_PATH);
        if let Some(stripped) = key.strip_prefix('-') {
            path.push_str(stripped);
        } else {
            path.push_str(key);
        }

        slashdot(&mut path, '.', '/', PROC_PATH.len());

        Self {
            key: key.to_string(),
            path,
            value: value.to_string(),
            ignore_failure,
            glob_exclude,
        }
    }
}

/// Find the first char index in the ct string in the cs string.
fn strpbrk(cs: &str, ct: &str) -> Option<usize> {
    for (i, c) in cs.char_indices() {
        if ct.contains(c) {
            return Some(i);
        }
    }
    None
}

/// Check if a string is glob.
fn string_is_glob(str: &str) -> bool {
    strpbrk(str, GLOB_CHARS).is_some()
}

/// Check if a path starts with PROC_PATH.
fn is_proc_path(path: &str) -> bool {
    if let Ok(resolved_path) = canonicalize(path) {
        if resolved_path.starts_with(PROC_PATH) {
            return true;
        } else {
            warnx_c(&format!("Path is not under {}: {}", PROC_PATH, path));
        }
    }

    false
}

/// Convert old char in the string to new char.
fn slashdot(string: &mut String, old: char, new: char, start_pos: usize) {
    let mut warned: bool = false;
    let mut first: bool = true;

    for idx in start_pos..string.len() {
        if let Some(cur_char) = string.chars().nth(idx) {
            if cur_char == new || cur_char == old {
                if first {
                    if cur_char == new {
                        return;
                    } else {
                        first = false;
                    }
                }

                if let Some(next_char) = string.chars().nth(idx + 1) {
                    if (next_char == '/' || next_char == '.') && !warned {
                        warnx_c(&format!(
                            "separators should not be repeated: {}",
                            &string[idx..]
                        ));
                        warned = true;
                    }
                }

                if cur_char == old {
                    string.replace_range(idx..=idx, &new.to_string());
                }

                if cur_char == new {
                    string.replace_range(idx..=idx, &old.to_string());
                }
            }
        }
    }
}

/// Check if a path is in settinglist.
fn settinglist_findpath(setting_list: &[SysctlSetting], path: &str) -> bool {
    for setting in setting_list.iter() {
        if setting.path == path {
            return true;
        }
        if setting.glob_exclude {
            if let Ok(pattern) = Pattern::new(&setting.path) {
                if pattern.matches_path(Path::new(path)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Read a sysctl setting.
fn read_setting(name: &str, internal_config: &InternalConfig) -> i32 {
    let mut rc = EXIT_SUCCESS;

    if name.is_empty() {
        warnx_c(&format!("\"{}\" is an unknown key", name));
        return -1;
    }

    let mut tmp_name = format!("{}{}", PROC_PATH, name);
    slashdot(&mut tmp_name, '.', '/', PROC_PATH.len());

    let mut out_name = name.to_string();
    slashdot(&mut out_name, '/', '.', 0);

    let metadata = match metadata(&tmp_name) {
        Ok(metadata) => metadata,
        Err(_) => {
            if !internal_config.ignore_error {
                warn_c(&format!("cannot stat {}", tmp_name));
                rc = EXIT_FAILURE;
            }
            return rc;
        }
    };

    if metadata.st_mode() & S_IRUSR == 0 {
        return rc;
    }

    if !is_proc_path(&tmp_name) {
        return -1;
    }

    if metadata.is_dir() {
        tmp_name.push('/');
        return display_all(&tmp_name, internal_config);
    }

    if let Some(pattern) = &internal_config.pattern {
        if !pattern_match(&out_name, pattern) {
            return EXIT_SUCCESS;
        }
    }

    if internal_config.name_only {
        println!("{}", out_name);
        return rc;
    }

    let file = match File::open(&tmp_name) {
        Ok(file) => file,
        Err(_) => match errno() {
            Errno(ENOENT) => {
                if !internal_config.ignore_error {
                    warnx_c(&format!("\"{}\" is an unknown key", out_name));
                }
                rc = EXIT_FAILURE;
                return rc;
            }
            Errno(EACCES) => {
                warnx_c(&format!("permission denied on key '{}'", out_name));
                rc = EXIT_FAILURE;
                return rc;
            }
            Errno(EIO) => {
                rc = EXIT_FAILURE;
                return rc;
            }
            _ => {
                warnx_c(&format!("reading key \"{}\"", out_name));
                rc = EXIT_FAILURE;
                return rc;
            }
        },
    };

    set_errno(Errno(0));
    let mut first_line = true;
    let mut buf = String::new();
    let mut reader = BufReader::new(file);

    loop {
        match reader.read_line(&mut buf) {
            Ok(n) => {
                first_line = false;
                if n == 0 {
                    break;
                }

                if internal_config.print_name {
                    print!("{} = {}", out_name, buf);
                    if !buf.ends_with('\n') {
                        println!();
                    }
                } else if !internal_config.print_new_line && buf.ends_with('\n') {
                    print!("{}", buf.trim_end_matches('\n'));
                } else {
                    print!("{}", buf);
                }

                buf.clear();
            }
            Err(_) => {
                if first_line {
                    match errno() {
                        Errno(EACCES) => {
                            warnx_c(&format!("permission denied on key '{}'", out_name));
                            rc = EXIT_FAILURE;
                        }
                        Errno(EISDIR) => {
                            tmp_name.push('/');
                            rc = display_all(&tmp_name, internal_config);
                        }
                        Errno(EIO) => rc = EXIT_FAILURE,
                        Errno(0) => {}
                        _ => {
                            warnx_c(&format!("reading key \"{}\"", out_name));
                            rc = EXIT_FAILURE;
                        }
                    };
                }
                return rc;
            }
        }
    }

    EXIT_SUCCESS
}

/// Check if a filename is deprecated.
fn is_deprecated(filename: &str) -> bool {
    DEPRECATED.into_iter().any(|d| filename == d)
}

/// Check if a filename is verboten.
fn is_verboten(filename: &str) -> bool {
    VERBOTEN.into_iter().any(|d| filename == d)
}

/// Display all the sysctl settings.
fn display_all(path: &str, internal_config: &InternalConfig) -> i32 {
    let mut rc = EXIT_SUCCESS;

    let entries = match read_dir(path) {
        Ok(entries) => entries,
        Err(_) => {
            warnx_c(&format!("unable to open directory \"{}\"", path));
            return EXIT_FAILURE;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if internal_config.ignore_deprecated
            && is_deprecated(entry.file_name().to_str().unwrap_or_default())
        {
            continue;
        }

        if is_verboten(entry.file_name().to_str().unwrap_or_default()) {
            continue;
        }

        let file_path = Path::new(path).join(&entry.file_name());
        let file_path_str = file_path.to_string_lossy();

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                warn_c(&format!("cannot stat {}", file_path_str));
                continue;
            }
        };

        if file_type.is_dir() {
            let mut dir_path_string = String::from(file_path_str);
            dir_path_string.push('/');
            display_all(&dir_path_string, internal_config);
        } else {
            // file_path_str always starts with PROC_PATH.
            if let Some(setting_path) = file_path_str.strip_prefix(PROC_PATH) {
                rc |= read_setting(setting_path, internal_config);
            }
        }
    }

    rc
}

/// Write a sysctl setting.
fn write_setting(
    key: &str,
    path: &str,
    value: &str,
    ignore_failure: bool,
    internal_config: &InternalConfig,
) -> i32 {
    let mut rc = EXIT_SUCCESS;

    if key.is_empty() || path.is_empty() {
        return rc;
    }

    let metadata = match metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => {
            if !internal_config.ignore_error {
                warn_c(&format!("cannot stat {}", path));
                rc = EXIT_FAILURE;
            }
            return rc;
        }
    };

    if !is_proc_path(path) {
        return EXIT_FAILURE;
    }

    // path always starts with PROC_PATH.
    let mut dotted_key = String::from(&path[PROC_PATH.len()..]);
    slashdot(&mut dotted_key, '/', '.', 0);

    if metadata.st_mode() & S_IWUSR == 0 {
        set_errno(Errno(EPERM));
        warn_c(&format!("setting key \"{}\"", dotted_key));
        return rc;
    }

    if metadata.is_dir() {
        set_errno(Errno(EISDIR));
        warn_c(&format!("setting key \"{}\"", dotted_key));
        return rc;
    }

    if !internal_config.dry_run {
        match File::create(path) {
            Ok(mut file) => {
                let mut buf_to_write = String::from(value);
                buf_to_write.push('\n');
                match file.write_all(buf_to_write.as_bytes()) {
                    Ok(_) => rc = EXIT_SUCCESS,
                    Err(_) => {
                        warn_c(&format!("setting key \"{}\"", dotted_key));
                        return EXIT_FAILURE;
                    }
                };
            }
            Err(_) => {
                match errno() {
                    Errno(ENOENT) => {
                        if !internal_config.ignore_error {
                            warnx_c(&format!(
                                "\"{}\" is an unknown key{}",
                                dotted_key,
                                if ignore_failure { ", ignoring" } else { "" }
                            ));
                            if !ignore_failure {
                                rc = EXIT_FAILURE;
                            }
                        }
                    }
                    Errno(EPERM) | Errno(EROFS) | Errno(EACCES) => {
                        warnx_c(&format!(
                            "permission denied on key \"{}\"{}",
                            dotted_key,
                            if ignore_failure { ", ignoring" } else { "" }
                        ));
                    }
                    _ => {
                        warnx_c(&format!(
                            "setting key \"{}\"{}",
                            dotted_key,
                            if ignore_failure { ", ignoring" } else { "" }
                        ));
                    }
                }
                if !ignore_failure && errno() != Errno(ENOENT) {
                    rc = EXIT_FAILURE;
                }
            }
        }
    }

    if (rc == EXIT_SUCCESS && !internal_config.quiet) || internal_config.dry_run {
        if internal_config.name_only {
            println!("{}", dotted_key);
        } else if internal_config.print_name {
            println!("{} = {}", dotted_key, value);
        } else if internal_config.print_new_line {
            println!("{}", value);
        } else {
            print!("{}", value);
        }
    }

    rc
}

/// Parse each configuration line, there are multiple ways of specifying.
///
/// a key/value here:
///
/// key = value                               simple setting
/// -key = value                              ignore errors
/// key.pattern.*.with.glob = value           set keys that match glob
/// -key.pattern.exclude.with.glob            dont set this value
/// -key.pattern.exclude.*.glob               dont set values for keys matching glob
/// key.pattern.override.with.glob = value    set this glob match to value
fn parse_setting_line(
    path: &str,
    linenum: isize,
    line: &str,
    internal_config: &InternalConfig,
) -> Option<SysctlSetting> {
    let setting_line = line.trim_start_matches(' ');
    if setting_line.len() < 2 {
        return None;
    }

    if setting_line.starts_with('#') || setting_line.starts_with(';') {
        return None;
    }

    if let Some(pattern) = &internal_config.pattern {
        if !pattern_match(setting_line, pattern) {
            return None;
        }
    }

    let mut key;
    let mut value = "";
    let mut glob_exclude = false;
    let mut ignore_failure = false;

    if let Some(index) = setting_line.find('=') {
        (key, value) = setting_line.split_at(index);
        if key.starts_with('-') {
            ignore_failure = true;
            key = &key[1..];
        }
        value = &value[1..];
    } else if let Some(stripped) = setting_line.strip_prefix('-') {
        glob_exclude = true;
        key = stripped;
    } else {
        warnx_c(&format!(
            "{}({}): invalid syntax, continuing...",
            path, linenum
        ));
        return None;
    }

    key = key.trim_end_matches(' ');
    value = value.trim_matches(' ').trim_end_matches(' ');

    Some(SysctlSetting::new(key, value, ignore_failure, glob_exclude))
}

/// Go through the setting list, expand and sort out setting globs and actually write the settings out.
fn write_setting_list(setting_list: &[SysctlSetting], internal_config: &InternalConfig) -> i32 {
    let mut rc = EXIT_SUCCESS;

    for setting in setting_list.iter() {
        if setting.glob_exclude {
            continue;
        }

        if string_is_glob(&setting.path) {
            if let Ok(paths) = glob(&setting.path) {
                for path_buf in paths.into_iter().flatten() {
                    let path_str = path_buf.to_string_lossy();
                    if settinglist_findpath(setting_list, &path_str) {
                        continue;
                    }
                    rc |= write_setting(
                        &setting.key,
                        &path_str,
                        &setting.value,
                        setting.ignore_failure,
                        internal_config,
                    );
                }
            }
        } else {
            rc |= write_setting(
                &setting.key,
                &setting.path,
                &setting.value,
                setting.ignore_failure,
                internal_config,
            );
        }
    }

    rc
}

/// Check if the string matches the pattern.
fn pattern_match(string: &str, pattern: &str) -> bool {
    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_) => return false,
    };
    regex.is_match(string)
}

/// Preload the sysctl's from the conf file. We parse the file and then reform it (strip out whitespace).
fn preload(
    setting_list: &mut Vec<SysctlSetting>,
    filename: &str,
    internal_config: &InternalConfig,
) -> i32 {
    let rc = EXIT_SUCCESS;
    let mut line_num = 0;

    let paths = match glob(filename) {
        Ok(paths) => paths,
        Err(_) => {
            err_c(EXIT_FAILURE, "glob failed");
            return EXIT_FAILURE;
        }
    };

    for entry in paths.flatten() {
        let fp: Box<dyn BufRead>;

        if entry.to_string_lossy() == "-" {
            fp = Box::new(BufReader::new(std::io::stdin()));
        } else {
            match File::open(&entry) {
                Ok(file) => fp = Box::new(BufReader::new(file)),
                Err(_) => {
                    warn_c(&format!("cannot open \"{}\"", entry.to_string_lossy()));
                    return EXIT_FAILURE;
                }
            }
        }

        for line in fp.lines().map_while(Result::ok) {
            line_num += 1;

            if line.len() < 2 {
                continue;
            }

            if let Some(setting) =
                parse_setting_line(&entry.to_string_lossy(), line_num, &line, internal_config)
            {
                setting_list.push(setting);
            }
        }
    }

    rc
}

/// Preload the sysctl's from the conf file in system.
fn preload_system(setting_list: &mut Vec<SysctlSetting>, internal_config: &InternalConfig) -> i32 {
    let dirs = [
        "/etc/sysctl.d",
        "/run/sysctl.d",
        "/usr/local/lib/sysctl.d",
        "/usr/lib/sysctl.d",
        "/lib/sysctl.d",
    ];

    let mut rc = 0;
    let mut cfgs: Vec<(String, String)> = Vec::new();

    for dir in dirs {
        if let Ok(entries) = read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".conf") && !cfgs.clone().into_iter().any(|cfg| cfg.0 == name) {
                    let value = entry.path().to_string_lossy().to_string();
                    cfgs.push((name, value));
                }
            }
        }
    }

    cfgs.sort_by_key(|cfg| cfg.0.clone());

    for cfg in cfgs {
        if !internal_config.quiet {
            println!("* Applying {} ...", cfg.1);
        }
        rc |= preload(setting_list, &cfg.1, internal_config);
    }

    if let Ok(metadata) = metadata(DEFAULT_PRELOAD) {
        if metadata.is_file() {
            if !internal_config.quiet {
                println!("* Applying {} ...", DEFAULT_PRELOAD);
            }
            rc |= preload(setting_list, DEFAULT_PRELOAD, internal_config);
        }
    }

    rc
}

///
pub fn handle_input(config: Config) -> UResult<()> {
    let mut rc = EXIT_SUCCESS;

    let internal_config = InternalConfig::from(&config)?;

    let mut setting_list = Vec::new();

    if config.system {
        rc |= preload_system(&mut setting_list, &internal_config);
        rc |= write_setting_list(&setting_list, &internal_config);
        match rc {
            EXIT_SUCCESS => return Ok(()),
            _ => return Err(rc.into()),
        };
    }

    if internal_config.display_all_opt {
        match display_all(PROC_PATH, &internal_config) {
            EXIT_SUCCESS => return Ok(()),
            err_code => return Err(err_code.into()),
        };
    }

    if internal_config.preload_file_opt {
        let mut ret = EXIT_SUCCESS;
        for filename in config.load {
            ret |= preload(&mut setting_list, &filename, &internal_config)
        }
        ret |= write_setting_list(&setting_list, &internal_config);
        match ret {
            EXIT_SUCCESS => return Ok(()),
            _ => return Err(ret.into()),
        };
    }

    if config.variable.is_empty() {
        errx_c(
            EXIT_FAILURE,
            &format!(
                "no variables specified\nTry `{} --help' for more information.",
                util_name()
            ),
        );
    }

    if internal_config.name_only && internal_config.quiet {
        errx_c(
            EXIT_FAILURE,
            &format!(
                "options -N and -q cannot coexist\nTry `{} --help' for more information.",
                util_name()
            ),
        );
    }

    for variable in config.variable {
        if internal_config.write_mode || variable.contains('=') {
            match parse_setting_line("command line", 0, &variable, &internal_config) {
                Some(setting) => {
                    rc |= write_setting(
                        &setting.key,
                        &setting.path,
                        &setting.value,
                        setting.ignore_failure,
                        &internal_config,
                    )
                }
                None => rc |= EXIT_FAILURE,
            }
        } else {
            rc += read_setting(&variable, &internal_config);
        }
    }

    match rc {
        EXIT_SUCCESS => Ok(()),
        _ => Err(rc.into()),
    }
}
