//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use utils::defaults::UserAddDefaults;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod utils {
    ///
    pub mod chkname;
    ///
    pub mod constants;
    ///
    pub mod copydir;
    ///
    pub mod defaults;
    ///
    pub mod exitcodes;
    ///
    pub mod find_new_gid;
    ///
    pub mod find_new_sub_gids;
    ///
    pub mod find_new_sub_uids;
    ///
    pub mod find_new_uid;
    ///
    pub mod getdef;
    ///
    pub mod gshadow;
    ///
    pub mod passwd;
    ///
    pub mod prefix_flag;
    ///
    pub mod root_flag;
    ///
    pub mod shadow;
    ///
    pub mod strtoday;
    ///
    pub mod subordinateio;
}
///
pub mod useradd_common;

///
const ABOUT: &str = help_section!("about", "useradd.md");
///
const USAGE: &str = help_usage!("useradd.md");
///
const USER_DEFAULTS_FILE: &str = "/etc/default/useradd";

/// This the main of useradd
///
#[uucore::main]
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let mut defaults_data = UserAddDefaults::new_defaults();
    defaults_data.from_file(USER_DEFAULTS_FILE)?;

    let mut config: useradd_common::Config =
        useradd_common::parse_useradd_cmd_args(args, ABOUT, USAGE, &mut defaults_data)?;

    useradd_common::handle_input(&mut config, &mut defaults_data)
}

/// This the oe_app of free
///
pub fn oe_app<'a>() -> Command<'a> {
    useradd_common::useradd_app(ABOUT, USAGE)
}
