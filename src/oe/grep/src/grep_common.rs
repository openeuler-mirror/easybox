//! This file is part of the easybox package.
//
// (c) SodaGreeny574 <1968629133@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use aho_corasick::AhoCorasick;
use atty;
use clap::crate_version;
use clap::{Arg, ArgAction, Command};
use core::any::Any;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use fancy_regex::Regex as FancyRegex;
use glob::Pattern;
use rayon::prelude::*;
use regex::Regex;
use regex::RegexBuilder;
use std::collections::VecDeque;
use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use uucore::error::{UResult, USimpleError, UUsageError};
use uucore::format_usage;
use walkdir::WalkDir;

#[derive(Clone)]

enum RegexWrapper {
    Standard(Regex),
    Fancy(fancy_regex::Regex),
}
#[derive(Debug, Clone)]
/// Config.
pub struct Config {
    ///
    pub file: Vec<String>,
    ///
    pub pattern: Vec<String>,
    ///
    pub extended_regexp: bool,
    ///
    pub fixed_strings: bool,
    ///
    pub basic_regexp: bool,
    ///
    pub perl_regexp: bool,
    ///
    pub ignore_case: bool,
    ///
    pub word_regexp: bool,
    ///
    pub line_regexp: bool,
    ///
    pub null_data: bool,
    ///
    pub no_messages: bool,
    ///
    pub invert_match: bool,
    ///
    pub version: bool,
    ///
    pub help: bool,
    ///
    pub max_count: Option<usize>,
    ///
    pub byte_offset: bool,
    ///
    pub line_number: bool,
    ///
    pub line_buffered: bool,
    ///
    pub with_filename: bool,
    ///
    pub no_filename: bool,
    ///
    pub label: Option<String>,
    ///
    pub only_matching: bool,
    ///
    pub quiet: bool,
    ///
    pub binary_files: String,
    ///
    pub text: bool,
    ///
    pub directories: Option<String>,
    ///
    pub devices: Option<String>,
    ///
    pub recursive: bool,
    ///
    pub dereference_recursive: bool,
    ///
    pub include: Vec<String>,
    ///
    pub exclude: Vec<String>,
    ///
    pub exclude_from: Option<String>,
    ///
    pub exclude_dir: Vec<String>,
    ///
    pub files_without_match: bool,
    ///
    pub files_with_matches: bool,
    ///
    pub count: bool,
    ///
    pub initial_tab: bool,
    ///
    pub null: bool,
    ///
    pub before_context: Option<usize>,
    ///
    pub after_context: Option<usize>,
    ///
    pub context: Option<usize>,
    ///
    pub color: Option<String>,
    ///
    pub break_output: bool,
    ///
    pub group_separator: Option<String>,
    ///
    pub word_size: Option<usize>,
    ///
    pub threads: Option<usize>,
    ///
    pub binary: bool,
    ///
    pub file_pattern: Option<String>,
    ///
    pub no_ignore_case: bool,
    ///
    pub binary_without_match: bool,
    ///
    pub combined_pattern: String,
    ///
    pub encoding: Option<String>,
}

