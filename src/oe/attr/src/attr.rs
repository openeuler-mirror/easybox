//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use attr_common::{Config, Operation};
use clap::Command;
use uucore::{
    error::{UResult, UUsageError},
    help_section, help_usage,
};

/// attr common functions
pub mod attr_common;
pub mod lib_attr;

const ABOUT: &str = help_section!("about", "attr.md");
const USAGE: &str = help_usage!("attr.md");

#[uucore::main]
/// This the main of attr
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: Config = attr_common::parse_attr_cmd_args(args, ABOUT, USAGE)?;
    match config.opflag {
        Operation::SETOP => attr_common::handle_setop(&config),
        Operation::GETOP => attr_common::handle_getop(&config),
        Operation::REMOVEOP => attr_common::handle_removeop(&config),
        Operation::LISTOP => attr_common::handle_listop(&config),
        Operation::NONEOP => Err(UUsageError::new(
            libc::EXIT_FAILURE,
            "At least one of -s, -g, -r, or -l is required",
        )),
    }
}

/// This the oe_app of attr
///
pub fn oe_app<'a>() -> Command<'a> {
    attr_common::attr_app(ABOUT, USAGE)
}
