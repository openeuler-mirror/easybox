//! This file is part of the uutils coreutils package.
//
// (c) Lin Guantao <moyihust@gmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
// use std::io::{stdin, Read};

use clap::Command;
use uucore::{error::UResult, help_section, help_usage};

///
pub mod free_common;

const ABOUT: &str = help_section!("about", "free.md");
const USAGE: &str = help_usage!("free.md");

/// This the main of free
///
#[uucore::main]
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let args: Vec<String> = args.map(|arg| arg.into_string().unwrap()).collect();
    let config: free_common::Config =
        free_common::parse_free_cmd_args(args.iter().map(AsRef::as_ref), ABOUT, USAGE)?;

    free_common::handle_input(&config)
}

/// This the oe_app of free
///
pub fn oe_app<'a>() -> Command<'a> {
    free_common::free_app(ABOUT, USAGE)
}
