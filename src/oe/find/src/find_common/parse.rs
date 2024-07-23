//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use uucore::error::{UResult, USimpleError};

use crate::find_common::{
    actions,
    operators::{and, not, or},
};

use super::{
    actions::format::{FormatString, NewLine, NullTerminated},
    options,
    tests::{
        self,
        time_type::{self, Modify},
    },
    Config, FindConstruct, FindFilter, FindOption,
};

macro_rules! options_parser {
    ($iter:ident, $config:ident, $option_ty:ty) => {{
        let filter = <$option_ty>::construct_from_iter_with_config($iter, $config)?;
        filter.take_effect($config)?;
        Ok(Box::new(filter))
    }};
}

macro_rules! filter_parser {
    ($iter:ident, $config:ident, $filter_ty:ty) => {{
        let filter = <$filter_ty>::construct_from_iter_with_config($iter, $config)?;
        Ok(Box::new(filter))
    }};
}

macro_rules! action_parser {
    ($iter:ident, $config:ident, $filter_ty:ty) => {{
        let filter = <$filter_ty>::construct_from_iter_with_config($iter, $config)?;
        $config.has_actions = true;
        Ok(Box::new(filter))
    }};
}

/// Parse top-level expressions and build the filters with logical combinators.
pub fn parse_filter_exprs(
    args: impl Iterator<Item = String>,
    config: &mut Config,
) -> UResult<Option<Box<dyn FindFilter>>> {
    let mut left_hand: Option<Box<dyn FindFilter>> = None;
    let mut iter = args.peekable();

    while let Some(arg) = iter.peek() {
        let arg = arg.as_str();

        match arg {
            "-a" | "-and" => {
                if left_hand.is_none() {
                    return Err(USimpleError::new(
                        1,
                        "-a/-and is a binary operator. No filters before -a/-and",
                    ));
                }

                iter.next(); // Consume the operator specifier.
                left_hand = Some(and(
                    left_hand.unwrap(),
                    parse_single_filter(&mut iter, config)?,
                ))
            }
            "-o" | "-or" => {
                if left_hand.is_none() {
                    return Err(USimpleError::new(
                        1,
                        "-o/-or is a binary operator. No filters before -o/-or",
                    ));
                }

                iter.next(); // Consume the operator specifier.
                left_hand = Some(or(
                    left_hand.unwrap(),
                    parse_single_filter(&mut iter, config)?,
                ))
            }
            "," => {
                if left_hand.is_none() {
                    return Err(USimpleError::new(
                        1,
                        ", is a binary operator. No filters before ,",
                    ));
                }

                iter.next(); // Consume the operator specifier.
                left_hand = Some(or(
                    left_hand.unwrap(),
                    parse_single_filter(&mut iter, config)?,
                ))
            }
            "!" | "-not" => {
                iter.next(); // Consume the operator specifier.
                let filter = not(parse_single_filter(&mut iter, config)?);
                left_hand = if let Some(lhs) = left_hand {
                    Some(and(lhs, filter))
                } else {
                    Some(filter)
                }
            }
            "(" => {
                iter.next(); // Consume the `(`.
                let filter = parse_parenthesed_exprs(&mut iter, config)?;
                left_hand = if let Some(lhs) = left_hand {
                    Some(and(lhs, filter))
                } else {
                    Some(filter)
                }
            }

            _ => {
                let filter = parse_single_filter(&mut iter, config)?;
                left_hand = if let Some(lhs) = left_hand {
                    Some(and(lhs, filter))
                } else {
                    Some(filter)
                }
            }
        }
    }

    Ok(left_hand)
}

