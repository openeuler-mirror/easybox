//! This file is part of the uutils coreutils package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::error::{set_exit_code, UResult};
use uucore::{help_section, help_usage};

pub mod pidof_common;

const ABOUT: &str = help_section!("about", "pidof.md");
const USAGE: &str = help_usage!("pidof.md");

#[uucore::main]
/// This is the main of pidof
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: pidof_common::Config = pidof_common::parse_pidof_cmd_args(args, ABOUT, USAGE)?;
    let handle_res = pidof_common::handle_input(config)?;
    set_exit_code(handle_res);
    Ok(())
}

/// This is the oe_app of pidof
///
pub fn oe_app<'a>() -> Command<'a> {
    pidof_common::pidof_app(ABOUT, USAGE)
}