/// Options.
///
pub mod options {
    ///
    pub static EXTENDED_REGEXP: &str = "extended-regexp";
    ///
    pub static FIXED_STRINGS: &str = "fixed-strings";
    ///
    pub static BASIC_REGEXP: &str = "basic-regexp";
    ///
    pub static PERL_REGEXP: &str = "perl-regexp";
    ///
    pub static PATTERN: &str = "pattern";
    ///
    pub static FILE: &str = "file";
    ///
    pub static IGNORE_CASE: &str = "ignore-case";
    ///
    pub static WORD_REGEXP: &str = "word-regexp";
    ///
    pub static LINE_REGEXP: &str = "line-regexp";
    ///
    pub static NULL_DATA: &str = "null-data";
    ///
    pub static NO_MESSAGES: &str = "no-messages";
    ///
    pub static INVERT_MATCH: &str = "invert-match";
    ///
    pub static VERSION: &str = "version";
    ///
    pub static HELP: &str = "help";
    ///
    pub static MAX_COUNT: &str = "max-count";
    ///
    pub static BYTE_OFFSET: &str = "byte-offset";
    ///
    pub static LINE_NUMBER: &str = "line-number";
    ///
    pub static LINE_BUFFERED: &str = "line-buffered";
    ///
    pub static WITH_FILENAME: &str = "with-filename";
    ///
    pub static NO_FILENAME: &str = "no-filename";
    ///
    pub static LABEL: &str = "label";
    ///
    pub static ONLY_MATCHING: &str = "only-matching";
    ///
    pub static QUIET: &str = "quiet";
    ///
    pub static BINARY_FILES: &str = "binary-files";
    ///
    pub static TEXT: &str = "text";
    ///
    pub static DIRECTORIES: &str = "directories";
    ///
    pub static DEVICES: &str = "devices";
    ///
    pub static RECURSIVE: &str = "recursive";
    ///
    pub static DEREFERENCE_RECURSIVE: &str = "dereference-recursive";
    ///
    pub static INCLUDE: &str = "include";
    ///
    pub static EXCLUDE: &str = "exclude";
    ///
    pub static EXCLUDE_FROM: &str = "exclude-from";
    ///
    pub static EXCLUDE_DIR: &str = "exclude-dir";
    ///
    pub static FILES_WITHOUT_MATCH: &str = "files-without-match";
    ///
    pub static FILES_WITH_MATCHES: &str = "files-with-matches";
    ///
    pub static COUNT: &str = "count";
    ///
    pub static INITIAL_TAB: &str = "initial-tab";
    ///
    pub static NULL: &str = "null";
    ///
    pub static BEFORE_CONTEXT: &str = "before-context";
    ///
    pub static AFTER_CONTEXT: &str = "after-context";
    ///
    pub static CONTEXT: &str = "context";
    ///
    pub static COLOR: &str = "color";
    ///
    pub static BREAK_OUTPUT: &str = "break-output";
    ///
    pub static GROUP_SEPARATOR: &str = "group-separator";
    ///
    pub static WORD_SIZE: &str = "word-size";
    ///
    pub static THREADS: &str = "threads";
    ///
    pub static REGEXP: &str = "regexp";
    ///
    pub static FILE_PATTERN: &str = "file-pattern";
    ///
    pub static NO_IGNORE_CASE: &str = "no-ignore-case";
    ///
    pub static BINARY_WITHOUT_MATCH: &str = "binary-without-match";
    ///
    pub static BINARY: &str = "binary";
    ///
    pub static ENCODING: &str = "encoding";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let file: Vec<String> = options
            .get_many::<String>(options::FILE)
            .map(|vals| vals.map(String::from).collect())
            .unwrap_or_else(Vec::new);

        let mut pattern: Vec<String> = options
            .get_many::<String>(options::REGEXP)
            .map(|vals| vals.map(String::from).collect())
            .unwrap_or_else(Vec::new);

