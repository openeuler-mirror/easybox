//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::sha256sum_algorithm::sha256_reader;
use std::fmt::Debug;
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::{BufReader, Read, Stdin};
use std::path::Path;
use std::vec;
use uucore::display::Quotable;
use uucore::error::USimpleError;
use uucore::error::{FromIo, UResult};
use uucore::format_usage;

use clap::{crate_version, Arg, Command};

const SHA256SUM_CMD_PARSE_ERROR: i32 = 1;

/// Config.
#[derive(Debug)]
pub struct Config {
    ///
    pub binary: bool,
    ///
    pub check: bool,
    ///
    pub tag: bool,
    ///
    pub text: bool,
    ///
    pub zero: bool,
    ///
    pub ignore_missing: bool,
    ///
    pub quiet: bool,
    ///
    pub status: bool,
    ///
    pub strict: bool,
    ///
    pub warn: bool,
    ///
    pub to_read: Vec<String>,
}

/// options.
pub mod options {
    ///
    pub static BINARY: &str = "binary";
    ///
    pub static CHECK: &str = "check";
    ///
    pub static TAG: &str = "tag";
    ///
    pub static TEXT: &str = "text";
    ///
    pub static ZERO: &str = "zero";
    ///
    pub static IGNORE_MISSING: &str = "ignore-missing";
    ///
    pub static QUIET: &str = "quiet";
    ///
    pub static STATUS: &str = "status";
    ///
    pub static STRICT: &str = "strict";
    ///
    pub static WARN: &str = "warn";
    ///
    pub static FILE: &str = "file";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let file_args: Vec<String> = options
            .get_many::<String>(options::FILE)
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect();
        let mut file: Vec<String> = vec![];

        for filename in &file_args {
            if filename == "-" {
                file.push(filename.to_string());
                continue;
            } else {
                file.push(filename.to_string());
            }
        }

        if file_args.is_empty() {
            file.push(String::from("-"));
        }

        if !options.contains_id(options::CHECK) {
            fn invalid_without_check(options: &clap::ArgMatches, option_id: &str) -> UResult<()> {
                if options.contains_id(option_id) {
                    return Err(USimpleError::new(
                        SHA256SUM_CMD_PARSE_ERROR,
                        format!(
                            "the --{} option is meaningful only when verifying checksums\n\
                            Try 'sha256sum --help' for more information.",
                            option_id
                        ),
                    ));
                }
                Ok(())
            }
            for option_id in &[
                options::IGNORE_MISSING,
                options::QUIET,
                options::STATUS,
                options::STRICT,
                options::WARN,
            ] {
                invalid_without_check(options, *option_id)?;
            }
        } else {
            fn invalid_with_check(options: &clap::ArgMatches, option_id: &str) -> UResult<()> {
                if options.contains_id(option_id) {
                    return Err(USimpleError::new(
                        SHA256SUM_CMD_PARSE_ERROR,
                        format!(
                            "the --{} option is meaningless when verifying checksums\n\
                            Try 'sha256sum --help' for more information.",
                            option_id
                        ),
                    ));
                }
                Ok(())
            }
            for option_id in &[options::BINARY, options::TAG, options::TEXT, options::ZERO] {
                invalid_with_check(options, *option_id)?;
            }
        }
        Ok(Self {
            binary: options.contains_id(options::BINARY),
            check: options.contains_id(options::CHECK),
            tag: options.contains_id(options::TAG),
            text: options.contains_id(options::TEXT),
            zero: options.contains_id(options::ZERO),
            ignore_missing: options.contains_id(options::IGNORE_MISSING),
            quiet: options.contains_id(options::QUIET),
            status: options.contains_id(options::STATUS),
            strict: options.contains_id(options::STRICT),
            warn: options.contains_id(options::WARN),
            to_read: file,
        })
    }
}

///
pub fn parse_sha256sum_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
) -> UResult<Config> {
    let command = sha256sum_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn sha256sum_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        // Format arguments.
        .arg(
            Arg::new(options::BINARY)
                .short('b')
                .long(options::BINARY)
                .help("read in binary mode"),
        )
        .arg(
            Arg::new(options::CHECK)
                .short('c')
                .long(options::CHECK)
                .help("read SHA256 sums from the FILEs and check them"),
        )
        .arg(
            Arg::new(options::TAG)
                .long(options::TAG)
                .help("create a BSD-style checksum")
                .conflicts_with(options::CHECK),
        )
        .arg(
            Arg::new(options::TEXT)
                .short('t')
                .long(options::TEXT)
                .help("read in text mode (default)"),
        )
        .arg(Arg::new(options::ZERO).short('z').long(options::ZERO).help(
            "end each output line with NUL, not newline,\
                \nand disable file name escaping",
        ))
        .arg(
            Arg::new(options::IGNORE_MISSING)
                .long(options::IGNORE_MISSING)
                .help("don't fail or report status for missing files"),
        )
        .arg(
            Arg::new(options::QUIET)
                .long(options::QUIET)
                .help("don't print OK for each successfully verified file"),
        )
        .arg(
            Arg::new(options::STATUS)
                .long(options::STATUS)
                .help("don't output anything, status code shows success"),
        )
        .arg(
            Arg::new(options::STRICT)
                .long(options::STRICT)
                .help("exit non-zero for improperly formatted checksum lines"),
        )
        .arg(
            Arg::new(options::WARN)
                .short('w')
                .long(options::WARN)
                .help("arn about improperly formatted checksum lines"),
        )
        // "multiple" arguments are used to check whether there is more than one
        // file passed in.
        .arg(
            Arg::new(options::FILE)
                .index(1)
                .multiple_occurrences(true)
                .value_hint(clap::ValueHint::FilePath),
        )
}

