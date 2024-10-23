//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

/// Utility modules used by the usermod command.
pub mod utils {
    /// Module for checking names.
    pub mod chkname;
    /// Module for chown.
    pub mod chowndir;
    /// Module containing constants used across the application.
    pub mod constants;
    /// Module for copying directories.
    pub mod copydir;
    /// Module for exit codes definitions.
    pub mod exitcodes;
    /// Module for retrieving user IDs.
    pub mod get_uid;
    /// Module for getting default configurations.
    pub mod getdef;
    /// Module for getting the time.
    pub mod gettime;
    /// Module for handling group shadow files.
    pub mod gshadow;
    /// Module for list manipulation utilities.
    pub mod list;
    /// Module for handling prefix.
    pub mod prefix_flag;
    /// Module for processing root flags.
    pub mod root_flag;
    /// Module for handling user shadow files.
    pub mod shadow;
    /// Module for converting strings to dates.
    pub mod strtoday;
    /// Module for sub files.
    pub mod subordinateio;
    /// Module for checking if a user is currently logged in.
    pub mod user_busy;
}
///
pub mod usermod_common;
///
const ABOUT: &str = help_section!("about", "usermod.md");
///
const USAGE: &str = help_usage!("usermod.md");

/// This the main of usermod
///
#[uucore::main]
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let mut config = usermod_common::parse_usermod_cmd_args(args, ABOUT, USAGE)?;
    usermod_common::usermod_main(&mut config)?;

    Ok(())
}

/// This the oe_app of free
///
pub fn oe_app<'a>() -> Command<'a> {
    usermod_common::usermod_app(ABOUT, USAGE)
}
