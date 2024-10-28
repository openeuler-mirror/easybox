//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::find_common::metadata::FindMetadata;
use std::borrow::Cow;
use std::env::current_dir;
use std::fmt::Debug;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::find_common::get_metadata;
use crate::this_filter_consume_no_args;
use crate::this_filter_has_side_effects;
use crate::this_filter_is_based_on_metadata;

use super::tests::is_follow_link_enabled_when_build;
use super::Config;
use super::FindConstruct;
use super::FindFile;
use super::FindFilter;
use super::FindInstruction;
use super::FindOption;
use chrono::DateTime;
use chrono::Local;
use uucore::error::UResult;
use uucore::error::USimpleError;

const YESEXPR: &str = r"[1yY]";
const NOEXPR: &str = r"[0nN]";

const CTIME_FORMAT: &str = "%a %b %d %H:%M:%S %Y";

/// Delete this file. Implies -depth (content-first)
#[derive(Debug)]
pub struct Delete;

impl Delete {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for Delete {
    fn default() -> Self {
        Self::new()
    }
}

impl FindOption for Delete {
    fn take_effect(&self, config: &mut super::Config) -> uucore::error::UResult<()> {
        config.global_option.depth = true;
        Ok(())
    }
}

impl FindFilter for Delete {
    fn filter(&mut self, _file: &FindFile) -> uucore::error::UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        _side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        std::fs::remove_file(file.get_path())?;
        Ok(true)
    }

    this_filter_has_side_effects!();
}

impl FindConstruct for Delete {
    this_filter_consume_no_args!();
}

#[derive(Debug)]
///
pub struct Exec {
    commands: Vec<String>,

    change_dir: bool,
    dir: PathBuf,

    prompt: bool,
    append: bool,

    file_cache: Vec<PathBuf>,
    current_args: i64,
    arg_max: Option<i64>,

    debug: bool,
}

impl Exec {
    ///
    pub fn new(
        commands: Vec<String>,
        change_dir: bool,
        prompt: bool,
        append: bool,
        config: &Config,
    ) -> Self {
        Self {
            commands,
            change_dir,
            dir: current_dir().unwrap(),
            prompt,
            append,

            file_cache: vec![],
            current_args: 0,
            arg_max: config.global_option.arg_max,

            debug: config.debug_exec,
        }
    }

    ///
    pub fn enable_dir(&mut self) {
        self.change_dir = true;
    }

    ///
    pub fn enable_prompt(&mut self) {
        assert!(!self.append);
        self.prompt = true;
    }
}

impl FindFilter for Exec {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        let path = file.get_path();
        let (path, dir) = if self.change_dir {
            (
                path.file_name().ok_or(USimpleError::new(
                    1,
                    format!("Cannot get file name of `{}`", path.to_string_lossy()),
                ))?,
                path.parent()
                    .map(|p| p.to_owned())
                    .ok_or(USimpleError::new(
                        1,
                        format!("Cannot get work dir for `{}`", path.to_string_lossy()),
                    ))?,
            )
        } else {
            (path.as_os_str(), self.dir.clone())
        };
        let path = path.to_string_lossy();

