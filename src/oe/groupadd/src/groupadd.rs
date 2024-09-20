//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod utils {
    ///
    pub mod constants;
    ///
    pub mod exitcodes;
    ///
    pub mod getdef;

    ///
    pub mod prefix_flag;
    ///
    pub mod root_flag;
}

///
pub mod groupadd_common;

///
const ABOUT: &str = help_section!("about", "groupadd.md");
///
const USAGE: &str = help_usage!("groupadd.md");

/// This the main of groupadd
///
#[uucore::main]
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let mut config: groupadd_common::Config =
        groupadd_common::parse_groupadd_cmd_args(args, ABOUT, USAGE)?;
    groupadd_common::handle_input(&mut config)
}

/// This the oe_app of free
///
pub fn oe_app<'a>() -> Command<'a> {
    groupadd_common::groupadd_app(ABOUT, USAGE)
}
