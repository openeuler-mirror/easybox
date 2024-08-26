//! This file is part of the uutils coreutils package.
//
// (c) Lin Guantao <moyihust@gmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod md5sum_common;

const ABOUT: &str = help_section!("about", "md5sum.md");
const USAGE: &str = help_usage!("md5sum.md");

#[uucore::main]
/// This the main of md5sum
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: md5sum_common::Config = md5sum_common::parse_md5sum_cmd_args(args, ABOUT, USAGE)?;

    md5sum_common::handle_input(&config)
}

/// This the oe_app of md5sum
///
pub fn oe_app<'a>() -> Command<'a> {
    md5sum_common::md5sum_app(ABOUT, USAGE)
}
