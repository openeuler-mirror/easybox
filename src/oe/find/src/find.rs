//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{
    error::{set_exit_code, UResult},
    help_section, help_usage,
};

use crate::find_common::{parse_find_cmd_args, search};

/// find common functions
pub mod find_common;

const ABOUT: &str = help_section!("about", "find.md");
const USAGE: &str = help_usage!("find.md");

#[uucore::main]
/// This the main of find
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let (mut config, mut filters) = parse_find_cmd_args(args, ABOUT, USAGE)?;
    search(&mut config, filters.as_mut())?;

    set_exit_code(config.status);

    Ok(())
}

/// This the oe_app of find
///
pub fn oe_app<'a>() -> Command<'a> {
    find_common::find_app(ABOUT, USAGE)
}