        if let Some(pattern_files) = options.get_many::<String>(options::FILE_PATTERN) {
            for file_name in pattern_files {
                let file = File::open(file_name).map_err(|_| {
                    UUsageError::new(1, format!("Unable to open mode file '{}'", file_name))
                })?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line.map_err(|_| {
                        UUsageError::new(1, format!("Error reading mode file '{}'", file_name))
                    })?;
                    pattern.push(line);
                }
            }
        }
        if let Some(positional_patterns) = options.get_many::<String>("pattern") {
            for p in positional_patterns {
                pattern.push(p.clone());
            }
        }

        if pattern.is_empty() {
            if let Some(positional_pattern) = options.get_one::<String>(options::PATTERN) {
                pattern.push(positional_pattern.to_string());
            }
        }

        let combined_pattern = pattern.join("|");
        let mut exclude = options
            .values_of(options::EXCLUDE)
            .map(|vals| vals.map(String::from).collect())
            .unwrap_or_else(Vec::new);

        if let Some(exclude_from_file) = options.value_of(options::EXCLUDE_FROM) {
            let file = File::open(exclude_from_file).map_err(|_| {
                UUsageError::new(
                    1,
                    format!("Unable to open exclusion file '{}'", exclude_from_file),
                )
            })?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(|_| {
                    UUsageError::new(
                        1,
                        format!("Error reading exclusion file '{}'", exclude_from_file),
                    )
                })?;
                exclude.push(line);
            }
        }

        let max_count = options
            .value_of(options::MAX_COUNT)
            .map(|val| {
                val.parse::<usize>()
                    .map_err(|_| UUsageError::new(1, format!("Invalid --max-count value: {}", val)))
            })
            .transpose()?;

        let mut before_context = options
            .value_of(options::BEFORE_CONTEXT)
            .map(|val| {
                val.parse::<usize>().map_err(|_| {
                    UUsageError::new(1, format!("Invalid --before-context value: {}", val))
                })
            })
            .transpose()?;

        let mut after_context = options
            .value_of(options::AFTER_CONTEXT)
            .map(|val| {
                val.parse::<usize>().map_err(|_| {
                    UUsageError::new(1, format!("Invalid --after-context value: {}", val))
                })
            })
            .transpose()?;
        let context = options
            .value_of(options::CONTEXT)
            .map(|val| {
                val.parse::<usize>()
                    .map_err(|_| UUsageError::new(1, format!("Invalid --context value: {}", val)))
            })
            .transpose()?;

        if let Some(context_val) = options.value_of(options::CONTEXT) {
            let context_val = context_val.parse::<usize>().map_err(|_| {
                UUsageError::new(1, format!("Invalid --context value: {}", context_val))
            })?;
            if before_context.is_none() {
                before_context = Some(context_val);
            }
            if after_context.is_none() {
                after_context = Some(context_val);
            }
        }

        let threads = options
            .value_of(options::THREADS)
            .map(|val| {
                val.parse::<usize>()
                    .map_err(|_| UUsageError::new(1, format!("Invalid --threads value: {}", val)))
            })
            .transpose()?;

        let word_size = options
            .value_of(options::WORD_SIZE)
            .map(|val| {
                val.parse::<usize>()
                    .map_err(|_| UUsageError::new(1, format!("Invalid --word-size value: {}", val)))
            })
            .transpose()?;
        let color = options.value_of(options::COLOR).map(String::from);

        let binary_without_match = options.is_present(options::BINARY_WITHOUT_MATCH);

        let binary_files = if binary_without_match {
            "without-match".to_string()
        } else {
            options
                .get_one::<String>(options::BINARY_FILES)
                .unwrap_or(&"without-match".to_string())
                .clone()
        };
        Ok(Self {
            file,
            pattern,
            combined_pattern: combined_pattern.clone(),
            extended_regexp: options.is_present(options::EXTENDED_REGEXP),
            fixed_strings: options.is_present(options::FIXED_STRINGS),
            basic_regexp: options.is_present(options::BASIC_REGEXP),
            perl_regexp: options.is_present(options::PERL_REGEXP),
            ignore_case: options.is_present(options::IGNORE_CASE),
            word_regexp: options.is_present(options::WORD_REGEXP),
            line_regexp: options.is_present(options::LINE_REGEXP),
            null_data: options.is_present(options::NULL_DATA),
            no_messages: options.is_present(options::NO_MESSAGES),
            invert_match: options.is_present(options::INVERT_MATCH),
            version: options.is_present(options::VERSION),
            help: options.is_present(options::HELP),
            max_count,
            byte_offset: options.is_present(options::BYTE_OFFSET),
            line_number: options.is_present(options::LINE_NUMBER),
            line_buffered: options.is_present(options::LINE_BUFFERED),
            with_filename: options.is_present(options::WITH_FILENAME),
            no_filename: options.is_present(options::NO_FILENAME),
            label: options.value_of(options::LABEL).map(String::from),
            only_matching: options.is_present(options::ONLY_MATCHING),
            quiet: options.is_present(options::QUIET),
            binary_files,
            text: options.is_present(options::TEXT),
            directories: options.value_of(options::DIRECTORIES).map(String::from),
            devices: options.value_of(options::DEVICES).map(String::from),
            recursive: options.is_present(options::RECURSIVE),
            dereference_recursive: options.is_present(options::DEREFERENCE_RECURSIVE),
            include: options
                .values_of(options::INCLUDE)
                .map(|vals| vals.map(String::from).collect())
                .unwrap_or_else(Vec::new),
            exclude,
            exclude_from: options.value_of(options::EXCLUDE_FROM).map(String::from),
            exclude_dir: options
                .values_of(options::EXCLUDE_DIR)
                .map(|vals| vals.map(String::from).collect())
                .unwrap_or_else(Vec::new),
            files_without_match: options.is_present(options::FILES_WITHOUT_MATCH),
            files_with_matches: options.is_present(options::FILES_WITH_MATCHES),
            count: options.is_present(options::COUNT),
            initial_tab: options.is_present(options::INITIAL_TAB),
            null: options.is_present(options::NULL),
            before_context,
            after_context,
            context,
            color,
            break_output: options.is_present(options::BREAK_OUTPUT),
            group_separator: options.value_of(options::GROUP_SEPARATOR).map(String::from),
            word_size,
            threads,
            binary: options.is_present(options::BINARY),
            no_ignore_case: options.is_present(options::NO_IGNORE_CASE),
            file_pattern: None,
            binary_without_match,
            encoding: options.value_of(options::ENCODING).map(String::from),
        })
    }
}

