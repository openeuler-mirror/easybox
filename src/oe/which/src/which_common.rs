//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::env::var_os;
use std::ffi::{CString, OsStr, OsString};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

use nix::unistd::{eaccess, getcwd, geteuid, getgid, getuid, isatty, AccessFlags, Uid, User};
use nix::NixPath;
use uucore::error::{UResult, USimpleError};
use uucore::{format_usage, show_warning, util_name};

use clap::{crate_version, Arg, ArgMatches, Command};

const EXIT_FAILURE: i32 = -1;
const ADDITIONAL_HELP: &str =
    "Recommended use is to write the output of (alias; declare -f) to standard
input, so that which can show aliasess and shell functions. See which for
examples.

If the options --read-alias and/or --read-functions are specified then the
output can be a full alias or function definition, optionally followed by
the full path of each command used inside of those.";

/// Config
pub struct Config {
    /// Print all matches in PATH, not just the first.
    pub show_all: bool,
    /// Read list of aliasess from stdin.
    pub read_alias: bool,
    /// Ignore option --read-functions; don't read stdin.
    pub skip_functions: bool,
    /// Read shell functions from stdin.
    pub read_functions: bool,
    /// Stop processing options on the right if not on tty.
    pub tty_only: bool,
    /// Output a tilde for HOME directory for non-root.
    pub show_tilde: bool,
    /// Don't expand a dot to current directory in output.
    pub show_dot: bool,
    /// Ignore option --read-alias; don't read stdin.
    pub skip_alias: bool,
    /// Skip directories in PATH that start with a tilde.
    pub skip_tilde: bool,
    /// Skip directories in PATH that start with a dot.
    pub skip_dot: bool,
    /// The command list to search.
    pub command_list: Vec<String>,
    /// Current work directory
    pub cwd: OsString,
    /// Home directory
    pub home: OsString,
    /// The function start type, used in func_search
    pub function_start_type: i32,
    /// Store all functions from stdin
    pub functions: Vec<FunctionSt>,
    /// The PATH
    pub path_list: OsString,
    /// The current user info
    pub current_user: User,
}