        if self.append {
            let path_len = path.len() as i64;

            if self.arg_max.is_none()
                || self.current_args + path_len > self.arg_max.unwrap()
                || dir != self.dir
            {
                let len = self.commands.len();
                let program = &self.commands[0];
                let args = &self.commands[1..len - 1];
                let args: Vec<_> = args
                    .iter()
                    .cloned()
                    .chain(
                        self.file_cache
                            .iter()
                            .map(|file| file.to_string_lossy().to_string()),
                    )
                    .collect();

                let mut child = Command::new(program)
                    .args(&args)
                    .current_dir(dir.clone())
                    .spawn()?;

                let pid = child.id();
                if self.debug {
                    eprintln!("Debug exec: Launching process (PID: {pid}): {program}, {args:?}");
                }

                let status = child.wait()?;
                if !status.success() {
                    side_effects.push(FindInstruction::Exit(Some(status.code().unwrap_or(1))));
                }

                if self.debug {
                    eprintln!(
                        "Debug exec: Process (PID: {pid}) exited with status: {}",
                        status.code().unwrap_or(0)
                    );
                }
                self.file_cache = vec![PathBuf::from(&*path)];
                self.current_args = path_len;
                self.dir = dir;
            } else {
                self.file_cache.push(PathBuf::from(&*path));
                self.current_args += path_len;
            }

            Ok(true)
        } else {
            let pattern =
                regex::Regex::new(r"\{\}").map_err(|e| USimpleError::new(1, e.to_string()))?;
            let commands: Vec<Cow<str>> = self
                .commands
                .iter()
                .map(|arg| pattern.replace_all(arg, &path))
                .collect();

            if commands.is_empty() {
                Err(USimpleError::new(1, "The command is empty"))
            } else {
                let program = &commands[0];
                let args = &commands[1..];

                if self.prompt {
                    eprint!(
                        "{program} {} ({YESEXPR}/{NOEXPR}):",
                        args.iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    let mut stdin = std::io::stdin().lock();
                    let mut res = String::new();
                    stdin.read_line(&mut res)?;
                    let yre = regex::Regex::new(YESEXPR).unwrap();

                    if !yre.is_match(&res) {
                        return Ok(false);
                    }
                }

                let mut child = Command::new(program.as_ref())
                    .args(args.iter().map(|arg| arg.as_ref()))
                    .current_dir(dir)
                    .spawn()?;
                let pid = child.id();
                if self.debug {
                    eprintln!("Debug exec: Launching process (PID: {pid}): {program}, {args:?}");
                }

                let status = child.wait()?;
                if self.debug {
                    eprintln!(
                        "Debug exec: Process (PID: {pid}) exited with status: {}",
                        status.code().unwrap_or(0)
                    );
                }
                Ok(status.success())
            }
        }
    }

    this_filter_has_side_effects!();
}

impl Drop for Exec {
    fn drop(&mut self) {
        if self.append && !self.file_cache.is_empty() {
            let len = self.commands.len();
            let program = &self.commands[0];
            let args = &self.commands[1..len - 1];
            Command::new(program)
                .args(
                    args.iter().cloned().chain(
                        self.file_cache
                            .iter()
                            .map(|file| file.to_string_lossy().to_string()),
                    ),
                )
                .current_dir(self.dir.clone())
                .status()
                .unwrap();
        }
    }
}

impl FindConstruct for Exec {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        let mut commands = vec![];
        let program = iter
            .next()
            .ok_or(USimpleError::new(1, "No program name for -exec..."))?;
        commands.push(program);

        for arg in iter {
            match arg.as_str() {
                ";" => return Ok(Self::new(commands, false, false, false, config)),
                "+" => {
                    let len = commands.len();
                    return if commands[len - 1] != "{}" {
                        Err(USimpleError::new(1, "The last arg before `+` must be `{}`"))
                    } else {
                        Ok(Self::new(commands, false, false, true, config))
                    };
                }
                arg => commands.push(arg.to_string()),
            }
        }

        Err(USimpleError::new(
            1,
            "No `;` or `+` after the command for -exec...",
        ))
    }
}

#[derive(Debug)]
///
pub struct ExecDir {
    inner: Exec,
}

impl FindFilter for ExecDir {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        self.inner.filter_with_side_effects(file, side_effects)
    }

    this_filter_has_side_effects!();
}

impl FindConstruct for ExecDir {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        let mut inner = Exec::construct_from_iter_with_config(iter, config)?;
        inner.enable_dir();

        Ok(Self { inner })
    }
}

/// A Wrapper of exec. Ask user before running the command.
#[derive(Debug)]
pub struct OkExec {
    inner: Exec,
}

impl FindFilter for OkExec {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        self.inner.filter_with_side_effects(file, side_effects)
    }

    this_filter_has_side_effects!();
}

impl FindConstruct for OkExec {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        if !config.from_cli {
            return Err(USimpleError::new(1, "Cannot combine -ok with -files0-from"));
        }

        let mut inner = Exec::construct_from_iter_with_config(iter, config)?;
        inner.enable_prompt();

        Ok(Self { inner })
    }
}

impl FindOption for OkExec {
    fn take_effect(&self, config: &mut Config) -> UResult<()> {
        config.has_ok = true;
        Ok(())
    }
}

#[derive(Debug)]
///
pub struct OkExecDir {
    inner: Exec,
}

impl FindFilter for OkExecDir {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        self.inner.filter_with_side_effects(file, side_effects)
    }

    this_filter_has_side_effects!();
}

