//! This file is part of the easybox package.
//
// (c)  Allen Xu <xubo3006@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;

use uucore::error::{set_exit_code, UResult};
use uucore::{help_section, help_usage};

pub mod flock_common;

const ABOUT: &str = help_section!("about", "flock.md");
const USAGE: &str = help_usage!("flock.md");

#[uucore::main]
/// This is the main of flock
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: flock_common::Config = flock_common::parse_flock_cmd_args(args, ABOUT, USAGE)?;
    let handle_res = flock_common::handle_input(config)?;
    set_exit_code(handle_res);
    Ok(())
}

/// This the oe_app of flock
///
pub fn oe_app<'a>() -> Command<'a> {
    flock_common::flock_app(ABOUT, USAGE)
}