/// Function info
pub struct FunctionSt {
    /// Name
    pub name: String,
    /// Contents
    pub lines: Vec<String>,
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
enum FileStatus {
    FS_EXISTS,
    FS_EXECABLE,
    FS_DIRECTORY,
    FS_READABLE,
}

/// Command Options
pub mod options {
    ///
    pub static SKIPDOT: &str = "skip-dot";
    ///
    pub static SKIPTILDE: &str = "skip-tilde";
    ///
    pub static SHOWDOT: &str = "show-dot";
    ///
    pub static SHOWTILDE: &str = "show-tilde";
    ///
    pub static TTYONLY: &str = "tty-only";
    ///
    pub static ALL: &str = "all";
    ///
    pub static READALIAS: &str = "read-alias";
    ///
    pub static SKIPALIAS: &str = "skip-alias";
    ///
    pub static READFUNC: &str = "read-functions";
    ///
    pub static SKIPFUNC: &str = "skip-functions";
    ///
    pub static COMMAND: &str = "command";
}

impl Config {
    /// Generate which general Config
    pub fn from(args_matches: &ArgMatches) -> UResult<Self> {
        // options::COMMAND set 'required' to true, so there at least one command exist.
        let command_list: Vec<String> = args_matches
            .get_many::<String>(options::COMMAND)
            .map(|v| v.map(String::from))
            .unwrap()
            .collect();
        let last_index = args_matches.index_of(options::COMMAND).unwrap();
        let tty_only_index = match args_matches.index_of(options::TTYONLY) {
            Some(val) => match isatty(1) {
                Ok(false) => val,
                _ => last_index,
            },
            None => last_index,
        };
        Ok(Self {
            skip_dot: match args_matches.index_of(options::SKIPDOT) {
                Some(val) => val < tty_only_index,
                None => false,
            },
            skip_tilde: match args_matches.index_of(options::SKIPTILDE) {
                Some(val) => val < tty_only_index,
                None => false,
            },
            skip_alias: args_matches.contains_id(options::SKIPALIAS),
            show_dot: match args_matches.index_of(options::SHOWDOT) {
                Some(val) => val < tty_only_index,
                None => false,
            },
            show_tilde: match args_matches.index_of(options::SHOWTILDE) {
                Some(val) => val < tty_only_index && !geteuid().is_root(),
                None => false,
            },
            tty_only: tty_only_index != last_index,
            read_functions: args_matches.contains_id(options::READFUNC),
            skip_functions: args_matches.contains_id(options::SKIPFUNC),
            show_all: args_matches.contains_id(options::ALL),
            read_alias: args_matches.contains_id(options::READALIAS),
            command_list,
            cwd: OsString::new(),
            home: OsString::new(),
            function_start_type: 0,
            functions: Vec::new(),
            path_list: var_os("PATH").unwrap_or_default(),
            current_user: uidget_get_current_user_info(),
        })
    }
}

/// Generate which general Config
pub fn parse_which_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = which_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

/// Command arguments setting
pub fn which_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg_required_else_help(true)
        // Format arguments.
        .arg(
            Arg::new(options::SKIPDOT)
                .long(options::SKIPDOT)
                .help("Skip directories in PATH that start with a dot."),
        )
        .arg(
            Arg::new(options::SKIPTILDE)
                .long(options::SKIPTILDE)
                .help("Skip directories in PATH that start with a tilde."),
        )
        .arg(
            Arg::new(options::SHOWDOT)
                .long(options::SHOWDOT)
                .help("Don't expand a dot to current directory in output."),
        )
        .arg(
            Arg::new(options::SHOWTILDE)
                .long(options::SHOWTILDE)
                .help("Output a tilde for HOME directory for non-root."),
        )
        .arg(
            Arg::new(options::TTYONLY)
                .long(options::TTYONLY)
                .help("Stop processing options on the right if not on tty."),
        )
        .arg(
            Arg::new(options::ALL)
                .long(options::ALL)
                .short('a')
                .help("Print all matches in PATH, not just the first."),
        )
        .arg(
            Arg::new(options::READALIAS)
                .long(options::READALIAS)
                .short('i')
                .help("Read list of aliasess from stdin."),
        )
        .arg(
            Arg::new(options::SKIPALIAS)
                .long(options::SKIPALIAS)
                .help("Ignore option --read-alias; don't read stdin."),
        )
        .arg(
            Arg::new(options::READFUNC)
                .long(options::READFUNC)
                .help("Read shell functions from stdin."),
        )
        .arg(
            Arg::new(options::SKIPFUNC)
                .long(options::SKIPFUNC)
                .help("Ignore option --read-functions; don't read stdin."),
        )
        .after_help(ADDITIONAL_HELP)
        .arg(
            Arg::new(options::COMMAND)
                .multiple_values(true)
                .required(true)
                .hide(true)
                .help("The command name you want to search."),
        )
        .trailing_var_arg(true)
}

fn uidget_get_current_user_info() -> User {
    if let Ok(Some(user_entry)) = User::from_uid(Uid::current()) {
        return user_entry;
    }
    User {
        name: String::from("I have no name!"),
        passwd: CString::default(),
        uid: getuid(),
        gid: getgid(),
        gecos: CString::default(),
        dir: PathBuf::from("/"),
        shell: PathBuf::from("/bin/sh"),
    }
}

/// Get current working directory from $PWD
pub fn get_current_working_directory() -> UResult<OsString> {
    let mut res: OsString;
    if let Ok(val) = getcwd() {
        res = OsString::from(val);
    } else {
        res = var_os("PWD").unwrap_or(OsString::from("error"));
    }
    let res_bytes = res.as_bytes();
    if res_bytes[0] != b'/' {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Can't get current working directory",
        ));
    }
    if res_bytes[res_bytes.len() - 1] != b'/' {
        res.push(&OsStr::from_bytes(&[b'/']));
    }
    Ok(res)
}