impl FindConstruct for OkExecDir {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        let mut inner = Exec::construct_from_iter_with_config(iter, config)?;
        inner.enable_dir();
        inner.enable_prompt();

        Ok(Self { inner })
    }
}

#[derive(Debug)]
///
pub enum OutputTarget {
    ///
    Stdout,

    ///
    File(PathBuf),
}

impl OutputTarget {
    ///
    pub fn output(&self, s: &str) -> UResult<()> {
        match self {
            OutputTarget::Stdout => {
                print!("{}", s);
                std::io::stdout().flush().unwrap();
                Ok(())
            }
            OutputTarget::File(ref path) => {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                f.write_all(s.as_bytes())?;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
struct LsInner {
    target: OutputTarget,
    posixly_correct: bool,
    follow_link: bool,
}

impl LsInner {
    pub fn new(target: OutputTarget, posixly_correct: bool, follow_link: bool) -> Self {
        Self {
            target,
            posixly_correct,
            follow_link,
        }
    }
}

impl LsInner {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).and_then(|m| {
            let inode = m.st_ino();

            // Display blocks in 1KB defaultly, but 512 bytes when POSIXLY_CORRECT is set.
            let blocks = if self.posixly_correct {
                m.st_block()
            } else {
                m.st_block() >> 1
            };

            let size = m.st_len();

            let file_type = file_type_symbol(m.st_mode());
            let perm = m.st_mode();
            let perm = format!("{:o}", perm & 0o777);

            let nlink = m.st_nlink();

            let uid = m.st_uid();
            let gid = m.st_gid();
            let user = users::get_user_by_uid(uid).ok_or(USimpleError::new(
                1,
                format!("Cannot get corresponding user name for uid {}", uid),
            ))?;
            let user = user.name().to_string_lossy().to_string();
            let group = users::get_group_by_gid(gid).ok_or(USimpleError::new(
                1,
                format!("Cannot get corresponding group name for gid {}", gid),
            ))?;
            let group = group.name().to_string_lossy().to_string();

            let duration_since_epoch = Duration::from_secs(m.st_mtime() as u64);
            let modified_time = SystemTime::UNIX_EPOCH + duration_since_epoch;
            let modified_datetime: chrono::DateTime<chrono::Local> = modified_time.into();

            self.target.output(&format!(
                "{} {} {}{} {} {} {} {} {} {}\n",
                inode,
                blocks,
                file_type,
                perm,
                nlink,
                user,
                group,
                size,
                modified_datetime.format("%b %e %H:%M"),
                file.get_path().to_string_lossy().escape_default(),
            ))?;

            Ok(true)
        })
    }
}

#[derive(Debug)]
///
pub struct Ls {
    inner: LsInner,
}

impl FindFilter for Ls {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        self.inner.filter(file)
    }

    this_filter_has_side_effects!();
    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Ls {
    fn construct_from_iter_with_config(
        _iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        Ok(Self {
            inner: LsInner::new(
                OutputTarget::Stdout,
                config.global_option.posixly_correct,
                is_follow_link_enabled_when_build(config),
            ),
        })
    }
}

#[derive(Debug)]
///
pub struct FLs {
    inner: LsInner,
}

impl FindFilter for FLs {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        self.inner.filter(file)
    }

    this_filter_has_side_effects!();
    this_filter_is_based_on_metadata!();
}

impl FindConstruct for FLs {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -fls"))
            .map(|arg| Self {
                inner: LsInner::new(
                    OutputTarget::File(arg.into()),
                    config.global_option.posixly_correct,
                    is_follow_link_enabled_when_build(config),
                ),
            })
    }
}

///
pub trait Format: Sized {
    ///
    fn format(&self, file: &FindFile) -> UResult<String>;

    ///
    fn construct(iter: &mut impl Iterator<Item = String>, config: &Config) -> UResult<Self>;
}

///
pub mod format {
    use std::{ffi::OsStr, fs::read_link};

    use chrono::{DateTime, Local, TimeZone};
    use uucore::error::{UResult, USimpleError};

    use super::{
        format_time, mode_bits_to_file_type, mode_bits_to_numbers, mode_bits_to_symbols, Format,
        CTIME_FORMAT,
    };
    use crate::find_common::{
        get_metadata,
        tests::{get_filesystem_name, is_follow_link_enabled_when_build},
        Config, FindFile,
    };

