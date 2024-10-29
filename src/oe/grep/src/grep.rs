//! This file is part of the easybox package.
//
// (c) SodaGreeny574 <1968629133@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use std::path::Path;
use uucore::error::UResult;
use uucore::error::USimpleError;
use uucore::error::UUsageError;
use uucore::help_section;
use uucore::help_usage;

pub mod grep_common;

const ABOUT: &str = help_section!("about", "grep.md");
const USAGE: &str = help_usage!("grep.md");

#[uucore::main]
/// This is the main of grep
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    run_grep(args)?;
    Ok(())
}

fn run_grep(args: impl uucore::Args) -> UResult<()> {
    let config = grep_common::parse_grep_cmd_args(args, ABOUT, USAGE)?;

    if let Some(threads) = config.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|e| USimpleError::new(2, format!("Failed to build thread pool: {}", e)))?;
    }

    let mut any_matched = false;

    if config.file.is_empty() {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let label = config.label.as_deref().unwrap_or("standard input");
        let matched = grep_common::handle_input(&mut handle, &config, Some(label))?;
        if matched {
            any_matched = true;
        }
    } else {
        for path_str in &config.file {
            let path = Path::new(path_str);

            if config.recursive {
                if path.is_dir() {
                    let matched = grep_common::handle_recursive_search(&config, path)?;
                    if matched {
                        any_matched = true;
                    }
                } else if path.is_file() {
                    let matched = grep_common::handle_file(path, &config)?;
                    if matched {
                        any_matched = true;
                    }
                } else {
                    eprintln!("{} is not a valid file or directory", path_str);
                    return Err(UUsageError::new(
                        2,
                        format!("{} is not a valid file or directory", path_str),
                    ));
                }
            } else {
                if path.is_file() {
                    let matched = grep_common::handle_file(path, &config)?;
                    if matched {
                        any_matched = true;
                    }
                } else if path.is_dir() {
                    eprintln!(
                        "{} is a directory (recursive search not specified)",
                        path_str
                    );
                    return Err(UUsageError::new(
                        2,
                        format!(
                            "{} is a directory (recursive search not specified)",
                            path_str
                        ),
                    ));
                } else {
                    eprintln!("{} is not a file", path_str);
                    return Err(UUsageError::new(2, format!("{} is not a file", path_str)));
                }
            }
        }
    }

    if any_matched {
        Ok(())
    } else if config.quiet {
        Ok(())
    } else {
        Err(UUsageError::new(1, "No match was found"))
    }
}

/// This is the oe_app of grep
///
pub fn oe_app<'a>() -> Command<'a> {
    grep_common::grep_app(ABOUT, USAGE)
}