/// Impl getenv("HOME") and sh_get_home_dir()
pub fn get_home_dir(current_user: &User) -> PathBuf {
    match var_os("HOME") {
        Some(val) => PathBuf::from(val),
        None => current_user.dir.clone(),
    }
}

/// Impl if (read_alias || read_functions) from main()
pub fn read_alias_functions(config: &mut Config) -> UResult<()> {
    let mut buf = String::new();
    let mut processing_aliasess = config.read_alias;
    let mut aliasess = Vec::new();
    if Ok(true) == isatty(0) {
        eprintln!(
            "{}: {}: Warning: stdin is a tty.",
            util_name(),
            match config.read_functions {
                true => match config.read_alias {
                    true => "--read-functions, --read-alias, -i",
                    false => "--read-functions",
                },
                false => "--read-alias, -i",
            }
        )
    }
    while std::io::stdin().read_line(&mut buf).is_ok() {
        let mut looks_like_function_start = false;
        let mut function_start_has_declare = false;
        if buf.is_empty() {
            break;
        }
        if config.read_functions {
            // bash version 2.0.5a and older output a pattern for `str' like
            // declare -fx FUNCTION_NAME ()
            // {
            //   body
            // }
            //
            // bash version 2.0.5b and later output a pattern for `str' like
            // FUNCTION_NAME ()
            // {
            //   body
            // }
            let last_non_whitespace = buf.split_ascii_whitespace().last();
            if buf.find(" ()").is_some() && last_non_whitespace == Some("()") {
                looks_like_function_start = true;
                function_start_has_declare = buf.starts_with("declare -");
            }
            // Add some zsh support here.
            // zsh does output a pattern for `str' like
            // FUNCTION () {
            //   body
            // }
            if buf.find(" () {").is_some() && last_non_whitespace == Some("{") {
                looks_like_function_start = true;
                config.function_start_type = 1;
                function_start_has_declare = false;
            }
        }
        if processing_aliasess && !looks_like_function_start {
            // bash version 2.0.5b can throw in lines like "declare -fx
            // FUNCTION_NAME", eat them.
            if buf.starts_with("declare -") {
                continue;
            }
            aliasess.push(buf.clone());
        } else if config.read_functions && looks_like_function_start {
            processing_aliasess = false;
            let mut p = buf.split_ascii_whitespace();
            if function_start_has_declare {
                p.next(); // Eat declare
                p.next(); // Eat -fx
            }
            let name = p.next().unwrap();
            let name_string = match name {
                "()" => String::new(),
                _ => String::from(name),
            };
            let mut lines = Vec::new();
            buf.clear();
            while std::io::stdin().read_line(&mut buf).is_ok() {
                if buf.is_empty() {
                    break;
                }
                lines.push(buf.clone());
                if buf == "}\n" {
                    break;
                }
                buf.clear();
            }
            config.functions.push(FunctionSt {
                name: name_string,
                lines,
            });
        }
        buf.clear();
    }
    if config.read_alias {
        for aliases in aliasess {
            process_alias(config, aliases)?;
        }
    }
    Ok(())
}