/// Convert files to BufReader
///
pub fn file_to_read(file_name: &str) -> UResult<Box<dyn Read>> {
    let file_buf: File =
        File::open(Path::new(file_name)).map_err_context(|| file_name.maybe_quote().to_string())?;
    Ok(Box::new(BufReader::new(file_buf))) // as Box<dyn Read>
}

///
pub fn get_input<'a>(stdin_ref: &'a Stdin) -> UResult<Box<dyn Read + 'a>> {
    Ok(Box::new(BufReader::new(stdin_ref.lock())))
}
/// Unescapestring
///
pub fn unescape_string(s: &str) -> String {
    s.replace("\\", "\\\\")
}

///
pub fn handle_input(config: &Config) -> UResult<()> {
    let mut ret_code = 0;
    if config.check {
        for file_name in &config.to_read {
            if file_name == "-" {
                let stdin_raw = stdin();
                let mut input: Box<dyn Read> = get_input(&stdin_raw)?;
                let retcode = check_hash_reader(&mut input, config)?;
                if retcode != 0 {
                    ret_code = retcode
                }
            } else if !Path::new(file_name).exists() {
                eprintln!("{}: No such file or directory", file_name);
                ret_code = 1;
                continue;
            } else {
                let mut file = file_to_read(file_name)?;
                let retcode = check_hash_reader(&mut file, config)?;
                if retcode != 0 {
                    ret_code = retcode
                }
            }
        }
        if ret_code != 0 {
            return Err(USimpleError::new(ret_code, ""));
        }
        return Ok(());
    }

    for file_name in &config.to_read {
        let hash;
        if file_name == "-" {
            let stdin_raw = stdin();
            let mut input: Box<dyn Read> = get_input(&stdin_raw)?;
            hash = calculate_hash(&mut input);
        } else if !Path::new(file_name).exists() {
            eprintln!("{}: No such file or directory", file_name);
            ret_code = 1;
            continue;
        } else {
            let mut file = file_to_read(file_name)?;
            hash = calculate_hash(&mut file);
        }
        if file_name.contains("\\") && !config.zero {
            print!("\\");
            print_hash(hash?, &unescape_string(file_name), config);
        } else {
            print_hash(hash?, &file_name, config);
        }
    }
    if ret_code != 0 {
        return Err(USimpleError::new(ret_code, ""));
    }
    Ok(())
}

/// Check if the hash value is correct
///
pub fn check_hash<R: Read>(input: &mut R, ref_hash: &str) -> UResult<bool> {
    let reader = BufReader::new(input);
    let hash = sha256_reader(reader);
    let hex_hash: String = hash
        .unwrap()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join("");
    if hex_hash != ref_hash {
        return Ok(false);
    } else {
        return Ok(true);
    }
}

/// Check if the hash value of each line is correct
///
pub fn check_hash_reader<R: Read>(input: &mut R, config: &Config) -> UResult<i32> {
    let mut ret_code = 0;
    for line_result in BufReader::new(input).lines() {
        match line_result {
            Ok(line) => match parse_check_line(&line, config) {
                Ok((hash, filename, retcode)) => {
                    if retcode != 0 {
                        ret_code = retcode;
                    }
                    if !Path::new(&filename).exists() {
                        if config.ignore_missing {
                            continue;
                        }
                        eprintln!("{}: No such file or directory", filename);
                        ret_code = 1;
                        continue;
                    }
                    let mut file = file_to_read(&filename)?;
                    if check_hash(&mut file, &hash)? {
                        if !config.quiet && !config.status {
                            print!("{}: OK\n", filename);
                        }
                    } else if config.status {
                        ret_code = 1;
                    }
                }
                Err(_) => {
                    ret_code = 1;
                }
            },
            Err(_) => {
                ret_code = 1;
            }
        }
    }
    Ok(ret_code)
}

/// Calculate hash value
///
pub fn calculate_hash<R: Read>(input: &mut R) -> UResult<String> {
    let reader = BufReader::new(input);
    let hash = sha256_reader(reader);
    let hex_hash: String = hash
        .unwrap()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join("");
    Ok(hex_hash)
}

/// Output according to config
///
pub fn print_hash(hash: String, end: &str, config: &Config) {
    if config.tag {
        print!("SHA256 ({end}) = {hash}");
    } else {
        print!("{} ", hash);
        if config.binary {
            print!("*")
        } else {
            print!(" ")
        }
        print!("{}", end);
    }
    if !config.zero {
        println!();
    } else {
        print!("\0");
    }
}

/// Check if the hash format is correct
///
pub fn check_hash_format(hash: &str) -> UResult<bool> {
    if hash.chars().count() != 32 {
        return Ok(false);
    }
    // Check if all characters are alphanumeric
    if hash.chars().all(|c| c.is_alphanumeric()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parsing check files
///
pub fn parse_check_line(input: &str, config: &Config) -> UResult<(String, String, i32)> {
    let mut ret_code = 0;
    let mut split = input.splitn(3, " ");
    let hash = split.next().ok_or(USimpleError::new(1, ""))?;
    let mut filename = split.next().ok_or(USimpleError::new(1, ""))?;

    if filename.len() > 0 {
        if config.warn {
            eprintln!("WARNING: {}: line is improperly formatted", filename);
        }
        if config.strict {
            ret_code = 1;
        }
    } else {
        filename = split.next().ok_or(USimpleError::new(1, ""))?;
    }

    if config.warn && !check_hash_format(hash)? {
        eprintln!(
            "WARNING: {}: no properly formatted SHA256 checksum lines found",
            filename
        );
        if config.strict {
            ret_code = 1;
        }
    }
    Ok((hash.to_string(), filename.to_string(), ret_code))
}
