//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use std::path::PathBuf;
use uucore::{
    error::{set_exit_code, UResult},
    help_section, help_usage,
    msg_log::errmsg_custom,
};

/// which common functions
pub mod which_common;

use which_common::{
    func_search, get_current_working_directory, get_home_dir, parse_which_cmd_args,
};

use crate::which_common::{path_search, read_alias_functions};

const ABOUT: &str = help_section!("about", "which.md");
const USAGE: &str = help_usage!("which.md");

#[uucore::main]
/// This the main of which
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let mut config: which_common::Config = parse_which_cmd_args(args, ABOUT, USAGE)?;
    let mut fail_count = 0;
    if config.show_dot {
        config.cwd = get_current_working_directory()?;
    }
    if config.show_tilde || config.skip_tilde {
        let res = get_home_dir(&config.current_user);
        let mut need_push = false;
        if !res.ends_with("/") {
            need_push = true;
        }
        config.home = res.into_os_string();
        if need_push {
            config.home.push("/");
        }
    }
    if config.skip_alias {
        config.read_alias = false;
    }
    if config.skip_functions {
        config.read_functions = false;
    }
    if config.read_alias || config.read_functions {
        read_alias_functions(&mut config)?;
    }
    for argv in &config.command_list {
        let mut found_something = false;
        if argv.is_empty() {
            continue;
        }
        if config.read_functions && argv.find('/') == None {
            found_something = func_search(false, argv, &config);
        }
        if (config.show_all || !found_something)
            && !path_search(false, PathBuf::from(argv), &config)?
            && !found_something
        {
            if absolute_program(argv) {
                let mut abs_path = String::new();
                if !argv.starts_with('.') && !argv.starts_with('/') && !argv.starts_with('~') {
                    abs_path.push_str("./");
                }
                abs_path.push_str(argv);
                print_fail(
                    argv.rsplit_once('/').unwrap().1,
                    abs_path.rsplit_once('/').unwrap().0,
                );
            } else {
                print_fail(argv, &config.path_list.to_string_lossy());
            }
            fail_count += 1;
        }
    }
    set_exit_code(fail_count);
    Ok(())
}

/// This the oe_app of which
///
pub fn oe_app<'a>() -> Command<'a> {
    which_common::which_app(ABOUT, USAGE)
}

/// Judge name is an absolute program
fn absolute_program(name: &str) -> bool {
    return name.find('/').is_some();
}

/// Print formatted error message
fn print_fail(name: &str, path_list: &str) {
    errmsg_custom(false, 0, false, &format!("no {} in ({})", name, path_list));
}
