//! This file is part of the easybox package.
//
// Copyright 2021 Collabora, Ltd.
// This file incorporates code from findutils developed by Collabora, Ltd.
// The original code is licensed under the MIT License.
// The original code can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//
// Changes made to the original code are part of the easybox package.
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, ArgMatches};
use shell_quote::{Bash, QuoteExt};
use std::{
    collections::HashMap,
    error::Error,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::{self},
    io::{self, BufRead, BufReader, Read, Write},
    os::unix::ffi::{OsStrExt, OsStringExt},
    process::{Command, Stdio},
};
use uucore::{
    error::{UResult, USimpleError},
    format_usage,
};

/// Options
pub struct Options {
    ///
    pub arg_file: Option<String>,
    ///
    pub delimiter: Option<u8>,
    ///
    pub exit_if_pass_char_limit: bool,
    ///
    pub max_args: Option<usize>,
    ///
    pub open_tty: bool,
    ///
    pub max_lines: Option<usize>,
    ///
    pub interactive: bool,
    ///
    pub no_run_if_empty: bool,
    ///
    pub null: bool,
    ///
    pub size: Option<usize>,
    ///
    pub verbose: bool,
    ///
    pub replace_pat: Option<String>,
    ///
    pub eof: Option<Vec<u8>>,
}

/// Command Options
pub mod options {
    ///
    pub static COMMAND: &str = "COMMAND";
    ///
    pub static ARG_FILE: &str = "arg-file";
    ///
    pub static DELIMITER: &str = "delimiter";
    ///
    pub static EXIT: &str = "exit";
    ///
    pub static MAX_ARGS: &str = "max-args";
    ///
    pub static OPEN_TTY: &str = "open-tty";
    ///
    pub static MAX_LINES: &str = "max-lines";
    ///
    pub static MAX_PROCS: &str = "max-procs";
    ///
    pub static INTERACTIVE: &str = "interactive";
    ///
    pub static NO_RUN_IF_EMPTY: &str = "no-run-if-empty";
    ///
    pub static NULL: &str = "null";
    ///
    pub static SIZE: &str = "size";
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static REPLACE: &str = "replace";
    ///
    pub static EOF: &str = "eof";
}

