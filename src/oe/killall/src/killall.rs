//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

pub mod killall_common;

const ABOUT: &str = help_section!("about", "killall.md");
const USAGE: &str = help_usage!("killall.md");

#[uucore::main]
/// This is the main of killall
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: killall_common::Config = killall_common::parse_base_cmd_args(args, ABOUT, USAGE)?;
    killall_common::handle_input(&config)?;
    Ok(())
}

/// This the oe_app of killall
///
pub fn oe_app<'a>() -> Command<'a> {
    killall_common::killall_app(ABOUT, USAGE)
}
