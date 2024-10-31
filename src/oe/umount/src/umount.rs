//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::umount_common::{parse_umount_cmd_args, umount_app, Config, UmountHandler};
use clap::Command;
use uucore::error::{UResult, USimpleError};
use uucore::{help_section, help_usage};
mod umount_common;
const ABOUT: &str = help_section!("about", "umount.md");
const USAGE: &str = help_usage!("umount.md");

#[uucore::main]
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: Config = parse_umount_cmd_args(args, ABOUT, USAGE)?;
    let umount_handler = UmountHandler::new(config);
    match umount_handler.process() {
        Ok(_) => {}
        Err(e) => {
            // eprintln!("Error during mount operation: {}", e);
            return Err(USimpleError::new(
                1,
                format!("Umount operation failed: {}", e),
            ));
        }
    }
    Ok(())
}
/// Creates and returns the command-line interface for the umount utility.
///
/// This function sets up the CLI with all available options and subcommands
/// for the umount operation. It uses the `clap` crate to define the interface.
///
/// # Returns
///
/// Returns a `Command` struct that represents the CLI for the umount utility.

///
pub fn oe_app<'a>() -> Command<'a> {
    umount_app(ABOUT, USAGE)
}