impl Options {
    /// Generate taskset general Options
    pub fn from(args_matches: &clap::ArgMatches) -> UResult<Self> {
        let arg_file = args_matches
            .value_of(options::ARG_FILE)
            .map(|value| value.to_owned());
        let delimiter = args_matches
            .value_of(options::DELIMITER)
            .map(|value| parse_delimiter(value).unwrap());
        let exit_if_pass_char_limit = args_matches.is_present(options::EXIT);
        let max_args = args_matches
            .value_of(options::MAX_ARGS)
            .map(|value| value.parse().unwrap());
        let open_tty = args_matches.is_present(options::OPEN_TTY);
        let max_lines = if args_matches.occurrences_of(options::MAX_LINES) > 0
            && args_matches.value_of(options::MAX_LINES).is_none()
        {
            Some(1)
        } else {
            args_matches
                .value_of(options::MAX_LINES)
                .map(|s| s.parse().expect("Failed to parse value"))
        };
        // The commented out code below is to obtain the value of max_procs, but the -P option cannot currently be implemented, so it is commented out.
        // let max_procs = args_matches
        //     .value_of(options::MAX_PROCS)
        //     .map(|value| value.parse().unwrap());
        let interactive = args_matches.is_present(options::INTERACTIVE);
        let no_run_if_empty = args_matches.is_present(options::NO_RUN_IF_EMPTY);
        let null = args_matches.is_present(options::NULL);
        let size = args_matches
            .value_of(options::SIZE)
            .map(|value| value.parse().unwrap());
        let verbose = args_matches.get_flag(options::VERBOSE);
        let replace_pat = if args_matches.is_present(options::REPLACE) {
            Some(
                args_matches
                    .value_of(options::REPLACE)
                    .map(|value| value.to_owned())
                    .unwrap_or("{}".to_string()),
            )
        } else {
            None
        };
        let eof = if args_matches.is_present(options::EOF) {
            args_matches
                .value_of(options::EOF)
                .map(|eof_str| eof_str.as_bytes().to_vec())
        } else {
            None
        };
        Ok(Self {
            arg_file,
            delimiter,
            exit_if_pass_char_limit,
            max_args,
            open_tty,
            max_lines,
            interactive,
            no_run_if_empty,
            null,
            size,
            verbose,
            replace_pat,
            eof,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgumentKind {
    Initial,
    HardTerminated,
    SoftTerminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Argument {
    arg: OsString,
    kind: ArgumentKind,
}

struct ExhaustedCommandSpace {
    arg: Argument,
    out_of_chars: bool,
}

trait CommandSizeLimiter {
    fn try_arg(
        &mut self,
        arg: Argument,
        cursor: LimiterCursor<'_>,
    ) -> Result<Argument, ExhaustedCommandSpace>;
    fn dyn_clone(&self) -> Box<dyn CommandSizeLimiter>;
}

struct LimiterCursor<'collection> {
    limiters: &'collection mut [Box<dyn CommandSizeLimiter>],
}

impl LimiterCursor<'_> {
    fn try_next(self, arg: Argument) -> Result<Argument, ExhaustedCommandSpace> {
        if self.limiters.is_empty() {
            Ok(arg)
        } else {
            let (current, remaining) = self.limiters.split_at_mut(1);
            current[0].try_arg(
                arg,
                LimiterCursor {
                    limiters: remaining,
                },
            )
        }
    }
}

struct LimiterCollection {
    limiters: Vec<Box<dyn CommandSizeLimiter>>,
}

impl LimiterCollection {
    fn new() -> Self {
        Self { limiters: vec![] }
    }

    fn add(&mut self, limiter: impl CommandSizeLimiter + 'static) {
        self.limiters.push(Box::new(limiter));
    }

    fn try_arg(&mut self, arg: Argument) -> Result<Argument, ExhaustedCommandSpace> {
        let cursor = LimiterCursor {
            limiters: &mut self.limiters[..],
        };
        cursor.try_next(arg)
    }
}

impl Clone for LimiterCollection {
    fn clone(&self) -> Self {
        Self {
            limiters: self
                .limiters
                .iter()
                .map(|limiter| limiter.dyn_clone())
                .collect(),
        }
    }
}

#[cfg(windows)]
fn count_osstr_chars_for_exec(s: &OsStr) -> usize {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().count() + 1
}

#[cfg(unix)]
fn count_osstr_chars_for_exec(s: &OsStr) -> usize {
    s.as_bytes().len() + 1
}

#[derive(Clone)]
struct MaxCharsCommandSizeLimiter {
    current_size: usize,
    max_chars: usize,
}

impl MaxCharsCommandSizeLimiter {
    fn new(max_chars: usize) -> Self {
        Self {
            current_size: 0,
            max_chars,
        }
    }

    #[cfg(windows)]
    fn new_system(_env: &HashMap<OsString, OsString>) -> MaxCharsCommandSizeLimiter {
        const MAX_CMDLINE: usize = 32767;
        MaxCharsCommandSizeLimiter::new(MAX_CMDLINE)
    }

    #[cfg(unix)]
    fn new_system(env: &HashMap<OsString, OsString>) -> Self {
        const ARG_HEADROOM: usize = 2048;
        let arg_max = unsafe { uucore::libc::sysconf(uucore::libc::_SC_ARG_MAX) } as usize;

        let env_size: usize = env
            .iter()
            .map(|(var, value)| count_osstr_chars_for_exec(var) + count_osstr_chars_for_exec(value))
            .sum();

        Self::new(arg_max - ARG_HEADROOM - env_size)
    }
}

impl CommandSizeLimiter for MaxCharsCommandSizeLimiter {
    fn try_arg(
        &mut self,
        arg: Argument,
        cursor: LimiterCursor<'_>,
    ) -> Result<Argument, ExhaustedCommandSpace> {
        let chars = count_osstr_chars_for_exec(&arg.arg);
        if self.current_size + chars <= self.max_chars {
            let arg = cursor.try_next(arg)?;
            self.current_size += chars;
            Ok(arg)
        } else {
            Err(ExhaustedCommandSpace {
                arg,
                out_of_chars: true,
            })
        }
    }

    fn dyn_clone(&self) -> Box<dyn CommandSizeLimiter> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct MaxArgsCommandSizeLimiter {
    current_args: usize,
    max_args: usize,
}

impl MaxArgsCommandSizeLimiter {
    fn new(max_args: usize) -> Self {
        Self {
            current_args: 0,
            max_args,
        }
    }
}

impl CommandSizeLimiter for MaxArgsCommandSizeLimiter {
    fn try_arg(
        &mut self,
        arg: Argument,
        cursor: LimiterCursor<'_>,
    ) -> Result<Argument, ExhaustedCommandSpace> {
        if self.current_args < self.max_args {
            let arg = cursor.try_next(arg)?;
            if arg.kind != ArgumentKind::Initial {
                self.current_args += 1;
            }
            Ok(arg)
        } else {
            Err(ExhaustedCommandSpace {
                arg,
                out_of_chars: false,
            })
        }
    }

    fn dyn_clone(&self) -> Box<dyn CommandSizeLimiter> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct MaxLinesCommandSizeLimiter {
    current_line: usize,
    max_lines: usize,
}

impl MaxLinesCommandSizeLimiter {
    fn new(max_lines: usize) -> Self {
        Self {
            current_line: 1,
            max_lines,
        }
    }
}

impl CommandSizeLimiter for MaxLinesCommandSizeLimiter {
    fn try_arg(
        &mut self,
        arg: Argument,
        cursor: LimiterCursor<'_>,
    ) -> Result<Argument, ExhaustedCommandSpace> {
        if self.current_line <= self.max_lines {
            let arg = cursor.try_next(arg)?;
            if arg.kind == ArgumentKind::HardTerminated {
                self.current_line += 1;
            }
            Ok(arg)
        } else {
            Err(ExhaustedCommandSpace {
                arg,
                out_of_chars: false,
            })
        }
    }

    fn dyn_clone(&self) -> Box<dyn CommandSizeLimiter> {
        Box::new(self.clone())
    }
}
/// The `CommandResult` enum is used to represent the outcome of a command execution.
pub enum CommandResult {
    /// Indicates that the command was executed successfully.
    Success,
    /// Indicates that the command failed to execute correctly.
    Failure,
}

impl CommandResult {
    fn combine(&mut self, other: Self) {
        if matches!(*self, CommandResult::Success) {
            *self = other;
        }
    }
}
/// Describes a failure in command execution.
#[derive(Debug)]
pub enum CommandExecutionError {
    /// exit code 255
    UrgentlyFailed,
    /// The command was killed by a signal.
    Killed {
        /// `signal` is the number that identifies the signal that caused the termination.
        signal: i32,
    },
    /// The command could not be run, usually due to an I/O error.
    CannotRun(io::Error),
    /// The command was not found in the path.
    NotFound,
    /// An unknown error occurred during command execution.
    Unknown,
}

impl Display for CommandExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandExecutionError::UrgentlyFailed => write!(f, "Command exited with code 255"),
            CommandExecutionError::Killed { signal } => {
                write!(f, "Command was killed with signal {signal}")
            }
            CommandExecutionError::CannotRun(err) => write!(f, "Command could not be run: {err}"),
            CommandExecutionError::NotFound => write!(f, "Command not found"),
            CommandExecutionError::Unknown => write!(f, "Unknown error running command"),
        }
    }
}

impl Error for CommandExecutionError {}

enum ExecAction {
    Command(Vec<OsString>),
    Echo,
}

struct CommandBuilderOptions {
    action: ExecAction,
    limiters: LimiterCollection,
    interactive: bool,
    verbose: bool,
    close_stdin: bool,
    replace_pat: Option<String>,
    eof: Option<Vec<u8>>,
    no_run_if_empty: bool,
    open_tty: bool,
}

impl CommandBuilderOptions {
    fn new(
        action: ExecAction,
        mut limiters: LimiterCollection,
        replace_pat: Option<String>,
        eof: Option<Vec<u8>>,
        no_run_if_empty: bool,
        open_tty: bool,
    ) -> Result<Self, ExhaustedCommandSpace> {
        let mut initial_args = match &action {
            ExecAction::Command(args) => args.iter().map(|arg| arg.as_ref()).collect(),
            ExecAction::Echo => vec![OsStr::new("echo")],
        };
        if let Some(ref pat) = replace_pat {
            if pat != "{}" {
                let pat_osstr = OsStr::new(pat);
                initial_args = initial_args
                    .into_iter()
                    .filter(|&arg| arg != pat_osstr)
                    .collect::<Vec<&OsStr>>();
            }
        }

        for arg in initial_args {
            limiters.try_arg(Argument {
                arg: arg.to_owned(),
                kind: ArgumentKind::Initial,
            })?;
        }

        Ok(Self {
            action,
            limiters,
            verbose: false,
            close_stdin: false,
            replace_pat,
            eof,
            interactive: false,
            no_run_if_empty,
            open_tty,
        })
    }
}

struct CommandBuilder<'options> {
    options: &'options CommandBuilderOptions,
    extra_args: Vec<OsString>,
    limiters: LimiterCollection,
}

impl CommandBuilder<'_> {
    fn new(options: &CommandBuilderOptions) -> CommandBuilder<'_> {
        CommandBuilder {
            options,
            extra_args: vec![],
            limiters: options.limiters.clone(),
        }
    }

    fn add_arg(&mut self, arg: Argument) -> Result<Argument, ExhaustedCommandSpace> {
        let arg = self.limiters.try_arg(arg)?;
        if arg.arg != "" {
            self.extra_args.push(arg.arg.clone());
        }

        Ok(Argument {
            arg: arg.arg,
            kind: arg.kind,
        })
    }

    fn drop_arg(&mut self) {
        if !self.extra_args.is_empty() {
            self.extra_args.pop();
        }
    }

    fn execute_command(self, mut command: Command) -> Result<CommandResult, CommandExecutionError> {
        match &self.options.action {
            ExecAction::Command(_) => match command.status() {
                Ok(status) => {
                    if status.success() {
                        Ok(CommandResult::Success)
                    } else if let Some(err) = status.code() {
                        if err == 255 {
                            Err(CommandExecutionError::UrgentlyFailed)
                        } else {
                            Ok(CommandResult::Failure)
                        }
                    } else {
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            if let Some(signal) = status.signal() {
                                Err(CommandExecutionError::Killed { signal })
                            } else {
                                Err(CommandExecutionError::Unknown)
                            }
                        }
                        #[cfg(not(unix))]
                        Err(CommandExecutionError::Unknown)
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    Err(CommandExecutionError::NotFound)
                }
                Err(e) => Err(CommandExecutionError::CannotRun(e)),
            },
            ExecAction::Echo => {
                println!(
                    "{}",
                    self.extra_args
                        .iter()
                        .map(|arg| arg.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                Ok(CommandResult::Success)
            }
        }
    }

    fn execute(self) -> Result<CommandResult, CommandExecutionError> {
        let (entry_point, initial_args): (&OsStr, &[OsString]) = match &self.options.action {
            ExecAction::Command(args) => (&args[0], &args[1..]),
            ExecAction::Echo => (OsStr::new("echo"), &[]),
        };
        if self.options.no_run_if_empty && self.extra_args.is_empty() {
            return Ok(CommandResult::Success);
        }

        let mut command = Command::new(entry_point);

        if let Some(replace_str) = &self.options.replace_pat {
            let replacement = self
                .extra_args
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            let initial_args: Vec<OsString> = initial_args
                .iter()
                .map(|arg| {
                    let arg_str = arg.to_string_lossy();
                    OsString::from(arg_str.replace(replace_str, &replacement))
                })
                .collect();
            if replacement.len() > 0 {
                command.args(&initial_args);
            } else {
                return Ok(CommandResult::Success);
            }
        } else {
            command.args(initial_args).args(&self.extra_args);
        };

        if self.options.close_stdin {
            command.stdin(Stdio::null());
        }

        if self.options.open_tty {
            let tty =
                fs::File::open("/dev/tty").map_err(|e| format!("Failed to open /dev/tty: {}", e));
            command.stdin(tty.unwrap());
        }

        if self.options.verbose {
            let args: Vec<&OsStr> = command.get_args().collect();
            let mut args_str = String::new();
            let program = command.get_program().to_str().unwrap();
            args_str.push_str(program);
            if args.len() > 0 {
                args_str.push_str(" ");
            }
            for arg in args {
                if arg == "" {
                    break;
                }
                args_str.push_quoted(Bash, arg);
                args_str.push(' ');
            }
            if args_str.ends_with(' ') {
                args_str.pop();
            }
            eprint!("{args_str}");

            if self.options.interactive {
                io::stderr().flush().ok();
                let response = prompt_user_and_get_response()
                    .map_err(|e| CommandExecutionError::CannotRun(e))?;

                if !response {
                    eprint!("\n");
                    io::stderr().flush().ok();
                    return Ok(CommandResult::Success);
                }
            } else {
                eprint!("\n");
            }
        }

        self.execute_command(command)
    }
}

trait ArgumentReader {
    fn next(&mut self) -> io::Result<Option<Argument>>;
}

struct WhitespaceDelimitedArgumentReader<R: Read> {
    rd: R,
    pending: Vec<u8>,
}

impl<R> WhitespaceDelimitedArgumentReader<R>
where
    R: Read,
{
    fn new(rd: R) -> Self {
        Self {
            rd,
            pending: vec![],
        }
    }
}

impl<R> ArgumentReader for WhitespaceDelimitedArgumentReader<R>
where
    R: Read,
{
    fn next(&mut self) -> io::Result<Option<Argument>> {
        let mut result = vec![];
        let mut terminated_by_newline = false;

        let mut pending = vec![];
        std::mem::swap(&mut pending, &mut self.pending);

        enum Escape {
            Slash,
            Quote(u8),
        }

        let mut escape: Option<Escape> = None;
        let mut i = 0;
        loop {
            if i == pending.len() {
                pending.resize(4096, 0);
                let bytes_read = loop {
                    match self.rd.read(&mut pending[..]) {
                        Ok(bytes_read) => break bytes_read,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                    }
                };

                if bytes_read == 0 {
                    if let Some(Escape::Quote(q)) = &escape {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("unmatched double quote: {q}"),
                        ));
                    } else if i == 0 {
                        return Ok(None);
                    } else {
                        pending.clear();
                        break;
                    }
                }

                pending.resize(bytes_read, 0);
                i = 0;
            }

            match (&escape, pending[i]) {
                (None, b'\'')
                    if i + 2 < pending.len()
                        && pending[i + 1] == b'\''
                        && pending[i + 2].is_ascii_whitespace() =>
                {
                    i += 2;
                    break;
                }
                (Some(Escape::Quote(quote)), c) if c == *quote => escape = None,
                (Some(Escape::Quote(_)), c) => result.push(c),
                (Some(Escape::Slash), c) => {
                    result.push(c);
                    escape = None;
                }
                (None, c @ (b'"' | b'\'')) => escape = Some(Escape::Quote(c)),
                (None, b'\\') => escape = Some(Escape::Slash),
                (None, c) if c == 0x0C || c == 0x0B => {
                    if !result.is_empty() {
                        result.push(c)
                    }
                }
                (None, c) if c.is_ascii_whitespace() => {
                    if !result.is_empty() {
                        terminated_by_newline = c == b'\n';
                        break;
                    }
                }
                (None, c) => result.push(c),
            }

            i += 1;
        }

        if i < pending.len() {
            self.pending = pending.split_off(i + 1);
        }

        Ok(Some(Argument {
            arg: String::from_utf8_lossy(&result[..]).into_owned().into(),
            kind: if terminated_by_newline {
                ArgumentKind::HardTerminated
            } else {
                ArgumentKind::SoftTerminated
            },
        }))
    }
}

