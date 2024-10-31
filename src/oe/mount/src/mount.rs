//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::mount_common::{mount_app, parse_mount_cmd_args, Config, ConfigHandler};
use clap::Command;
use uucore::error::{UResult, USimpleError};
use uucore::{help_section, help_usage};
///
pub mod mount_common;

const ABOUT: &str = help_section!("about", "mount.md");
const USAGE: &str = help_usage!("mount.md");

#[uucore::main]
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: Config = parse_mount_cmd_args(args, ABOUT, USAGE)?;
    let config_handler = ConfigHandler::new(config);
    match config_handler.process() {
        Ok(_) => {}
        Err(e) => {
            // eprintln!("Error during mount operation: {}", e);
            return Err(USimpleError::new(
                1,
                format!("Mount operation failed: {}", e),
            ));
        }
    }

    Ok(())
}
/// Creates and returns the command-line interface for the mount utility.
///
/// This function sets up the CLI with all available options and subcommands
/// for the mount operation. It uses the `clap` crate to define the interface.
///
/// # Returns
///
/// Returns a `Command` struct that represents the CLI for the mount utility.

///
pub fn oe_app<'a>() -> Command<'a> {
    mount_app(ABOUT, USAGE)
}