    ///
    pub fn has_ascii_special_characters(s: &str) -> bool {
        s.chars().map(|c| c as u8).any(|c| c <= 0x1f || c == 0x7f)
    }

    /// Quote the filename if it has ascii special characters.
    fn filename(s: &str) -> String {
        if has_ascii_special_characters(s) {
            format!("'{}'", s.escape_default())
        } else {
            s.to_string()
        }
    }

    ///
    pub fn unescape(s: &str) -> String {
        let re = regex::Regex::new(r"\\([0-7]{1,3}|.)").unwrap();
        let escape_c_re = regex::Regex::new(r"\\c.*$").unwrap();

        // Mimic '\c' escape sequence.
        let s = escape_c_re.replace(s, "");

        re.replace_all(&s, |caps: &regex::Captures| match &caps[1] {
            "a" => "\x07".to_string(),
            "b" => "\x08".to_string(),
            "c" => panic!("Unsupported escape sequence: \\c"),
            "f" => "\x0c".to_string(),
            "n" => "\n".to_string(),
            "r" => "\r".to_string(),
            "t" => "\t".to_string(),
            "v" => "\x0b".to_string(),
            "0" => "\0".to_string(),
            "\\" => "\\".to_string(),

            s => match u8::from_str_radix(s, 8) {
                Ok(c) => {
                    let c = c as char;
                    c.to_string()
                }
                Err(_) => format!("\\{s}"),
            },
        })
        .to_string()
    }

    fn ctime_timestamp(timestamp: i64) -> String {
        let datetime: DateTime<Local> = Local.timestamp_opt(timestamp, 0).unwrap();
        datetime.format(CTIME_FORMAT).to_string()
    }