struct ByteDelimitedArgumentReader<R: Read> {
    rd: BufReader<R>,
    delimiter: u8,
}

impl<R> ByteDelimitedArgumentReader<R>
where
    R: Read,
{
    fn new(rd: R, delimiter: u8) -> Self {
        Self {
            rd: BufReader::new(rd),
            delimiter,
        }
    }
}

impl<R> ArgumentReader for ByteDelimitedArgumentReader<R>
where
    R: Read,
{
    fn next(&mut self) -> io::Result<Option<Argument>> {
        Ok(loop {
            let mut buf = vec![];
            let bytes_read = self.rd.read_until(self.delimiter, &mut buf)?;
            if bytes_read > 0 {
                let need_to_trim_delimiter = buf[buf.len() - 1] == self.delimiter;
                let bytes = if need_to_trim_delimiter {
                    if buf.len() == 1 {
                        continue;
                    }

                    &buf[..buf.len() - 1]
                } else {
                    &buf[..]
                };
                break Some(Argument {
                    arg: String::from_utf8_lossy(bytes).into_owned().into(),
                    kind: ArgumentKind::HardTerminated,
                });
            } else {
                break None;
            }
        })
    }
}

///
#[derive(Debug)]
pub enum XargsError {
    ///
    ArgumentTooLarge,
    ///
    CommandExecution(CommandExecutionError),
    ///
    Io(io::Error),
    ///
    Untyped(String),
}

