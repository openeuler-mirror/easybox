//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod pgrep_common;
pub mod process;
pub mod signals;

const ABOUT: &str = help_section!("about", "pgrep.md");
const USAGE: &str = help_usage!("pgrep.md");

#[uucore::main]
/// This the main of pgrep
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: pgrep_common::Config = pgrep_common::parse_pgrep_cmd_args(args, ABOUT, USAGE)?;
    pgrep_common::handle_input(config)
}

/// This the oe_app of pgrep
///
pub fn oe_app<'a>() -> Command<'a> {
    pgrep_common::pgrep_app(ABOUT, USAGE)
}
