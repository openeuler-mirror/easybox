//! This file is part of the uutils coreutils package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::error::{set_exit_code, UResult};
use uucore::{help_section, help_usage};

pub mod chage_common;

const ABOUT: &str = help_section!("about", "chage.md");
const USAGE: &str = help_usage!("chage.md");

#[uucore::main]
/// This is the main of chage
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: chage_common::Config = chage_common::parse_chage_cmd_args(args, ABOUT, USAGE)?;
    let handle_res = chage_common::handle_input(config)?;
    set_exit_code(handle_res);
    Ok(())
}

/// This is the oe_app of chage
///
pub fn oe_app<'a>() -> Command<'a> {
    chage_common::chage_app(ABOUT, USAGE)
}
