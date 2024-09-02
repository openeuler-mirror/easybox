//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod sysctl_common;

const ABOUT: &str = help_section!("about", "sysctl.md");
const USAGE: &str = help_usage!("sysctl.md");

#[uucore::main]
/// This the main of sysctl
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: sysctl_common::Config = sysctl_common::parse_sysctl_cmd_args(args, ABOUT, USAGE)?;
    sysctl_common::handle_input(config)
}

/// This the oe_app of sysctl
///
pub fn oe_app<'a>() -> Command<'a> {
    sysctl_common::sysctl_app(ABOUT, USAGE)
}
