//! This file is part of the easybox package.
//
// (c) Allen Xu <xubo3006@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::error::{set_exit_code, UResult};
use uucore::{help_section, help_usage};

///
pub mod pstree_common;

const ABOUT: &str = help_section!("about", "pstree.md");
const AFTERHELP: &str = help_section!("after help", "pstree.md");
const USAGE: &str = help_usage!("pstree.md");

#[uucore::main]
/// This the main of pstree
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: pstree_common::Config =
        pstree_common::parse_pstree_cmd_args(args, ABOUT, USAGE, AFTERHELP)?;
    let handle_res = pstree_common::handle_input(config)?;
    set_exit_code(handle_res);
    Ok(())
}

/// This the oe_app of pstree
///
pub fn oe_app<'a>() -> Command<'a> {
    pstree_common::pstree_app(ABOUT, USAGE, AFTERHELP)
}
