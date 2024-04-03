//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::lib_attr;
use std::io::Read;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

use uucore::error::{UResult, UUsageError};
use uucore::format_usage;

use clap::{crate_version, Arg, ArgMatches, Command};
use lib_attr::{attr_get, attr_list, attr_remove, attr_set};
use libc::EXIT_FAILURE;

const BAD_USAGE_MESSAGE: &str = "Only one of -s, -g, -r, or -l allowed";
const BAD_USAGE_MESSAGE_VALUE: &str = "-V only allowed with -s";
const BAD_USAGE_MESSAGE_V_MUST_WITH_S: &str = "-V only allowed with -s";

/// Config
pub struct Config {
    /// Set to false if should keep output quiet
    pub verbose: bool,
    /// Follow symbolic link
    pub follow: bool,
    /// Operation type
    pub opflag: Operation,
    /// Is on the root attribute namespace
    pub rootflag: bool,
    /// Is on the security attribute namespace
    pub secureflag: bool,
    /// The attribute name
    pub attrname: String,
    /// The attribute value
    pub attrvalue: Option<String>,
    /// The file to operate
    pub filename: String,
}

/// Command Options
pub mod options {
    ///
    pub const SET_ATTR: &str = "s";
    ///
    pub const ATTR_VALUE: &str = "V";
    ///
    pub const GET_ATTR: &str = "g";
    ///
    pub const REMOVE_ATTR: &str = "r";
    ///
    pub const LIST_ATTR: &str = "l";
    ///
    pub const FOLLOW_LINK: &str = "L";
    ///
    pub const ROOT_FLAG: &str = "R";
    ///
    pub const SECURE_FLAG: &str = "S";
    ///
    pub const VERBOSE: &str = "q";
    ///
    pub const FILE_NAME: &str = "pathname";
}

/// Operation types
#[derive(PartialEq)]
pub enum Operation {
    /// Empty operation
    NONEOP,
    /// Set operation
    SETOP,
    /// Get operation
    GETOP,
    /// Remove operation
    REMOVEOP,
    /// List operation
    LISTOP,
}

impl Operation {
    /// Get operation from options string
    pub fn from(optstr: &str) -> Operation {
        match optstr {
            options::SET_ATTR => Operation::SETOP,
            options::GET_ATTR => Operation::GETOP,
            options::LIST_ATTR => Operation::LISTOP,
            options::REMOVE_ATTR => Operation::REMOVEOP,
            _ => Operation::NONEOP,
        }
    }
}

impl Config {
    /// Generate attr general Config
    pub fn from(args_matches: &ArgMatches) -> UResult<Self> {
        let mut opflag = Operation::NONEOP;
        let mut attrvalue: Option<String> = None;
        let mut attrname: Option<String> = None;
        let operation_optlist = [
            options::SET_ATTR,
            options::GET_ATTR,
            options::REMOVE_ATTR,
            options::LIST_ATTR,
        ];
        for opt in operation_optlist {
            if args_matches.contains_id(opt) {
                if opflag != Operation::NONEOP {
                    return Err(UUsageError::new(EXIT_FAILURE, BAD_USAGE_MESSAGE));
                }
                opflag = Operation::from(opt);
                attrname = args_matches.get_one::<String>(opt).cloned();
            }
        }
        if let Some(val) = args_matches.get_one::<String>(options::ATTR_VALUE) {
            if opflag != Operation::NONEOP && opflag != Operation::SETOP {
                return Err(UUsageError::new(EXIT_FAILURE, BAD_USAGE_MESSAGE_VALUE));
            }
            opflag = Operation::SETOP;
            attrvalue = Some(val.to_string());
            if attrname.is_none() {
                return Err(UUsageError::new(
                    EXIT_FAILURE,
                    BAD_USAGE_MESSAGE_V_MUST_WITH_S,
                ));
            }
        }
        Ok(Self {
            opflag,
            attrname: attrname.unwrap_or_default(),
            attrvalue,
            filename: args_matches
                .get_one::<String>(options::FILE_NAME)
                .unwrap()
                .to_string(),
            follow: args_matches.get_flag(options::FOLLOW_LINK),
            rootflag: args_matches.get_flag(options::ROOT_FLAG),
            secureflag: args_matches.get_flag(options::SECURE_FLAG),
            verbose: args_matches.get_flag(options::VERBOSE),
        })
    }
}

