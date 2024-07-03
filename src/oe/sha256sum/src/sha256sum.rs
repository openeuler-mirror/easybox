//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;

use uucore::{error::UResult, help_section, help_usage};
///
pub mod sha256sum_algorithm;
///
pub mod sha256sum_common;

const ABOUT: &str = help_section!("about", "sha256sum.md");
const USAGE: &str = help_usage!("sha256sum.md");

#[uucore::main]
/// This the main of sha256sum
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: sha256sum_common::Config =
        sha256sum_common::parse_sha256sum_cmd_args(args, ABOUT, USAGE)?;

    sha256sum_common::handle_input(&config)
}

/// This the oe_app of sha256sum
///
pub fn oe_app<'a>() -> Command<'a> {
    sha256sum_common::sha256sum_app(ABOUT, USAGE)
}