impl Display for XargsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XargsError::ArgumentTooLarge => write!(f, "Argument too large"),
            XargsError::CommandExecution(e) => write!(f, "{e}"),
            XargsError::Io(e) => write!(f, "{e}"),
            XargsError::Untyped(s) => write!(f, "{s}"),
        }
    }
}

impl Error for XargsError {}

impl From<String> for XargsError {
    fn from(s: String) -> Self {
        Self::Untyped(s)
    }
}

impl From<&'_ str> for XargsError {
    fn from(s: &'_ str) -> Self {
        s.to_owned().into()
    }
}

impl From<CommandExecutionError> for XargsError {
    fn from(e: CommandExecutionError) -> Self {
        Self::CommandExecution(e)
    }
}

impl From<io::Error> for XargsError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

fn process_input(
    builder_options: CommandBuilderOptions,
    mut args: Box<dyn ArgumentReader>,
    options: &Options,
) -> Result<CommandResult, XargsError> {
    let mut current_builder = CommandBuilder::new(&builder_options);
    let mut have_pending_command = false;
    let mut result = CommandResult::Success;

    while let Some(arg) = args.next()? {
        match current_builder.add_arg(arg) {
            Ok(arg) => {
                if let Some(_) = builder_options.replace_pat {
                    if arg.kind == ArgumentKind::HardTerminated {
                        if let Some(ref eof_str) = builder_options.eof {
                            let arg_str = &arg.arg.into_vec();
                            if arg_str == eof_str {
                                current_builder.drop_arg();
                                result.combine(current_builder.execute()?);
                                return Ok(result);
                            } else {
                                result.combine(current_builder.execute()?);
                                current_builder = CommandBuilder::new(&builder_options);
                            }
                        } else {
                            result.combine(current_builder.execute()?);
                            current_builder = CommandBuilder::new(&builder_options);
                        }
                    }
                } else if let Some(ref eof_str) = builder_options.eof {
                    let arg_str = &arg.arg.into_vec();
                    if arg_str == eof_str {
                        current_builder.drop_arg();
                        result.combine(current_builder.execute()?);
                        return Ok(CommandResult::Success);
                    }
                }
            }
            Err(ExhaustedCommandSpace { arg, out_of_chars }) => {
                if out_of_chars
                    && options.exit_if_pass_char_limit
                    && (options.max_args.is_some() || options.max_lines.is_some())
                {
                    return Err(XargsError::ArgumentTooLarge);
                } else if have_pending_command {
                    result.combine(current_builder.execute()?);
                }
                current_builder = CommandBuilder::new(&builder_options);
                if let Err(ExhaustedCommandSpace { .. }) = current_builder.add_arg(arg) {
                    return Err(XargsError::ArgumentTooLarge);
                }
            }
        }
        have_pending_command = true;
    }

    if !options.no_run_if_empty || have_pending_command {
        result.combine(current_builder.execute()?);
    }

    Ok(result)
}