/// Generate attr general Config
pub fn parse_attr_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = attr_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

/// Command arguments setting
pub fn attr_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .arg_required_else_help(true)
        // Format arguments.
        .arg(
            Arg::new(options::SET_ATTR)
                .short('s')
                .display_order(1)
                .takes_value(true)
                .value_name("attrname")
                .help("set the named attribute of the object to the given value"),
        )
        .arg(
            Arg::new(options::ATTR_VALUE)
                .short('V')
                .takes_value(true)
                .display_order(5)
                .value_name("attrvalue")
                .help("set the value of the target attribute"),
        )
        .arg(
            Arg::new(options::GET_ATTR)
                .short('g')
                .takes_value(true)
                .display_order(2)
                .value_name("attrname")
                .help("search the named object and print the value associated with that attribute name"),
        )
        .arg(
            Arg::new(options::REMOVE_ATTR)
                .short('r')
                .takes_value(true)
                .display_order(3)
                .value_name("attrname")
                .help("remove an attribute with the given name from the object if the attribute exists"),
        )
        .arg(
            Arg::new(options::LIST_ATTR)
                .short('l')
                .takes_value(false)
                .display_order(4)
                .help("list the names of all the attributes that are associated with the object"),
        )
        .arg(
            Arg::new(options::FOLLOW_LINK)
                .short('L')
                .action(clap::ArgAction::SetTrue)
                .help("operate on the attributes of the object referenced by the symbolic link"),
        )
        .arg(
            Arg::new(options::ROOT_FLAG)
                .short('R')
                .action(clap::ArgAction::SetTrue)
                .help("operate in the root attribute namespace rather that the USER attribute namespace"),
        )
        .arg(
            Arg::new(options::SECURE_FLAG)
                .short('S')
                .action(clap::ArgAction::SetTrue)
                .help("specifie use of the security attribute namespace"),
        )
        .arg(
            Arg::new(options::VERBOSE)
                .short('q')
                .action(clap::ArgAction::SetFalse)
                .help("be quiet, output error messages (to stderr) but will not print status messages"),
        )
        .arg(Arg::new(options::FILE_NAME).index(1).hide(true).required(true))
}

/// Set attribute and read value from stdin if need
pub fn handle_setop(config: &Config) -> UResult<()> {
    let mut attrvalue = Vec::<u8>::default();
    match &config.attrvalue {
        Some(val) => attrvalue = <String as Clone>::clone(val).into_bytes(),
        None => {
            std::io::stdin().read_to_end(&mut attrvalue)?;
        }
    };
    attr_set(config, &attrvalue)?;
    if config.verbose {
        println!(
            "Attribute \"{}\" set to a {} byte value for {}:",
            config.attrname,
            attrvalue.len(),
            config.filename
        );
        std::io::stdout().write_all(&attrvalue)?;
        print!("\n");
    }
    Ok(())
}

/// Get attribute from file and output detail if need
pub fn handle_getop(config: &Config) -> UResult<()> {
    let attrvalue = attr_get(config)?;
    if config.verbose {
        println!(
            "Attribute \"{}\" had a {} byte value for {}:",
            config.attrname,
            attrvalue.len(),
            config.filename
        );
    }
    std::io::stdout().write_all(&attrvalue)?;
    if config.verbose {
        print!("\n");
    }
    Ok(())
}

/// Remove attribute from file
pub fn handle_removeop(config: &Config) -> UResult<()> {
    attr_remove(config)?;
    Ok(())
}

/// List attributes from file and output them
pub fn handle_listop(config: &Config) -> UResult<()> {
    let alist = attr_list(config)?;
    for i in alist {
        if config.verbose {
            println!(
                "Attribute {:?} has a {} byte value for {}",
                i.0, i.1, config.filename
            );
        } else {
            std::io::stdout().write_all(i.0.as_os_str().as_bytes())?;
            print!("\n");
        }
    }
    Ok(())
}
