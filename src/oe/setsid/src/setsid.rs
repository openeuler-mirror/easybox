//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod setsid_common;

const ABOUT: &str = help_section!("about", "setsid.md");
const USAGE: &str = help_usage!("setsid.md");

#[uucore::main]
/// This the main of setsid
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: setsid_common::Config = setsid_common::parse_setsid_cmd_args(args, ABOUT, USAGE)?;
    setsid_common::handle_input(config)
}

/// This the oe_app of setsid
///
pub fn oe_app<'a>() -> Command<'a> {
    setsid_common::setsid_app(ABOUT, USAGE)
}