fn prompt_user_and_get_response() -> io::Result<bool> {
    eprint!("?... ");
    io::stderr().flush()?;

    let tty = match std::fs::File::open("/dev/tty") {
        Ok(t) => io::BufReader::new(t),
        Err(e) => return Err(e),
    };

    match tty.lines().next() {
        Some(Ok(line)) => Ok(line.trim().eq_ignore_ascii_case("y")),
        Some(Err(e)) => Err(e),
        None => Ok(false),
    }
}

fn validate_positive_usize(s: String) -> Result<(), String> {
    match s.parse::<usize>() {
        Ok(v) if v > 0 => Ok(()),
        Ok(v) => Err(format!("Value must be > 0, not: {v}")),
        Err(e) => Err(e.to_string()),
    }
}

fn parse_delimiter(s: &str) -> Result<u8, String> {
    if let Some(hex) = s.strip_prefix("\\x") {
        u8::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex sequence: {}", e))
    } else if let Some(oct) = s.strip_prefix("\\0") {
        u8::from_str_radix(oct, 8).map_err(|e| format!("Invalid octal sequence: {}", e))
    } else if let Some(special) = s.strip_prefix('\\') {
        match special {
            "a" => Ok(b'\x07'),
            "b" => Ok(b'\x08'),
            "f" => Ok(b'\x0C'),
            "n" => Ok(b'\n'),
            "r" => Ok(b'\r'),
            "t" => Ok(b'\t'),
            "v" => Ok(b'\x0B'),
            "0" => Ok(b'\0'),
            "\\" => Ok(b'\\'),
            _ => Err(format!("Invalid escape sequence: {s}")),
        }
    } else {
        let bytes = s.as_bytes();
        if bytes.len() == 1 {
            Ok(bytes[0])
        } else {
            Err("Delimiter must be one byte".to_owned())
        }
    }
}

