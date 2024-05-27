//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

pub mod lspci_common;

const ABOUT: &str = help_section!("about", "lspci.md");
const USAGE: &str = help_usage!("lspci.md");

#[uucore::main]
/// This is the main of lspci
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let config: lspci_common::Config = lspci_common::parse_base_cmd_args(args, ABOUT, USAGE)?;
    let bridges: Vec<lspci_common::Bridge>;
    if config.map_mode {
        lspci_common::map_the_bus(&config);
        return Ok(());
    }
    if config.path > 0 || config.tree {
        bridges = lspci_common::grow_tree(&config);
    } else {
        bridges = Vec::new();
    }
    if config.tree {
        lspci_common::show_forest(&bridges, &config)
    } else {
        lspci_common::show(&config);
    }
    Ok(())
}

/// This the oe_app of lspci
///
pub fn oe_app<'a>() -> Command<'a> {
    lspci_common::lspci_app(ABOUT, USAGE)
}