///
pub fn grep_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::PATTERN)
                .index(1)
                .conflicts_with(options::FILE_PATTERN)
                .value_name("PATTERN")
                .help("Patterns to search for"),
        )
        .arg(
            Arg::new(options::FILE)
                .index(2)
                .multiple_values(true)
                .takes_value(true)
                .value_hint(clap::ValueHint::FilePath)
                .help("Files to search in"),
        )
        .arg(
            Arg::new(options::EXTENDED_REGEXP)
                .short('E')
                .long(options::EXTENDED_REGEXP)
                .help("Use extended regular expressions"),
        )
        .arg(
            Arg::new(options::FIXED_STRINGS)
                .short('F')
                .long(options::FIXED_STRINGS)
                .help("PATTERNS are strings"),
        )
        .arg(
            Arg::new(options::BASIC_REGEXP)
                .short('G')
                .long(options::BASIC_REGEXP)
                .help("Use basic regular expressions (default)"),
        )
        .arg(
            Arg::new(options::PERL_REGEXP)
                .short('P')
                .long(options::PERL_REGEXP)
                .help("Use Perl regular expressions"),
        )
        .arg(
            Arg::new(options::REGEXP)
                .short('e')
                .long(options::REGEXP)
                .action(ArgAction::Append)
                .value_name("PATTERN")
                .help("Use PATTERNS for matching"),
        )

        .arg(
            Arg::new(options::FILE_PATTERN)
                .short('f')
                .long(options::FILE_PATTERN)
                .action(ArgAction::Append)
                .value_name("FILE")
                .value_hint(clap::ValueHint::FilePath)
                .help("Take PATTERNS from FILE"),
        )
        .arg(
            Arg::new(options::IGNORE_CASE)
                .short('i')
                .long(options::IGNORE_CASE)
                .help("Ignore case distinctions in patterns and data"),
        )
        .arg(
            Arg::new(options::NO_IGNORE_CASE)
                .long(options::NO_IGNORE_CASE)
                .help("Do not ignore case distinctions (default)"),
        )
        .arg(
            Arg::new(options::WORD_REGEXP)
                .short('w')
                .long(options::WORD_REGEXP)
                .help("Match only whole words"),
        )
        .arg(
            Arg::new(options::LINE_REGEXP)
                .short('x')
                .long(options::LINE_REGEXP)
                .help("Match only whole lines"),
        )
        .arg(
            Arg::new(options::NULL_DATA)
                .short('z')
                .long(options::NULL_DATA)
                .help("A data line ends in 0 byte, not newline"),
        )
        // Miscellaneous
        .arg(
            Arg::new(options::NO_MESSAGES)
                .short('s')
                .long(options::NO_MESSAGES)
                .help("Suppress error messages"),
        )
        .arg(
            Arg::new(options::INVERT_MATCH)
                .short('v')
                .long(options::INVERT_MATCH)
                .help("Select non-matching lines"),
        )
        .arg(
            Arg::new(options::VERSION)
                .short('V')
                .long(options::VERSION)
                .help("Display version information and exit"),
        )
        .arg(
            Arg::new(options::HELP)
                .long(options::HELP)
                .help("Display this help text and exit"),
        )
        .arg(
            Arg::new(options::MAX_COUNT)
                .short('m')
                .long(options::MAX_COUNT)
                .takes_value(true)
                .help("Stop after NUM selected lines"),
        )
        .arg(
            Arg::new(options::BYTE_OFFSET)
                .short('b')
                .long(options::BYTE_OFFSET)
                .help("Print the byte offset with output lines"),
        )
        .arg(
            Arg::new(options::LINE_NUMBER)
                .short('n')
                .long(options::LINE_NUMBER)
                .help("Print line number with output lines"),
        )
        .arg(
            Arg::new(options::LINE_BUFFERED)
                .long(options::LINE_BUFFERED)
                .help("Flush output on every line"),
        )
        .arg(
            Arg::new(options::WITH_FILENAME)
                .short('H')
                .long(options::WITH_FILENAME)
                .help("Print file name with output lines"),
        )
        .arg(
            Arg::new(options::NO_FILENAME)
                .short('h')
                .long(options::NO_FILENAME)
                .help("Suppress the file name prefix on output"),
        )
        .arg(
            Arg::new(options::LABEL)
                .long(options::LABEL)
                .takes_value(true)
                .help("Use LABEL as the standard input file name prefix"),
        )
        .arg(
            Arg::new(options::ONLY_MATCHING)
                .short('o')
                .long(options::ONLY_MATCHING)
                .help("Show only nonempty parts of lines that match"),
        )

        .arg(
            Arg::new(options::QUIET)
                .short('q')
                .long(options::QUIET)
                .alias("silent")
                .help("Suppress all normal output"),
        )
        .arg(
            Arg::new(options::BINARY_FILES)
                .long(options::BINARY_FILES)
                .takes_value(true)
                .value_name("TYPE")
                .possible_values(&["binary", "text", "without-match"])
                .action(ArgAction::Set)
                .default_value("without-match")
                .help("Assume that binary files are TYPE; TYPE is 'binary', 'text', or 'without-match'"),
        )
        .arg(
            Arg::new(options::TEXT)
                .short('a')
                .long(options::TEXT)
                .help("Equivalent to --binary-files=text"),
        )
        .arg(
            Arg::new(options::BINARY_WITHOUT_MATCH)
                .short('I')
                .long(options::BINARY_WITHOUT_MATCH)
                .action(ArgAction::SetTrue)
                .conflicts_with(options::BINARY_FILES)
                .help("Equivalent to --binary-files=without-match"),
        )

        .arg(
            Arg::new(options::DIRECTORIES)
                .short('d')
                .long(options::DIRECTORIES)
                .takes_value(true)
                .help("How to handle directories; ACTION is 'read', 'recurse', or 'skip'"),
        )
        .arg(
            Arg::new(options::DEVICES)
                .short('D')
                .long(options::DEVICES)
                .takes_value(true)
                .help("How to handle devices, FIFOs and sockets; ACTION is 'read' or 'skip'"),
        )
        .arg(
            Arg::new(options::RECURSIVE)
                .short('r')
                .long(options::RECURSIVE)
                .help("Recursively search directories"),
        )
        .arg(
            Arg::new(options::DEREFERENCE_RECURSIVE)
                .short('R')
                .long(options::DEREFERENCE_RECURSIVE)
                .help("Likewise, but follow all symlinks"),
        )
        .arg(
            Arg::new(options::EXCLUDE)
                .long(options::EXCLUDE)
                .takes_value(true)
                .action(ArgAction::Append)
                .help("Skip files that match GLOB"),
        )
        .arg(
            Arg::new(options::INCLUDE)
                .long(options::INCLUDE)
                .takes_value(true)
                .action(ArgAction::Append)
                .help("Search only files that match GLOB"),
        )

        .arg(
            Arg::new(options::EXCLUDE_FROM)
                .long(options::EXCLUDE_FROM)
                .takes_value(true)
                .value_hint(clap::ValueHint::FilePath)
                .help("Skip files that match any file pattern from FILE"),
        )
        .arg(
            Arg::new(options::EXCLUDE_DIR)
                .long(options::EXCLUDE_DIR)
                .takes_value(true)
                .action(ArgAction::Append)
                .help("Skip directories that match GLOB"),
        )
        .arg(
            Arg::new(options::FILES_WITHOUT_MATCH)
                .short('L')
                .long(options::FILES_WITHOUT_MATCH)
                .help("Print only names of FILEs with no selected lines"),
        )
        .arg(
            Arg::new(options::FILES_WITH_MATCHES)
                .short('l')
                .long(options::FILES_WITH_MATCHES)
                .help("Print only names of FILEs with selected lines"),
        )
        .arg(
            Arg::new(options::COUNT)
                .short('c')
                .long(options::COUNT)
                .help("Print only a count of selected lines per FILE"),
        )
        .arg(
            Arg::new(options::INITIAL_TAB)
                .short('T')
                .long(options::INITIAL_TAB)
                .help("Make tabs line up (if needed)"),
        )
        .arg(
            Arg::new(options::NULL)
                .short('Z')
                .long(options::NULL)
                .help("Print 0 byte after FILE name"),
        )
        // Context control
        .arg(
            Arg::new(options::BEFORE_CONTEXT)
                .short('B')
                .long(options::BEFORE_CONTEXT)
                .takes_value(true)
                .help("Print NUM lines of leading context"),
        )
        .arg(
            Arg::new(options::AFTER_CONTEXT)
                .short('A')
                .long(options::AFTER_CONTEXT)
                .takes_value(true)
                .help("Print NUM lines of trailing context"),
        )
        .arg(
            Arg::new(options::CONTEXT)
                .short('C')
                .long(options::CONTEXT)
                .takes_value(true)
                .help("Print NUM lines of output context"),
        )
        .arg(
            Arg::new(options::COLOR)
                .long(options::COLOR)
                .alias("colour")
                .takes_value(true)
                .possible_values(&["always", "auto", "never"])
                .help("Surround the matched (non-empty) string with escape sequences to display them in color on the terminal; WHEN can be 'never', 'always', or 'auto'"),
        )
        .arg(
            Arg::new(options::BINARY)
                .short('U')
                .long(options::BINARY)
                .help("Do not strip CR characters at EOL (MSDOS/Windows)"),
        )
        .arg(
            Arg::new(options::THREADS)
                .short('j')
                .long(options::THREADS)
                .takes_value(true)
                .help("Number of threads to use"),
        )
        .arg(
            Arg::new(options::WORD_SIZE)
                .long(options::WORD_SIZE)
                .takes_value(true)
                .help("Specify the word size"),
        )
        .arg(
            Arg::new(options::BREAK_OUTPUT)
                .long(options::BREAK_OUTPUT)
                .takes_value(true)
                .help("Break output at word boundaries"),
        )
        .arg(
            Arg::new(options::GROUP_SEPARATOR)
                .long(options::GROUP_SEPARATOR)
                .takes_value(true)
                .help("Use GROUP as the group separator"),
        )
        .arg(
            Arg::new(options::ENCODING)
                .long(options::ENCODING)
                .takes_value(true)
                .help("Specify the encoding of the input files"),
        )
}