/// Parse the cmd args
pub fn parse_xargs_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
) -> UResult<(Options, clap::ArgMatches)> {
    let arg_list = args.collect_lossy();
    let mut command = xargs_app(about, usage);
    let mut usage_doc = Vec::new();
    command.write_help(&mut usage_doc).unwrap();

    let matches = command.get_matches_from(
        &arg_list
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<&str>>(),
    );
    let options = Options::from(&matches)?;
    Ok((options, matches))
}

/// Create command.
pub fn xargs_app<'a>(about: &'a str, usage: &'a str) -> clap::Command<'a> {
    clap::Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .settings(&[clap::AppSettings::TrailingVarArg])
        .arg(
            Arg::new(options::COMMAND)
                .takes_value(true)
                .multiple(true)
                .help("The command to run")
                .allow_invalid_utf8(true),
        )
        .arg(
            Arg::new(options::ARG_FILE)
                .short('a')
                .long(options::ARG_FILE)
                .help("Read arguments from the given file instead of stdin"),
        )
        .arg(
            Arg::new(options::DELIMITER)
                .short('d')
                .long(options::DELIMITER)
                .takes_value(true)
                .validator(|s| parse_delimiter(&s).map(|_| ()))
                .help("Use the given delimiter to split the input"),
        )
        .arg(
            Arg::new(options::EOF)
                .short('E')
                .long(options::EOF)
                .takes_value(true)
                .visible_short_alias('e')
                .max_values(1)
                .min_values(0)
                .help(
                    "set logical EOF string; if END occursas a line of input, \
                    the rest of the input isignored (ignored if -0 or -d was specified)",
                ),
        )
        .arg(
            Arg::new(options::REPLACE)
                .short('I')
                .long(options::REPLACE)
                .takes_value(true)
                .visible_short_alias('i')
                .max_values(1)
                .min_values(0)
                .help(
                    "Replace R in INITIAL-ARGS with names read from standard input; \
                    if R is unspecified, assume {}",
                ),
        )
        .arg(Arg::new(options::EXIT).short('x').long(options::EXIT).help(
            "Exit if the number of arguments allowed by -L or -n do not \
                    fit into the number of allowed characters",
        ))
        .arg(
            Arg::new(options::MAX_ARGS)
                .short('n')
                .takes_value(true)
                .long(options::MAX_ARGS)
                .validator(|s| validate_positive_usize(s.to_string()))
                .help(
                    "Set the max number of arguments read from stdin to be passed \
                    to each command invocation (mutually exclusive with -L)",
                ),
        )
        .arg(
            Arg::new(options::OPEN_TTY)
                .short('o')
                .takes_value(false)
                .long(options::OPEN_TTY)
                .help(
                    "Reopen stdin as /dev/tty in the child process \
                    before executing the command; useful to run an \
                    interactive application.",
                ),
        )
        .arg(
            Arg::new(options::MAX_LINES)
                .short('L')
                .takes_value(true)
                .long(options::MAX_LINES)
                .visible_short_alias('l')
                .min_values(0)
                .max_values(1)
                .validator(|s| validate_positive_usize(s.to_string()))
                .help(
                    "Set the max number of lines from stdin to be passed to each \
                    command invocation (mutually exclusive with -n)",
                ),
        )
        .arg(
            Arg::new(options::MAX_PROCS)
                .short('P')
                .takes_value(true)
                .long(options::MAX_PROCS)
                .validator(|s| validate_positive_usize(s.to_string()))
                .help("Run up to this many commands in parallel [NOT IMPLEMENTED]"),
        )
        .arg(
            Arg::new(options::INTERACTIVE)
                .short('p')
                .long(options::INTERACTIVE)
                .help("Prompt before running commands"),
        )
        .arg(
            Arg::new(options::NO_RUN_IF_EMPTY)
                .short('r')
                .long(options::NO_RUN_IF_EMPTY)
                .help("If there are no input arguments, do not run the command at all"),
        )
        .arg(
            Arg::new(options::NULL)
                .short('0')
                .long(options::NULL)
                .help("Split the input by null terminators rather than whitespace"),
        )
        .arg(
            Arg::new(options::SIZE)
                .short('s')
                .long(options::SIZE)
                .takes_value(true)
                .validator(|s| validate_positive_usize(s.to_string()))
                .help(
                    "Set the max number of characters to be passed to each \
                    invocation",
                ),
        )
        .arg(
            Arg::new(options::VERBOSE)
                .short('t')
                .long(options::VERBOSE)
                .help("Be verbose")
                .action(clap::ArgAction::SetTrue),
        )
}

