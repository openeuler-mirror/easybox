//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::error::{set_exit_code, UResult};
use uucore::{help_section, help_usage};

///
pub mod xargs_common;

const ABOUT: &str = help_section!("about", "xargs.md");
const USAGE: &str = help_usage!("xargs.md");

#[uucore::main]
/// This the main of xargs
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let (config, matches) = xargs_common::parse_xargs_cmd_args(args, ABOUT, USAGE)?;
    let handle_res = xargs_common::xargs_main(config, matches)?;
    set_exit_code(handle_res);
    Ok(())
}

/// This the oe_app of xargs
///
pub fn oe_app<'a>() -> Command<'a> {
    xargs_common::xargs_app(ABOUT, USAGE)
}
