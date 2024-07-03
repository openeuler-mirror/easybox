//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::error::UResult;
use uucore::{help_section, help_usage};

///
pub mod column_common;
pub mod lib_column;

const ABOUT: &str = help_section!("about", "column.md");
const USAGE: &str = help_usage!("column.md");

#[uucore::main]
/// This the main of column
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config = column_common::parse_column_cmd_args(args, ABOUT, USAGE)?;
    column_common::column_main(config)?;
    Ok(())
}

/// This the oe_app of column
///
pub fn oe_app<'a>() -> Command<'a> {
    column_common::column_app(ABOUT, USAGE)
}
