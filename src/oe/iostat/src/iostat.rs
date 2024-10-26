//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

pub mod iostat_common;
mod iostat_utils;

const ABOUT: &str = help_section!("about", "iostat.md");
const USAGE: &str = help_usage!("iostat.md");

#[uucore::main]
/// This is the main of iostat
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: iostat_common::Config = iostat_common::parse_base_cmd_args(args, ABOUT, USAGE)?;
    iostat_common::handle_input(&config)?;
    Ok(())
}

/// This the oe_app of iostat
///
pub fn oe_app<'a>() -> Command<'a> {
    iostat_common::iostat_app(ABOUT, USAGE)
}
