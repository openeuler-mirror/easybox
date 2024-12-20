//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use uucore::error::{UResult, USimpleError};
use uucore::net_tools::{get_aftype, get_hw_list_str, get_hwtype};
use uucore::{help_section, help_usage};

pub mod arp_common;
pub mod arp_unsafe;

const ABOUT: &str = help_section!("about", "arp.md");
const USAGE: &str = help_usage!("arp.md");

use arp_common::{DFLT_AF, DFLT_HW};

#[uucore::main]
/// This the main of arp
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    if get_hwtype(DFLT_HW).is_none() {
        return Err(USimpleError::new(
            -1,
            format!("{}: hardware type not supported!", DFLT_HW),
        ));
    };
    if get_aftype(DFLT_AF).is_none() {
        return Err(USimpleError::new(
            -1,
            format!("{}: address family not supported!", DFLT_AF),
        ));
    };

    let after_help = format!(
        "<HW>=Use '-H <hw>' to specify hardware address type. Default: ether\n\
        List of possible hardware types (which support ARP): {}",
        get_hw_list_str(1)
    );
    let config: arp_common::Config =
        arp_common::parse_arp_cmd_args(args, ABOUT, USAGE, &after_help)?;
    arp_common::handle_input(config)
}

/// This the oe_app of arp
///
pub fn oe_app<'a>() -> Command<'a> {
    arp_common::arp_app(ABOUT, USAGE, "")
}
