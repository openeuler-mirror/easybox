//! This file is part of the easybox package.
//
// (c) Zhihua Zhao <YuukaC@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

mod file_common;
pub mod file_magic;
mod file_unsafe;
mod file_utils;

const ABOUT: &str = help_section!("about", "file.md");
const USAGE: &str = help_usage!("file.md");

#[uucore::main]
/// This the main of base32
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: file_common::Config = file_common::parse_file_cmd_args(args, ABOUT, USAGE)?;

    file_common::handle_input(&config)
}

/// This the oe_app of base32
///
pub fn oe_app<'a>() -> Command<'a> {
    file_common::base_app(ABOUT, USAGE)
}