/// Handle the input.
pub fn handle_input(
    mut options: Options,
    matches: ArgMatches,
) -> Result<CommandResult, XargsError> {
    let delimiter = match (options.delimiter, options.null) {
        (Some(delimiter), true) => {
            if matches.indices_of(options::NULL).unwrap().last()
                > matches.indices_of(options::DELIMITER).unwrap().last()
            {
                Some(b'\0')
            } else {
                Some(delimiter)
            }
        }
        (Some(delimiter), false) => Some(delimiter),
        (None, true) => Some(b'\0'),
        (None, false) => None,
    };

    let action = match matches.values_of_os(options::COMMAND) {
        Some(args) if args.len() > 0 => {
            ExecAction::Command(args.map(|arg| arg.to_owned()).collect())
        }
        _ => ExecAction::Echo,
    };

    let env = std::env::vars_os().collect();

    let mut limiters = LimiterCollection::new();

    if options.max_args.is_some() && options.replace_pat.is_some() {
        options.max_args = None;
        eprintln!("xargs: warning: options --max-args and --replace/-I/-i are mutually exclusive, ignoring previous --max-args value");
    }

    if options.eof.is_some() && (options.delimiter.is_some() || options.null) {
        options.eof = None;
        eprintln!("xargs: warning: the -E option has no effect if -0 or -d is used.\n");
    }

    match (
        matches.index_of(options::EOF),
        matches.index_of(options::REPLACE),
    ) {
        (Some(exit_index), Some(replace_index)) => {
            if exit_index > replace_index {
                options.eof = None;
            }
        }
        _ => {}
    }

    match (options.max_args, options.max_lines) {
        (Some(max_args), Some(max_lines)) => {
            eprintln!(
                    "xargs: warning: options --max-lines and --max-args/-n are mutually exclusive, ignoring previous --max-lines value");
            if matches.indices_of(options::MAX_LINES).unwrap().last()
                > matches.indices_of(options::MAX_ARGS).unwrap().last()
            {
                limiters.add(MaxLinesCommandSizeLimiter::new(max_lines));
            } else {
                limiters.add(MaxArgsCommandSizeLimiter::new(max_args));
            }
        }
        (Some(max_args), None) => limiters.add(MaxArgsCommandSizeLimiter::new(max_args)),
        (None, Some(max_lines)) => limiters.add(MaxLinesCommandSizeLimiter::new(max_lines)),
        (None, None) => (),
    }

    if let Some(max_chars) = options.size {
        limiters.add(MaxCharsCommandSizeLimiter::new(max_chars));
    }

    limiters.add(MaxCharsCommandSizeLimiter::new_system(&env));

    let mut builder_options = CommandBuilderOptions::new(
        action,
        limiters,
        options.replace_pat.clone(),
        options.eof.clone(),
        options.no_run_if_empty,
        options.open_tty,
    )
    .map_err(|_| "Base command and environment are too large to fit into one command execution")?;

    builder_options.verbose = options.verbose;

    if options.interactive {
        builder_options.verbose = true;
        builder_options.interactive = options.interactive;
    }
    builder_options.close_stdin = options.arg_file.is_none();

    let args_file: Box<dyn Read> = if let Some(path) = &options.arg_file {
        Box::new(fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?)
    } else {
        Box::new(io::stdin())
    };

    let args: Box<dyn ArgumentReader> = if let Some(delimiter) = delimiter {
        Box::new(ByteDelimitedArgumentReader::new(args_file, delimiter))
    } else {
        Box::new(WhitespaceDelimitedArgumentReader::new(args_file))
    };

    let result = process_input(builder_options, args, &options)?;
    Ok(result)
}

/// The main of the xargs
pub fn xargs_main(options: Options, matches: ArgMatches) -> UResult<i32> {
    match handle_input(options, matches) {
        Ok(CommandResult::Success) => Ok(0),
        Ok(CommandResult::Failure) => Ok(123),
        Err(e) => {
            eprintln!("Error: {e}");
            if let XargsError::CommandExecution(cx) = e {
                match cx {
                    CommandExecutionError::UrgentlyFailed => {
                        Err(USimpleError::new(124, "Urgently failed"))
                    }
                    CommandExecutionError::Killed { .. } => Err(USimpleError::new(125, "Killed")),
                    CommandExecutionError::CannotRun(_) => {
                        Err(USimpleError::new(126, "Cannot run"))
                    }
                    CommandExecutionError::NotFound => Err(USimpleError::new(127, "Not found")),
                    CommandExecutionError::Unknown => Err(USimpleError::new(1, "Unknown error")),
                }
            } else {
                Err(USimpleError::new(1, "Unknown error"))
            }
        }
    }
}