fn process_alias(config: &mut Config, aliases: String) -> UResult<()> {
    // Remove all whitespace and " '
    let mut p = aliases.split(|v: char| v.is_ascii_whitespace() || v == '"' || v == '\'');
    let mut pstr = p.next();
    if pstr == Some("alias") {
        pstr = p.next();
    }
    if pstr.is_none() {
        return Ok(());
    }
    let mut cmd = pstr.unwrap();
    let mut pstr_has_equal = false;
    if let Some(v) = cmd.find('=') {
        cmd = &cmd[0..v];
        pstr_has_equal = true;
    }
    let mut command_list = config.command_list.clone();
    for argv in &mut command_list {
        if *argv != cmd {
            continue;
        }
        if let Err(e) = std::io::stdout().write_all(aliases.as_bytes()) {
            show_warning!("Can't write to stdout! {:?}", e);
        }
        if !config.show_all {
            argv.clear();
        }
        if pstr_has_equal {
            // 'aa=bb' or 'aa= bb'
            cmd = pstr.unwrap().split_once('=').unwrap().1;
            while cmd.is_empty() {
                match p.next() {
                    Some(v) => cmd = v,
                    None => break,
                };
            }
        } else {
            // 'aa = bb' or 'aa =bb' or 'aa bb'
            cmd = p.next().unwrap_or_default();
            if cmd.starts_with('=') {
                pstr = cmd.strip_prefix('=');
                if pstr.is_none() {
                    pstr = p.next();
                }
                cmd = pstr.unwrap_or_default();
            }
        }
        if cmd.is_empty() {
            break;
        }
        // now cmd can be 'bb' or 'bb&cc' or 'bb&'
        let and_or_judge = |c: char| c == '&' || c == '|';
        loop {
            let mut found = false;
            let mut has_andor_in_cmd = false;
            if let Some(v) = cmd.split_once(and_or_judge) {
                pstr = Some(v.1);
                cmd = v.0;
                has_andor_in_cmd = true;
            }
            if !argv.is_empty() && argv == cmd {
                argv.clear();
            }
            if config.read_functions && cmd.find('/').is_none() {
                found = func_search(true, cmd, config);
            }
            if config.show_all || !found {
                path_search(true, PathBuf::from(cmd), config)?;
            }
            if has_andor_in_cmd {
                cmd = pstr.unwrap();
                if !cmd.is_empty() {
                    continue;
                }
            }
            match p.next() {
                Some(v) => cmd = v,
                None => break,
            };
        }
        break;
    }
    config.command_list = command_list;
    Ok(())
}

/// Search cmd in shell functions
pub fn func_search(indent: bool, cmd: &str, config: &Config) -> bool {
    for i in &config.functions {
        if i.name == cmd {
            if indent {
                print!("\t");
            }
            if config.function_start_type == 1 {
                println!("{} () {{", cmd);
            } else {
                println!("{} ()", cmd);
            }
            for j in &i.lines {
                if indent {
                    print!("\t");
                }
                if let Err(e) = std::io::stdout().write_all(j.as_bytes()) {
                    show_warning!("Can't write to stdout! {:?}", e);
                }
            }
            return true;
        }
    }
    return false;
}

/// Search cmd in paths
pub fn path_search(indent: bool, cmd: PathBuf, config: &Config) -> UResult<bool> {
    let mut found_something = false;
    let mut found_path_starts_with_dot = false;
    if !config.path_list.is_empty() {
        let mut next = true;
        let mut path_index = 0;
        while next {
            next = config.show_all;
            if let Some(result) = find_command_in_path(
                &cmd,
                config,
                &mut found_path_starts_with_dot,
                &mut path_index,
            ) {
                let full_path_buf = path_clean_up(result)?;
                let mut full_path = full_path_buf.as_path();
                let in_home =
                    (config.show_tilde || config.skip_tilde) && full_path.starts_with(&config.home);
                if indent {
                    print!("\t");
                }
                if !(config.skip_tilde && in_home)
                    && config.show_dot
                    && found_path_starts_with_dot
                    && full_path.starts_with(&config.cwd)
                {
                    full_path = full_path.strip_prefix(&config.cwd).unwrap();
                    print!("./");
                } else if in_home {
                    if config.skip_tilde {
                        next = true;
                        continue;
                    }
                    if config.show_tilde {
                        full_path = full_path.strip_prefix(&config.home).unwrap();
                        print!("~/");
                    }
                }
                std::io::stdout().write_all(full_path.as_os_str().as_bytes())?;
                std::io::stdout().write_all(&[b'\n'])?;
                found_something = true;
            } else {
                break;
            }
        }
    }
    return Ok(found_something);
}