///
pub fn parse_grep_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = grep_app(about, usage);
    let arg_list = args.collect_lossy();
    let mut config = Config::from(&command.try_get_matches_from(arg_list.clone())?)?;

    if (config.file.len() > 1 || config.recursive) && !config.no_filename {
        config.with_filename = true;
    }
    if config.quiet {
        config.files_with_matches = false;
        config.count = false;
    }
    if config.files_with_matches {
        config.count = false;
    }
    if config.files_with_matches {
        config.files_without_match = false;
    }
    Ok(config)
}

///
pub fn handle_input<R: Read + BufRead + Any + 'static>(
    input: &mut R,
    config: &Config,
    file_name: Option<&str>,
) -> UResult<bool> {
    let mut matched_in_file = false;

    let is_binary = if let Some(file) = (input as &mut dyn Any).downcast_mut::<File>() {
        let mut sample = [0; 1024];
        let size = file.read(&mut sample)?;
        let is_binary = sample[..size].contains(&0);
        file.seek(SeekFrom::Start(0))?;
        is_binary
    } else {
        false
    };

    if is_binary {
        println!(
            "Processing binary file with binary_files: {}",
            config.binary_files
        );
        match config.binary_files.as_str() {
            "binary" => {
                if !config.quiet {
                    println!(
                        "Binary file {} matches",
                        file_name.unwrap_or("standard input")
                    );
                }
                return Ok(true);
            }
            "text" => {
                println!(
                    "Processing binary file {} as text",
                    file_name.unwrap_or("standard input")
                );
            }
            "without-match" => {
                println!(
                    "Skipping binary file {}",
                    file_name.unwrap_or("standard input")
                );
                return Ok(false);
            }
            _ => {
                println!(
                    "Unknown binary_files option: {}. Skipping file.",
                    config.binary_files
                );
                return Ok(false);
            }
        }
    }

    let mut match_count = 0;
    let mut line_number = 0;
    let mut byte_offset = 0;

    let buf_reader: Box<dyn BufRead> = if config.text {
        let encoding = config.encoding.as_deref().unwrap_or("utf-8");
        let encoding = Encoding::for_label(encoding.as_bytes())
            .ok_or_else(|| UUsageError::new(1, format!("Unsupported encoding: {}", encoding)))?;
        let decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding))
            .build(input);
        Box::new(BufReader::new(decoder))
    } else {
        Box::new(input)
    };
    let regex = if config.fixed_strings {
        None
    } else {
        let mut pattern = config.pattern.join("|");
        if config.word_regexp {
            pattern = format!(r"\b({})\b", pattern);
        } else if config.line_regexp {
            pattern = format!(r"^({})$", pattern);
        }

        if config.perl_regexp {
            let re = FancyRegex::new(&pattern)
                .map_err(|e| UUsageError::new(1, format!("Invalid regex pattern: {}", e)))?;
            Some(RegexWrapper::Fancy(re))
        } else {
            let re = RegexBuilder::new(&pattern)
                .case_insensitive(config.ignore_case)
                .build()
                .map_err(|e| UUsageError::new(1, format!("Invalid regex pattern: {}", e)))?;
            Some(RegexWrapper::Standard(re))
        }
    };

    let aho_matcher = if config.fixed_strings {
        let patterns: Vec<&str> = config.pattern.iter().map(|s| s.as_str()).collect();
        Some(AhoCorasick::new(&patterns))
    } else {
        None
    };

    let before_context = config.before_context.unwrap_or(0);
    let after_context = config.after_context.unwrap_or(0);

    let mut context_buffer: VecDeque<String> = VecDeque::with_capacity(before_context);
    let mut remaining_after = 0;
    let mut in_group = false;

    for line_result in buf_reader.lines() {
        let line = line_result?;

        line_number += 1;
        let line_length = line.len() + 1;
        byte_offset += line_length;

        let line_to_search = if config.ignore_case && !config.fixed_strings {
            line.to_lowercase()
        } else {
            line.clone()
        };

        let is_match = if let Some(matcher) = &aho_matcher {
            let matched = matcher.is_match(&line);
            if matched {
                remaining_after = after_context;
            }
            matched
        } else {
            match &regex {
                Some(RegexWrapper::Fancy(re)) => re
                    .is_match(&line_to_search)
                    .map_err(|e| UUsageError::new(1, format!("Regex match error: {}", e)))?,
                Some(RegexWrapper::Standard(re)) => re.is_match(&line_to_search),
                None => false,
            }
        };

        if is_match != config.invert_match {
            matched_in_file = true;

            if config.files_with_matches {
                if let Some(file_name) = file_name {
                    if config.null {
                        print!("{}\0", file_name);
                    } else {
                        println!("{}", file_name);
                    }
                } else {
                    println!("{}", config.label.as_deref().unwrap_or("standard input"));
                }
                return Ok(true);
            }

            if config.files_without_match {
                return Ok(false);
            }

            if config.break_output && !in_group {
                println!("{}", config.group_separator.as_deref().unwrap_or("--"));
            }
            in_group = true;

            match_count += 1;

            if let Some(max) = config.max_count {
                if match_count >= max {
                    break;
                }
            }

            if before_context > 0 {
                for (i, ctx_line) in context_buffer.iter().enumerate() {
                    if config.line_number {
                        println!("{}-{}", line_number - context_buffer.len() + i, ctx_line);
                    } else {
                        println!("{}", ctx_line);
                    }
                }
            }

            let mut output_line = String::new();

            if config.byte_offset {
                output_line.push_str(&format!("{}:", byte_offset - line_length));
            }

            if (config.file.len() > 1 || config.recursive)
                && config.with_filename
                && !config.no_filename
            {
                if let Some(file_name) = file_name {
                    if config.null {
                        output_line.push_str(&format!("{}{}", file_name, "\0"));
                    } else {
                        output_line.push_str(&format!("{}:", file_name));
                    }
                }
            }

            if config.line_number {
                output_line.push_str(&format!("{}:", line_number));
            }

            if config.initial_tab {
                output_line.push('\t');
            }

            let matched_line = if let Some(color) = &config.color {
                if color == "always" || (color == "auto" && atty::is(atty::Stream::Stdout)) {
                    match &regex {
                        Some(RegexWrapper::Fancy(re)) => re
                            .replace_all(&line, |caps: &fancy_regex::Captures| {
                                format!("\x1b[31m{}\x1b[0m", &caps[0])
                            })
                            .into_owned(),
                        Some(RegexWrapper::Standard(re)) => re
                            .replace_all(&line, |caps: &regex::Captures| {
                                format!("\x1b[31m{}\x1b[0m", &caps[0])
                            })
                            .into_owned(),
                        None => line.clone(),
                    }
                } else {
                    line.clone()
                }
            } else {
                line.clone()
            };

            if config.only_matching {
                match &regex {
                    Some(RegexWrapper::Fancy(re)) => {
                        for caps_result in re.captures_iter(&line_to_search) {
                            let caps = caps_result.map_err(|e| {
                                UUsageError::new(1, format!("Regex capture error: {}", e))
                            })?;
                            if let Some(m) = caps.get(0) {
                                let matched_text = &line[m.start()..m.end()];
                                let mut match_output = output_line.clone();
                                match_output.push_str(matched_text);
                                println!("{}", match_output);
                            }
                        }
                    }
                    Some(RegexWrapper::Standard(re)) => {
                        for mat in re.find_iter(&line_to_search) {
                            let matched_text = &line[mat.start()..mat.end()];
                            let mut match_output = output_line.clone();
                            match_output.push_str(matched_text);
                            println!("{}", match_output);
                        }
                    }
                    None => {}
                }
            } else if !config.files_with_matches
                && !config.files_without_match
                && !config.quiet
                && !config.count
            {
                output_line.push_str(&matched_line);
                println!("{}", output_line);
                if config.line_buffered {
                    std::io::stdout()
                        .flush()
                        .map_err(|e| UUsageError::new(1, e.to_string()))?;
                }
            }

            if let Some(max) = config.max_count {
                if match_count >= max {
                    break;
                }
            }

            if config.quiet {
                return Ok(true);
            }

            remaining_after = after_context;
            context_buffer.clear();
        } else {
            if remaining_after > 0 && !config.quiet && !config.count {
                println!("{}", line);
                remaining_after -= 1;
            }

            if before_context > 0 {
                if context_buffer.len() == before_context {
                    context_buffer.pop_front();
                }
                context_buffer.push_back(line);
            }
        }
    }

    if config.files_without_match && !matched_in_file {
        if let Some(file_name) = file_name {
            if config.null {
                print!("{}\0", file_name);
            } else {
                println!("{}", file_name);
            }
        } else {
            println!("{}", config.label.as_deref().unwrap_or("standard input"));
        }
    }

    if config.count {
        println!("{}", match_count);
    }

    Ok(matched_in_file)
}