/// Parse one single filter (except operators) and its arguments.
fn parse_single_filter(
    iter: &mut impl Iterator<Item = String>,
    config: &mut Config,
) -> UResult<Box<dyn FindFilter>> {
    let filter_name = iter
        .next()
        .ok_or(USimpleError::new(1, "Cannot get the name of the filter"))?;
    match filter_name.as_str() {
        // Positional Options
        "-daystart" => options_parser!(iter, config, options::DayStart),
        // `-follow` implies `-noleaf`
        "-follow" => {
            config.global_option.no_leaf = true;
            options_parser!(iter, config, options::Follow)
        }
        "-regextype" => options_parser!(iter, config, options::RegexTypeSetting),
        "-warn" => options_parser!(iter, config, options::Warn),
        "-nowarn" => options_parser!(iter, config, options::NoWarn),

        // Global Options
        "-d" | "-depth" => options_parser!(iter, config, options::Depth),
        "-files0-from" => options_parser!(iter, config, options::Files0From),
        "-help" | "--help" => {
            println!("{}", config.help);
            std::process::exit(0)
        }
        "ignore_readdir_race" => options_parser!(iter, config, options::IgnoreReaddirRace),
        "-maxdepth" => options_parser!(iter, config, options::MaxDepth),
        "-mindepth" => options_parser!(iter, config, options::MinDepth),
        "-mount" => options_parser!(iter, config, options::Mount),
        "-noignore_readdir_race" => options_parser!(iter, config, options::NoIgnoreReaddirRace),
        "-noleaf" => options_parser!(iter, config, options::NoLeaf),
        "-version" | "--version" => {
            println!("find {} - {}", config.version, config.about);
            std::process::exit(0)
        } // Print version and exit.
        "-xdev" => options_parser!(iter, config, options::XDev),

        // Actions which turn off the default `-print`
        // `-delete` implies `-depth`.
        "-delete" => {
            config.global_option.depth = true;
            action_parser!(iter, config, actions::Delete)
        }
        "-exec" => action_parser!(iter, config, actions::Exec),
        "-execdir" => action_parser!(iter, config, actions::ExecDir),
        "-fls" => action_parser!(iter, config, actions::FLs),
        "-fprint" => action_parser!(iter, config, actions::FilePrint<NewLine>),
        "-fprint0" => action_parser!(iter, config, actions::FilePrint<NullTerminated>),
        "-fprintf" => action_parser!(iter, config, actions::FilePrint<FormatString>),
        "-ls" => action_parser!(iter, config, actions::Ls),
        "-ok" => action_parser!(iter, config, actions::OkExec),
        "-okdir" => action_parser!(iter, config, actions::OkExecDir),
        "-print" => action_parser!(iter, config, actions::Print<NewLine>),
        "-print0" => action_parser!(iter, config, actions::Print<NullTerminated>),
        "-printf" => action_parser!(iter, config, actions::Print<FormatString>),

        // Actions
        "-prune" => filter_parser!(iter, config, actions::Prune),
        "-quit" => filter_parser!(iter, config, actions::Quit),

        // Tests
        "-amin" => filter_parser!(iter, config, tests::AccessMin),
        "-anewer" => filter_parser!(iter, config, tests::AccessNewer),
        "-atime" => filter_parser!(iter, config, tests::AccessTime),
        "-cmin" => filter_parser!(iter, config, tests::ChangeMin),
        "-cnewer" => filter_parser!(iter, config, tests::ChangeNewer),
        "-ctime" => filter_parser!(iter, config, tests::ChangeTime),
        "-empty" => filter_parser!(iter, config, tests::Empty),
        "-executable" => filter_parser!(iter, config, tests::Executable),
        "-false" => filter_parser!(iter, config, tests::False),
        "-fstype" => filter_parser!(iter, config, tests::FileSystemType),
        "-gid" => filter_parser!(iter, config, tests::GroupId),
        "-group" => filter_parser!(iter, config, tests::Group),
        "-ilname" => filter_parser!(iter, config, tests::InsensitiveLinkedName),
        "-iname" => filter_parser!(iter, config, tests::InsensitiveName),
        "-inum" => filter_parser!(iter, config, tests::Inode),
        "-ipath" => filter_parser!(iter, config, tests::InsensitivePath),
        "-iregex" => filter_parser!(iter, config, tests::InsensitiveRegex),
        "-iwholename" => filter_parser!(iter, config, tests::InsensitiveWholeName),
        "-links" => filter_parser!(iter, config, tests::HardLinkCount),
        "-lname" => filter_parser!(iter, config, tests::LinkedName),
        "-mmin" => filter_parser!(iter, config, tests::ModifyMin),
        "-mtime" => filter_parser!(iter, config, tests::ModifyTime),
        "-name" => filter_parser!(iter, config, tests::Name),
        "-newer" => filter_parser!(iter, config, tests::NewerXY<Modify, Modify>),
        "-nogroup" => filter_parser!(iter, config, tests::NoGroup),
        "-nouser" => filter_parser!(iter, config, tests::NoUser),
        "-path" => filter_parser!(iter, config, tests::FilterPath),
        "-perm" => filter_parser!(iter, config, tests::Perm),
        "-readable" => filter_parser!(iter, config, tests::Readable),
        "-regex" => filter_parser!(iter, config, tests::Regex),
        "-samefile" => filter_parser!(iter, config, tests::SameFile),
        "-size" => filter_parser!(iter, config, tests::Size),
        "-true" => filter_parser!(iter, config, tests::True),
        "-type" => filter_parser!(iter, config, tests::Type),
        "-uid" => filter_parser!(iter, config, tests::UserId),
        "-used" => filter_parser!(iter, config, tests::Used),
        "-user" => filter_parser!(iter, config, tests::User),
        "-wholename" => filter_parser!(iter, config, tests::WholeName),
        "-writable" => filter_parser!(iter, config, tests::Writable),
        "-xtype" => filter_parser!(iter, config, tests::XType),

        // #[cfg(feature = "selinux")]
        // "-context" => filter_parser!(iter, config, tests::SELinuxContext),

        // -newerXY
        s if s.starts_with("-newer") && s.len() == 8 => {
            let (x, y) = (s.as_bytes()[6], s.as_bytes()[7]);

            match (x, y) {
                (b'a', b'a') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Access, time_type::Access>
                ),
                (b'a', b'm') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Access, time_type::Modify>
                ),
                (b'a', b'c') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Access, time_type::Change>
                ),
                (b'a', b't') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Access, time_type::DateString>
                ),

                (b'm', b'a') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Modify, time_type::Access>
                ),
                (b'm', b'm') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Modify, time_type::Modify>
                ),
                (b'm', b'c') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Modify, time_type::Change>
                ),
                (b'm', b't') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Modify, time_type::DateString>
                ),

                (b'c', b'a') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Change, time_type::Access>
                ),
                (b'c', b'm') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Change, time_type::Modify>
                ),
                (b'c', b'c') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Change, time_type::Change>
                ),
                (b'c', b't') => filter_parser!(
                    iter,
                    config,
                    tests::NewerXY::<time_type::Change, time_type::DateString>
                ),

                _ => Err(USimpleError::new(1, "Invalid XY pair for newer: {x}, {y}")),
            }
        }
        s => Err(USimpleError::new(
            1,
            format!("{} is an invalid name for filter", s),
        )),
    }
}

/// Consume the args till the occurrence of `)`. Build filters from the args consumed.
fn parse_parenthesed_exprs(
    iter: &mut impl Iterator<Item = String>,
    config: &mut Config,
) -> UResult<Box<dyn FindFilter>> {
    let mut args = vec![];

    for arg in iter {
        if arg == ")" {
            let parenthesized = parse_filter_exprs(args.into_iter(), config)?;
            return parenthesized.ok_or(USimpleError::new(1, "Empty parentheses are illegal"));
        }

        args.push(arg);
    }

    Err(USimpleError::new(1, "No matching closing parentheses"))
}