    // BUG: No support for flag/precision/width specifiers.
    // `printf_compat` crate may help.
    ///
    pub fn unescape_metadata(s: &str, file: &FindFile, follow_link: bool) -> UResult<String> {
        get_metadata(file, follow_link).map(|metadata| {
            let re = regex::Regex::new(r"%([%abcdDfFgGhHiklmMnpPsStuUyYZ]|[ABCT].)").unwrap();
            re.replace_all(s, |caps: &regex::Captures| {
                let field = &caps[1];
                match field {
                    "%" => "%".to_string(),
                    "a" => ctime_timestamp(metadata.st_atime()),
                    "b" => format!("{}", metadata.st_block()),
                    "c" => ctime_timestamp(metadata.st_ctime()),
                    "d" => format!("{}", file.depth),
                    "D" => format!("{}", metadata.st_dev()),
                    "f" => format!(
                        "{}",
                        file.get_path()
                            .file_name()
                            .unwrap_or(OsStr::new(""))
                            .to_string_lossy()
                    ),
                    "F" => get_filesystem_name(metadata.st_dev()).unwrap_or("Unknown".to_owned()),
                    "g" => metadata
                        .st_gname()
                        .unwrap_or(format!("{}", metadata.st_gid())),
                    "G" => format!("{}", metadata.st_gid()),
                    "h" => file
                        .get_path()
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(".".to_owned()),
                    "H" => file.starting_point.to_string_lossy().to_string(),
                    "i" => format!("{}", metadata.st_ino()),
                    "k" => format!("{}", metadata.st_block() >> 1),
                    "l" => read_link(file.get_path())
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or("".to_string()),
                    "m" => mode_bits_to_numbers(metadata.st_mode()),
                    "M" => mode_bits_to_symbols(metadata.st_mode()),
                    "n" => format!("{}", metadata.st_nlink()),
                    "p" => file.get_path().to_string_lossy().to_string(),
                    "P" => {
                        let starting_point = file.starting_point.to_string_lossy();
                        let path = file.get_path().to_string_lossy();
                        path.strip_prefix(starting_point.as_ref())
                            .map(str::to_string)
                            .unwrap_or(path.to_string())
                    }
                    "s" => format!("{}", metadata.st_len()),
                    "S" => {
                        let block_size = metadata.st_blksize() as f64;
                        let blocks = metadata.st_block() as f64;
                        let size = metadata.st_len() as f64;

                        format!("{}", block_size * blocks / size)
                    }
                    "t" => ctime_timestamp(metadata.st_mtime()),
                    "u" => metadata
                        .st_uname()
                        .unwrap_or(format!("{}", metadata.st_uid())),
                    "U" => format!("{}", metadata.st_uid()),
                    "y" => format!("{}", mode_bits_to_file_type(metadata.st_mode())),
                    "Y" => match file.get_path().read_link() {
                        Ok(p) => {
                            if p == file.get_path() {
                                "L".to_string()
                            } else {
                                mode_bits_to_file_type(
                                    file.get_pointed_metadata().unwrap().st_mode(),
                                )
                                .to_string()
                            }
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                "N".to_string()
                            } else {
                                "?".to_string()
                            }
                        }
                    },

                    // #[cfg(feature = "selinux")]
                    // "Z" => selinux::SecurityContext::of_path(file.get_path(), follow_link, false)
                    //     .map(|ctx| {
                    //         ctx.map(|ctx| {
                    //             ctx.to_c_string()
                    //                 .map(|ctx| {
                    //                     ctx.map(|s| s.to_string_lossy().to_string())
                    //                         .unwrap_or("".to_string())
                    //                 })
                    //                 .unwrap_or("".to_string())
                    //         })
                    //         .unwrap_or("".to_string())
                    //     })
                    //     .unwrap_or("".to_string()),
                    s if s.len() == 2 => {
                        let s = s.as_bytes();
                        let (type_specifier, format_specifier) = (s[0] as char, s[1] as char);
                        format_time(metadata, type_specifier, format_specifier)
                            .unwrap_or("".to_string())
                    }

                    s => panic!("Format string cannot process {s}"),
                }
            })
            .to_string()
        })
    }

    #[derive(Debug)]
    ///
    pub struct NewLine;

    impl NewLine {
        ///
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for NewLine {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Format for NewLine {
        fn format(&self, file: &FindFile) -> UResult<String> {
            let path = file.get_path();
            let path = path.to_string_lossy();

            Ok(format!("{}\n", filename(&path)))
        }

        fn construct(_iter: &mut impl Iterator<Item = String>, _config: &Config) -> UResult<Self> {
            Ok(Self)
        }
    }

    #[derive(Debug)]
    ///
    pub struct NullTerminated;

    impl NullTerminated {
        ///
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for NullTerminated {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Format for NullTerminated {
        fn format(&self, file: &FindFile) -> UResult<String> {
            let path = file.get_path();
            Ok(format!("{}\0", path.to_string_lossy()))
        }

        fn construct(_iter: &mut impl Iterator<Item = String>, _config: &Config) -> UResult<Self> {
            Ok(Self)
        }
    }

    #[derive(Debug)]
    ///
    pub struct FormatString {
        format: String,
        follow_link: bool,
    }

    impl FormatString {
        ///
        pub fn new(format: &str, follow_link: bool) -> Self {
            Self {
                format: unescape(format),
                follow_link,
            }
        }
    }

    impl Format for FormatString {
        fn format(&self, file: &FindFile) -> UResult<String> {
            unescape_metadata(&self.format, file, self.follow_link)
        }

        fn construct(iter: &mut impl Iterator<Item = String>, config: &Config) -> UResult<Self> {
            iter.next()
                .ok_or(USimpleError::new(1, "No format string"))
                .map(|arg| Self::new(&arg, is_follow_link_enabled_when_build(config)))
        }
    }
}

/// Print file name with format and output destination;
#[derive(Debug)]
pub struct Print<F: Format> {
    target: OutputTarget,
    formatter: F,
}

impl<F: Format> Print<F> {
    ///
    pub fn new(target: OutputTarget, formatter: F) -> Self {
        Self { target, formatter }
    }

    ///
    pub fn to_file(mut self, file: PathBuf) -> Self {
        self.target = OutputTarget::File(file);
        self
    }
}

impl<F: Format + Debug> FindFilter for Print<F> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let output = self.formatter.format(file)?;
        self.target.output(&output)?;

        Ok(true)
    }

    this_filter_has_side_effects!();
}

impl<F: Format + Debug> FindConstruct for Print<F> {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        F::construct(iter, config).map(|formatter| Self::new(OutputTarget::Stdout, formatter))
    }
}

#[derive(Debug)]
///
pub struct FilePrint<F: Format> {
    inner: Print<F>,
}

impl<F: Format> FilePrint<F> {
    ///
    pub fn new(target: OutputTarget, formatter: F) -> Self {
        Self {
            inner: Print::new(target, formatter),
        }
    }
}