///
pub fn handle_file(path: &Path, config: &Config) -> UResult<bool> {
    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            if !config.no_messages {
                eprintln!("{}: No such file or directory", path.display());
            }
            return Err(USimpleError::new(
                2,
                format!("{}: No such file or directory", path.display()),
            ));
        }
        Err(e) => {
            if !config.no_messages {
                eprintln!("{}: {}", path.display(), e);
            }
            return Err(USimpleError::new(1, e.to_string()));
        }
    };
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        match config.directories.as_deref() {
            Some("read") => {
                for entry in path.read_dir()? {
                    let entry = entry?;
                    handle_file(&entry.path(), config)?;
                }
                return Ok(false);
            }
            Some("recurse") => {
                handle_recursive_search(config, path)?;
                return Ok(false);
            }
            Some("skip") | _ => {
                return Ok(false);
            }
        }
    } else if file_type.is_file() {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let label = path.to_str();
        handle_input(&mut reader, config, label.map(|s| s)).map_err(|e| e.into())
    } else if file_type.is_fifo()
        || file_type.is_socket()
        || file_type.is_block_device()
        || file_type.is_char_device()
    {
        match config.devices.as_deref() {
            Some("read") => {
                let file = File::open(&path)?;
                let mut reader = BufReader::new(file);
                let label = path.to_str();
                handle_input(&mut reader, config, label.map(|s| s)).map_err(|e| e.into())
            }
            Some("skip") | _ => {
                return Ok(false);
            }
        }
    } else {
        return Ok(false);
    }
}

