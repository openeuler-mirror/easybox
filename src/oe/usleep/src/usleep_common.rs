//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::num::IntErrorKind;
use std::thread;
use std::time::Duration;

use uucore::error::{UResult, UUsageError};
use uucore::format_usage;
use uucore::msg_log::warnb;
use uucore::pretty_float::pretty_float;

use clap::{crate_version, Arg, Command};

/// usleep cmd parse error code.
pub static USLEEP_CMD_PARSE_ERROR: i32 = 2;

/// default microseconds value.
const DEFAULT_MICROSECONDS_VALUE: u64 = 1;

/// Config.
pub struct Config {
    ///
    pub usage: bool,
    ///
    pub oot: bool,
    ///
    pub microseconds: u64,
}

/// options.
pub mod options {
    ///
    pub static VERSION: &str = "version";
    ///
    pub static HELP: &str = "help";
    ///
    pub static USAGE: &str = "usage";
    ///
    pub static OOT: &str = "oot";
    ///
    pub static MICROSECONDS: &str = "microseconds";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let count: u64 = match options.get_many::<String>(options::MICROSECONDS) {
            Some(mut values) => {
                let value = values.next().unwrap();
                if let Some(_) = values.next() {
                    return Err(UUsageError::new(
                        USLEEP_CMD_PARSE_ERROR,
                        format!("exactly one argument (number of microseconds) must be used"),
                    ));
                }

                let (radix, skip) = match value {
                    v if v.starts_with("0x") || v.starts_with("0X") => (16, 2),
                    v if v.starts_with("0") => (8, 1),
                    _ => (10, 0),
                };

                match u64::from_str_radix(&value[skip..], radix) {
                    Ok(n) => n,
                    Err(e) => match e.kind() {
                        IntErrorKind::PosOverflow => u64::MAX,
                        _ => 0u64,
                    },
                }
            }
            None => DEFAULT_MICROSECONDS_VALUE,
        };

        Ok(Self {
            usage: options.contains_id(options::USAGE),
            oot: options.contains_id(options::OOT),
            microseconds: count,
        })
    }
}

///
pub fn parse_usleep_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = usleep_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn usleep_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::HELP)
                .short('?')
                .long(options::HELP)
                .help("Show this help message")
                .display_order(2),
        )
        .arg(
            Arg::new(options::VERSION)
                .short('v')
                .long(options::VERSION)
                .help("Display the version of this program, and exit")
                .display_order(0),
        )
        .arg(
            Arg::new(options::USAGE)
                .long(options::USAGE)
                .help("Display brief usage message")
                .display_order(3),
        )
        .arg(
            Arg::new(options::OOT)
                .short('o')
                .long(options::OOT)
                .help("oot says hey!")
                .display_order(1),
        )
        .arg(
            Arg::new(options::MICROSECONDS)
                .index(1)
                .multiple_occurrences(true),
        )
}

///
pub fn handle_input(config: Config) -> UResult<()> {
    if config.usage {
        println!(
            "Usage: usleep [-vo?] [-v|--version] [-o|--oot] [-?|--help] [--usage] [microseconds]"
        );
        return Ok(());
    }

    if config.oot {
        println!("oot says hey!");
        return Ok(());
    }

    warnb("warning: usleep is deprecated, and will be removed in near future!");
    warnb(&format!(
        "warning: use \"sleep {}\" instead...",
        pretty_float(config.microseconds as f64 / 1e6),
    ));

    thread::sleep(Duration::from_micros(config.microseconds));
    Ok(())
}