impl<F: Format + Debug> FindFilter for FilePrint<F> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let output = self.inner.formatter.format(file)?;
        self.inner.target.output(&output)?;

        Ok(true)
    }

    this_filter_has_side_effects!();
}

impl<F: Format + Debug> FindConstruct for FilePrint<F> {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No file for -fprint..."))
            .and_then(|file| {
                F::construct(iter, config)
                    .map(|formatter| Self::new(OutputTarget::File(file.into()), formatter))
            })
    }
}

/// Skip sub directories.
#[derive(Debug)]
pub struct Prune;

impl Default for Prune {
    fn default() -> Self {
        Self::new()
    }
}

impl Prune {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl FindFilter for Prune {
    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        side_effects.push(FindInstruction::Prune);
        self.filter(file)
    }

    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        Ok(true)
    }

    this_filter_has_side_effects!();
}

impl FindConstruct for Prune {
    this_filter_consume_no_args!();
}

/// Do what its name says.
#[derive(Debug)]
pub struct Quit;

impl Default for Quit {
    fn default() -> Self {
        Self::new()
    }
}

impl Quit {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl FindFilter for Quit {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        std::process::exit(0)
    }

    this_filter_has_side_effects!();
}

impl FindConstruct for Quit {
    this_filter_consume_no_args!();
}

fn mode_bits_to_numbers(mode: u32) -> String {
    let mode = mode & 0o777;
    format!("{:o}", mode)
}

fn mode_bits_to_symbols(perm: u32) -> String {
    let mut chars = ['-'; 10];

    chars[0] = mode_bits_to_file_type(perm);

    // User
    if perm & 0o400 != 0 {
        chars[1] = 'r';
    }
    if perm & 0o200 != 0 {
        chars[2] = 'w';
    }
    if perm & 0o100 != 0 {
        if perm & 0o4000 != 0 {
            chars[3] = 's';
        } else {
            chars[3] = 'x';
        }
    } else if perm & 0o4000 != 0 {
        chars[3] = 'S';
    }

    // Group
    if perm & 0o40 != 0 {
        chars[4] = 'r';
    }
    if perm & 0o20 != 0 {
        chars[5] = 'w';
    }
    if perm & 0o10 != 0 {
        if perm & 0o2000 != 0 {
            chars[6] = 's';
        } else {
            chars[6] = 'x';
        }
    } else if perm & 0o2000 != 0 {
        chars[6] = 'S';
    }

    // Other
    if perm & 0o4 != 0 {
        chars[7] = 'r';
    }
    if perm & 0o2 != 0 {
        chars[8] = 'w';
    }
    if perm & 0o1 != 0 {
        if perm & 0o1000 != 0 {
            chars[9] = 't';
        } else {
            chars[9] = 'x';
        }
    } else if perm & 0o1000 != 0 {
        chars[9] = 'T';
    }

    chars.iter().collect()
}

fn mode_bits_to_file_type(mode: u32) -> char {
    match (mode & 0o170000) >> 12 {
        0o01 => 'p',
        0o02 => 'c',
        0o04 => 'd',
        0o06 => 'b',
        0o10 => '-',
        0o12 => 'l',
        0o14 => 's',
        _ => '?',
    }
}

fn format_time(
    metadata: &dyn FindMetadata,
    type_specifier: char,
    format_specifier: char,
) -> Option<String> {
    let timestamp = match type_specifier {
        'A' => Some(metadata.st_atime()),
        'B' => None,
        'C' => Some(metadata.st_ctime()),
        'T' => Some(metadata.st_mtime()),
        _ => None,
    }?;
    let time = if timestamp > 0 {
        UNIX_EPOCH + Duration::from_secs(timestamp as u64)
    } else {
        UNIX_EPOCH - Duration::from_secs((-timestamp) as u64)
    };

    let datetime: DateTime<Local> = time.into();
    Some(format!(
        "{}",
        datetime.format(&format!("%{format_specifier}"))
    ))
}

fn file_type_symbol(mode: u32) -> char {
    match mode & 0o170000 {
        0o040000 => 'd',
        0o100000 => '-',
        0o120000 => 'l',
        0o010000 => 'p',
        0o020000 => 'c',
        0o060000 => 'b',
        0o140000 => 's',
        _ => '?',
    }
}