///
pub fn handle_recursive_search(config: &Config, start_path: &Path) -> UResult<bool> {
    let any_matched = AtomicBool::new(false);

    let exclude_patterns: Vec<Pattern> = config
        .exclude
        .iter()
        .map(|p| Pattern::new(p).unwrap())
        .collect();
    let exclude_dir_patterns: Vec<Pattern> = config
        .exclude_dir
        .iter()
        .map(|p| Pattern::new(p).unwrap())
        .collect();
    let errors = Mutex::new(Vec::new());

    let files: Vec<_> = WalkDir::new(start_path)
        .follow_links(config.dereference_recursive)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            if e.file_type().is_dir() {
                !exclude_dir_patterns.iter().any(|p| p.matches(&file_name))
            } else {
                !exclude_patterns.iter().any(|p| p.matches(&file_name))
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    files.par_iter().for_each(|entry| {
        let path = entry.path();
        if let Err(e) = handle_file(path, config) {
            let mut errors = errors.lock().unwrap();
            errors.push(format!("Error processing file {}: {}", path.display(), e));
        } else {
            any_matched.store(true, Ordering::Relaxed);
        }
    });

    let errors = errors.into_inner().unwrap();
    for error in &errors {
        eprintln!("{}", error);
    }

    if !errors.is_empty() {
        return Err(USimpleError::new(1, "An error occurred during processing"));
    }
    Ok(any_matched.load(Ordering::Relaxed))
}