/// Find command in PATH
fn find_command_in_path(
    name_s: &Path,
    config: &Config,
    found_path_starts_with_dot: &mut bool,
    path_index: &mut usize,
) -> Option<PathBuf> {
    let mut path_list: &OsStr = &config.path_list;
    let mut name = name_s.as_os_str();
    let mut abs_path = PathBuf::new();
    if name_s.file_name().unwrap_or_default().len() < name_s.len() {
        if !name_s.starts_with(".") && !name_s.has_root() && !name_s.starts_with("~") {
            abs_path.push("./");
        }
        abs_path.push(name);
        path_list = abs_path.parent().unwrap().as_os_str();
        name = abs_path.file_name().unwrap();
    }
    if *path_index > path_list.len() {
        return None;
    }
    let path_list_bytes = &path_list.as_bytes()[*path_index..];
    for path_bytes in path_list_bytes.split(|v| *v == b':') {
        *path_index += path_bytes.len() + 1;
        let mut path = Path::new(OsStr::from_bytes(path_bytes));
        if path_bytes.is_empty() {
            path = Path::new(".");
        }
        let tlide_expand_string: PathBuf;
        if path.starts_with("~") {
            tlide_expand_string = tilde_expand(path, &config.current_user);
            path = &tlide_expand_string;
            if config.skip_tilde {
                continue;
            }
        }
        if config.skip_dot && !path.has_root() {
            continue;
        }
        *found_path_starts_with_dot = path.starts_with(".");
        let mut full_path = PathBuf::from(path);
        full_path.push(name);
        let status = file_status(&full_path);
        if status.contains(&FileStatus::FS_EXISTS) && status.contains(&FileStatus::FS_EXECABLE) {
            return Some(full_path);
        }
    }
    None
}

/* Return some flags based on information about this file.
The EXISTS bit is non-zero if the file is found.
The EXECABLE bit is non-zero the file is executable.
Zero is returned if the file is not found. */
fn file_status(path: &Path) -> Vec<FileStatus> {
    use FileStatus::*;
    /* Determine whether this file exists or not. */
    let mut r = Vec::new();
    if path.exists() {
        /* If the file is a directory, then it is not "executable" in the
        sense of the shell. */
        if path.is_dir() {
            return vec![FS_EXISTS, FS_DIRECTORY];
        }
        r.push(FS_EXISTS);
        if eaccess(path, AccessFlags::X_OK).is_ok() {
            r.push(FS_EXECABLE);
        }
        if eaccess(path, AccessFlags::R_OK).is_ok() {
            r.push(FS_READABLE);
        }
    }
    return r;
}

/// Return a new string which is the result of tilde expanding STRING.
/// Include tilde_expand_word()
fn tilde_expand(string: &Path, current_user: &User) -> PathBuf {
    let mut res = PathBuf::new();
    let head_tlide: &OsStr;
    // We are sure that the first character in string is ~
    if let Component::Normal(v) = string.components().next().unwrap() {
        head_tlide = v;
    } else {
        return string.to_path_buf();
    }
    if head_tlide.len() == 1 {
        // Just a ~
        res.push(get_home_dir(current_user));
    } else {
        // In the format ~username
        let username_bytes = &head_tlide.as_bytes()[1..];
        let mut is_found_dir = false;
        if let Ok(username) = std::str::from_utf8(username_bytes) {
            // Look in the password database.
            if let Ok(Some(user_entry)) = User::from_name(username) {
                res.push(user_entry.dir);
                is_found_dir = true;
            }
        }
        if !is_found_dir {
            res.push(head_tlide);
        }
    }
    res.push(string.strip_prefix(head_tlide).unwrap());
    res
}

fn path_clean_up(path: PathBuf) -> UResult<PathBuf> {
    let mut result = PathBuf::new();
    if !path.has_root() {
        result.push(get_current_working_directory()?);
    }
    for i in path.components() {
        match i {
            Component::RootDir => result.push(Component::RootDir),
            Component::Normal(v) => result.push(v),
            Component::ParentDir => {
                if !result.pop() {
                    return Ok(path);
                }
            }
            _ => (),
        }
    }
    Ok(result)
}
