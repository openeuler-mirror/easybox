//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod usleep_common;

const ABOUT: &str = help_section!("about", "usleep.md");
const USAGE: &str = help_usage!("usleep.md");

#[uucore::main]
/// This the main of usleep
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: usleep_common::Config = usleep_common::parse_usleep_cmd_args(args, ABOUT, USAGE)?;
    usleep_common::handle_input(config)
}

/// This the oe_app of usleep
///
pub fn oe_app<'a>() -> Command<'a> {
    usleep_common::usleep_app(ABOUT, USAGE)
}
