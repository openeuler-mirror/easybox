//! This file is part of the easybox package.
//
// (c) Yuyichen2025 <vyu112@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use crate::less_common::{configs, less, less_app, reset_term, setup_term, Configs};
use clap::Command;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::enable_raw_mode;
use crossterm::{execute, terminal};
use std::fs::OpenOptions;
use std::{
    fs,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    string,
};
use uucore::{
    display::Quotable,
    error::{UResult, USimpleError, UUsageError},
    show,
};

/// public mod less_common;
pub mod less_common;

struct TermGuard;

/// Drop trait for TermGuard
impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
    }
}

#[uucore::main]
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let matches = match less_app().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(e) => {
            return Err(UUsageError::new(1, e.to_string()));
        }
    };

    let mut configs = Configs::from(&matches);

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnableMouseCapture)?;

    let _guard = TermGuard;

    let output_file = matches.get_one::<String>(configs::OUTPUT).cloned();

    let mut buff = string::String::new();

    if let Some(files) = matches.get_many::<string::String>(configs::FILES) {
        let mut stdout = setup_term();
        let mut file_nums = files.len();

        let mut files_iter = files.map(|s| s.as_str()).peekable();
        while let (Some(file), next_file) = (files_iter.next(), files_iter.peek()) {
            let file = Path::new(file);
            if !file.exists() {
                terminal::disable_raw_mode().unwrap();
                show!(USimpleError::new(
                    0,
                    format!("cannot open {}: No such file or directory", file.quote()),
                ));
                terminal::enable_raw_mode().unwrap();
                continue;
            }
            if file.is_dir() {
                if configs.force_open {
                    configs.test_config.force_open = true;
                    buff.push_str(&format!("force read : '{}'\n", file.display()));
                    if let Ok(entries) = fs::read_dir(file) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let entry_path = entry.path();
                                let metadata = match entry.metadata() {
                                    Ok(m) => m,
                                    Err(e) => {
                                        buff.push_str(&format!(
                                            "Error getting metadata for {}: {}\n",
                                            entry_path.display(),
                                            e
                                        ));
                                        continue;
                                    }
                                };
                                let entry_type = if metadata.is_dir() { "dir" } else { "file" };
                                buff.push_str(&format!(
                                    "{:10} {}\n",
                                    entry_type,
                                    entry_path.display()
                                ));
                            }
                        }
                    } else {
                        terminal::disable_raw_mode().unwrap();
                        show!(USimpleError::new(
                            0,
                            format!("Failed to read {}", file.quote()),
                        ));
                        terminal::enable_raw_mode().unwrap();
                        continue;
                    }
                } else {
                    terminal::disable_raw_mode().unwrap();
                    show!(UUsageError::new(
                        0,
                        format!("{} is a directory.", file.quote()),
                    ));
                    terminal::enable_raw_mode().unwrap();
                    continue;
                }
            } else {
                let file_result = if configs.force_open {
                    configs.test_config.force_open = true;
                    OpenOptions::new()
                        .read(true)
                        .open(file)
                        .map_err(|e| (e, false))
                } else {
                    File::open(file).map_err(|e| (e, true))
                };

                match file_result {
                    Ok(opened_file) => {
                        let mut reader = BufReader::new(opened_file);

                        if let Some(buffer_size) = configs.buffer_size {
                            configs.test_config.buffer = true;
                            reader = BufReader::with_capacity(buffer_size, reader.into_inner());
                        }

                        match reader.read_to_string(&mut buff) {
                            Ok(_) => {}
                            Err(why) => {
                                if !configs.force_open {
                                    terminal::disable_raw_mode().unwrap();
                                    show!(USimpleError::new(
                                        0,
                                        format!(
                                            "Failed to read file {}: {}",
                                            file.quote(),
                                            why.kind()
                                        ),
                                    ));
                                    terminal::enable_raw_mode().unwrap();
                                }
                            }
                        }
                    }
                    Err((why, show_error)) => {
                        if show_error {
                            terminal::disable_raw_mode().unwrap();
                            show!(USimpleError::new(
                                0,
                                format!("cannot open {}: {}", file.quote(), why.kind()),
                            ));
                            terminal::enable_raw_mode().unwrap();
                        }
                        continue;
                    }
                }
            }

            let output = less(
                &buff,
                &mut stdout,
                file_nums > 1,
                file.to_str(),
                next_file.copied(),
                &mut configs,
            )?;

            if let Some(ref output_filename) = output_file {
                if let Err(err) = fs::write(&output_filename, output) {
                    terminal::disable_raw_mode().unwrap();
                    show!(USimpleError::new(
                        0,
                        format!(
                            "Failed to write output to file {}: {}",
                            output_filename, err
                        ),
                    ));
                    terminal::enable_raw_mode().unwrap();
                }
            }
            if file_nums > 1 {
                file_nums -= 1;
            } else if file_nums == 0 {
                // no more files to read
                break;
            }
            buff.clear();
        }
        reset_term(&mut stdout);
    } else {
        if buff.is_empty() {
            return Err(UUsageError::new(1, "Missing filename"));
        }
        let mut stdout = setup_term();
        let output = less(&buff, &mut stdout, false, None, None, &mut configs)?;

        if let Some(ref output_filename) = output_file {
            if let Err(err) = fs::write(&output_filename, output) {
                terminal::disable_raw_mode().unwrap();
                show!(USimpleError::new(
                    0,
                    format!(
                        "Failed to write output to file {}: {}",
                        output_filename, err
                    ),
                ));
                terminal::enable_raw_mode().unwrap();
            }
        }
        reset_term(&mut stdout);
    }

    Ok(())
}

/// configuration for less
pub fn oe_app<'a>() -> Command<'a> {
    less_common::less_app()
}
