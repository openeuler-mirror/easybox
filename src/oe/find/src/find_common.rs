//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use once_cell::sync::OnceCell;
use uucore::error::{UResult, USimpleError};
use uucore::{format_usage, show_warning, util_name};

use clap::{arg, value_parser, ArgAction, ArgGroup, ArgMatches, Command};
use walkdir::WalkDir;

use self::actions::format::NewLine;
use self::metadata::FindMetadata;

#[cfg(test)]
use self::metadata::ForgeMetadata;

pub mod actions;
pub mod metadata;
pub mod operators;
pub mod options;
pub mod parse;
pub mod tests;
pub mod unsafes;

const EXIT_FAILURE: i32 = 1;
const ADDITIONAL_HELP: &str = "Default path is the current directory; default expression is -print.
Expression may consist of: operators, options, tests, and actions.

Operators (decreasing precedence; -and is implicit where no others are given):
      ( EXPR )   ! EXPR   -not EXPR   EXPR1 -a EXPR2   EXPR1 -and EXPR2
      EXPR1 -o EXPR2   EXPR1 -or EXPR2   EXPR1 , EXPR2

Positional options (always true):
      -daystart -follow -nowarn -regextype -warn

Normal options (always true, specified before other expressions):
      -depth -files0-from FILE -maxdepth LEVELS -mindepth LEVELS
      -mount -noleaf -xdev -ignore_readdir_race -noignore_readdir_race

Tests (N can be +N or -N or N):
      -amin N -anewer FILE -atime N -cmin N -cnewer FILE -context CONTEXT
      -ctime N -empty -false -fstype TYPE -gid N -group NAME -ilname PATTERN
      -iname PATTERN -inum N -iwholename PATTERN -iregex PATTERN
      -links N -lname PATTERN -mmin N -mtime N -name PATTERN -newer FILE
      -nouser -nogroup -path PATTERN -perm [-/]MODE -regex PATTERN
      -readable -writable -executable
      -wholename PATTERN -size N[bcwkMG] -true -type [bcdpflsD] -uid N
      -used N -user NAME -xtype [bcdpfls]

Actions:
      -delete -print0 -printf FORMAT -fprintf FILE FORMAT -print
      -fprint0 FILE -fprint FILE -ls -fls FILE -prune -quit
      -exec COMMAND ; -exec COMMAND {} + -ok COMMAND ;
      -execdir COMMAND ; -execdir COMMAND {} + -okdir COMMAND ;

Other common options:
      --help                   display this help and exit
      --version                output version information and exit
";

/// Enum for -H, -L and -P, which control how find treats symbolic links.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum LinkMode {
    ///
    H,

    ///
    L,
    #[default]

    ///
    P,
}

/// Enum for -D option. For compatibility.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DebugOption {
    ///
    Exec,

    ///
    Opt,

    ///
    Rates,

    ///
    Search,

    ///
    Stat,

    ///
    Tree,

    ///
    All,

    ///
    Help,
}

impl FromStr for DebugOption {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exec" => Ok(Self::Exec),
            "opt" => Ok(Self::Opt),
            "rates" => Ok(Self::Rates),
            "search" => Ok(Self::Search),
            "stat" => Ok(Self::Stat),
            "tree" => Ok(Self::Tree),
            "all" => Ok(Self::All),
            "help" => Ok(Self::Help),
            _ => Err(()),
        }
    }
}

impl FromStr for LinkMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "H" => Ok(Self::H),
            "L" => Ok(Self::L),
            "P" => Ok(Self::P),
            _ => Err(()),
        }
    }
}

/// Find's command-line options. Unlike other utils, find divied command-line arguments into three
/// parts: Options, starting points and expressions. This struct only include options. For the
/// whole definition of find's arguments, see `Config` and `parse_find_cmd_args`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Options {
    link_mode: LinkMode,
    opt_level: u8,
    debug_opts: Vec<DebugOption>,
}

impl Options {
    /// Generate find general Config
    pub fn from(args_matches: &ArgMatches) -> UResult<Self> {
        let debug_opts = args_matches
            .get_many::<String>("debugopts")
            .map_or(vec![], |str_opts| {
                str_opts
                    .into_iter()
                    .filter_map(|opt| {
                        println!("{opt}");
                        DebugOption::from_str(opt).ok()
                    })
                    .collect()
            });

        let opt_level = *args_matches.get_one::<u8>("optlevel").expect("Default");

        let link_mode = {
            let h = args_matches
                .indices_of("H")
                .map_or(0, |v| v.last().unwrap_or(0));
            let l = args_matches
                .indices_of("L")
                .map_or(0, |v| v.last().unwrap_or(0));
            let p = args_matches
                .indices_of("P")
                .map_or(0, |v| v.last().unwrap_or(0));

            if h == 0 && l == 0 && p == 0 {
                LinkMode::P
            } else {
                match (p > h, p > l, h > l) {
                    (true, true, _) => LinkMode::P,
                    (false, true, _) | (false, false, true) => LinkMode::H,
                    (true, false, _) | (false, false, false) => LinkMode::L,
                }
            }
        };

        Ok(Options {
            link_mode,
            opt_level,
            debug_opts,
        })
    }
}

/// Arguments for Find's -regextype option.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexType {
    ///
    Rust,
}

impl Default for RegexType {
    fn default() -> Self {
        Self::Rust
    }
}

impl TryFrom<&str> for RegexType {
    type Error = USimpleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "default" => Ok(Self::default()),
            "rust" => Ok(Self::Rust),
            _ => Err(USimpleError {
                code: EXIT_FAILURE,
                message: format!("`{value}` is not a valid regex type"),
            }),
        }
    }
}

impl Display for RegexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RegexType::Rust => "Rust/Perl",
        })
    }
}

impl RegexType {
    ///
    pub fn create_re_from_pattern(&self, pattern: &str) -> UResult<regex::Regex> {
        match self {
            RegexType::Rust => regex::Regex::new(&format!("^{pattern}$")).map_err(|e| {
                USimpleError::new(1, format!("Cannot build the rust regex `{pattern}`: {e}"))
            }),
        }
    }

    ///
    pub fn create_case_insensitive_re_from_pattern(&self, pattern: &str) -> UResult<regex::Regex> {
        match self {
            RegexType::Rust => regex::RegexBuilder::new(&format!("^{pattern}$"))
                .case_insensitive(true)
                .build()
                .map_err(|e| {
                    USimpleError::new(
                        1,
                        format!("Cannot build the rust case-insensitive regex `{pattern}`: {e}"),
                    )
                }),
        }
    }

    ///
    pub fn create_re(&self, pattern: &str, case_insensitive: bool) -> UResult<regex::Regex> {
        if case_insensitive {
            self.create_case_insensitive_re_from_pattern(pattern)
        } else {
            self.create_re_from_pattern(pattern)
        }
    }
}

/// Find's options which control how filters work.
pub struct FilterOption {
    ///
    pub follow_link: bool,

    ///
    pub no_leaf: bool,

    ///
    pub daystart: bool,

    ///
    pub regex_type: RegexType,

    ///
    pub warn: bool,
}

impl FilterOption {
    ///
    pub fn new() -> Self {
        Self {
            follow_link: false,
            no_leaf: false,
            daystart: false,
            regex_type: RegexType::default(),
            warn: true,
        }
    }
}

impl Default for FilterOption {
    fn default() -> Self {
        Self::new()
    }
}

/// Find's options which control how walkdir and the whole program work.
pub struct GlobalOption {
    ///
    pub depth: bool,

    ///
    pub ignore_readdir_race: bool,

    ///
    pub max_depth: Option<usize>,

    ///
    pub min_depth: Option<usize>,

    ///
    pub xdev: bool,

    ///
    pub no_leaf: bool,

    ///
    pub arg_max: Option<i64>,

    ///
    pub posixly_correct: bool,
}

impl GlobalOption {
    ///
    pub fn new() -> Self {
        Self {
            depth: false,
            ignore_readdir_race: false,
            max_depth: None,
            min_depth: None,
            xdev: false,
            no_leaf: false,
            arg_max: unsafes::get_sys_arg_max(),
            posixly_correct: std::env::var("POSIXLY_CORRECT").is_ok(),
        }
    }
}

impl Default for GlobalOption {
    fn default() -> Self {
        Self::new()
    }
}

/// Definination of find's command-line arguments and its state.
#[derive(Default)]
pub struct Config {
    /// Link mode
    pub link_mode: LinkMode,

    /// Starting points for the searching paths.
    pub starting_points: Vec<String>,

    ///
    pub from_cli: bool,

    /// Debug flags
    pub debug_tree: bool,

    ///
    pub debug_exec: bool,

    ///
    pub debug_search: bool,

    ///
    pub debug_rates: bool,

    ///
    pub debug_stat: bool,

    /// Flag checking whether the given exprs has -ok.
    pub has_ok: bool,

    /// Flag checking whether the given exprs does anything.
    pub has_actions: bool,

    /// Status code
    pub status: i32,

    /// Options from the filter exprs, may change when build the filter.
    pub filter_option: FilterOption,

    /// Global options from the filter exprs.
    pub global_option: GlobalOption,

    /// Help message.
    pub help: String,

    /// Version.
    pub version: String,

    /// About message.
    pub about: String,
}

/// Generate find config
pub fn parse_find_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
) -> UResult<(Config, Box<dyn FindFilter>)> {
    let command = find_app(about, usage);
    let arg_list = args.collect_lossy();
    let (args_for_optionts, rest) = split_opts_and_starting_points(arg_list);
    let (starting_points, args_for_exprs) = split_starting_points_and_exprs(rest);

    let options = Options::from(&command.try_get_matches_from(args_for_optionts)?)?;

    let mut config = Config {
        link_mode: options.link_mode,
        starting_points,
        from_cli: true,
        has_ok: false,
        has_actions: false,
        status: 0,
        filter_option: FilterOption::default(),
        global_option: GlobalOption::default(),

        help: format!("{}\n{}", format_usage(usage), ADDITIONAL_HELP),
        version: env!("CARGO_PKG_VERSION").to_string(),
        about: about.to_string(),

        debug_tree: options
            .debug_opts
            .iter()
            .any(|opt| *opt == DebugOption::Tree),
        debug_exec: options
            .debug_opts
            .iter()
            .any(|opt| *opt == DebugOption::Exec),
        debug_search: options
            .debug_opts
            .iter()
            .any(|opt| *opt == DebugOption::Search),
        debug_rates: options
            .debug_opts
            .iter()
            .any(|opt| *opt == DebugOption::Rates),
        debug_stat: options
            .debug_opts
            .iter()
            .any(|opt| *opt == DebugOption::Stat),
    };

    let filters = parse::parse_filter_exprs(args_for_exprs.into_iter(), &mut config)?;
    if config.debug_tree {
        eprintln!("{filters:?}");
    }

    if config.link_mode == LinkMode::L {
        config.global_option.no_leaf = true;
    }

    let filters = if let Some(filters) = filters {
        if !config.has_actions {
            operators::and(
                filters,
                Box::new(actions::Print::new(actions::OutputTarget::Stdout, NewLine)),
            )
        } else {
            filters
        }
    } else {
        Box::new(actions::Print::new(actions::OutputTarget::Stdout, NewLine))
    };

    Ok((config, filters))
}

/// Command arguments setting
pub fn find_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(util_name())
        .override_usage(format_usage(usage))
        .about(about)
        .arg(arg!(-D[debugopts]).action(ArgAction::Append))
        .arg(
            arg!(-O[optlevel])
                .value_parser(value_parser!(u8))
                .default_value("1"),
        )
        .arg(arg!(H: -H).multiple_occurrences(true))
        .arg(arg!(L: -L).multiple_occurrences(true))
        .arg(arg!(P: -P).multiple_occurrences(true))
        .group(
            ArgGroup::new("link-mode")
                .args(&["H", "L", "P"])
                .multiple(true),
        )
}

/// Split the args vector to options and starting points.
fn split_opts_and_starting_points<T>(
    args: impl IntoIterator<Item = T>,
) -> (Vec<T>, impl IntoIterator<Item = T>)
where
    T: Into<String> + Clone,
{
    let mut args: Vec<_> = args.into_iter().collect();
    // Skip the command name.
    let mut id = 1;
    let len = args.len();

    while id < len {
        let s: String = args[id].clone().into();
        let arg_count = try_get_arg_count(&s);

        match arg_count {
            Some(step) => id += step + 1,
            None => break,
        }
    }

    let starting_points = args.split_off(id.min(len));
    (args, starting_points)
}

/// Get how many args the option need.
fn try_get_arg_count(arg_str: &str) -> Option<usize> {
    match arg_str {
        "find" => Some(0),
        "-H" | "-L" | "-P" => Some(0),
        "-D" | "-O" => Some(1),
        _ => None,
    }
}

/// Split the rest of the args to starting points and filter expressions.
fn split_starting_points_and_exprs<T>(
    starting_points: impl IntoIterator<Item = T>,
) -> (Vec<T>, Vec<T>)
where
    T: Into<String> + Clone,
{
    let mut starting_points: Vec<_> = starting_points.into_iter().collect();
    let mut id = 0;
    let len = starting_points.len();

    while id < len {
        let s = starting_points[id].clone().into();

        if is_expr_start(&s) {
            break;
        }
        id += 1;
    }

    let exprs = starting_points.split_off(id.min(len));
    (starting_points, exprs)
}

///
pub fn is_expr_start(s: &str) -> bool {
    s == "(" || s.starts_with('-')
}

///
pub struct FindFile {
    path: PathBuf,
    metadata: OnceCell<Box<dyn FindMetadata>>,
    #[allow(unused)]
    symlink_metadata: OnceCell<Box<dyn FindMetadata>>,

    starting_point: PathBuf,
    depth: usize,

    #[cfg(test)]
    pub forge_metadata: ForgeMetadata,
    #[cfg(test)]
    pub forge_symlink_metadata: ForgeMetadata,

    debug: bool,
}

impl FindFile {
    ///
    pub fn new(
        path: impl AsRef<Path>,
        starting_point: impl AsRef<Path>,
        depth: usize,
        debug: bool,
    ) -> Self {
        let path = path.as_ref();
        let starting_point = starting_point.as_ref();
        Self {
            path: path.to_owned(),
            metadata: OnceCell::new(),
            symlink_metadata: OnceCell::new(),

            starting_point: starting_point.to_owned(),
            depth,

            #[cfg(test)]
            forge_metadata: ForgeMetadata::default(),
            #[cfg(test)]
            forge_symlink_metadata: ForgeMetadata::default(),

            debug,
        }
    }

    /// Get the path of the file.
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Get the metadata of the file.
    #[cfg(not(test))]
    pub fn get_metadata(&self) -> UResult<&dyn FindMetadata> {
        self.metadata
            .get_or_try_init(|| {
                let m = std::fs::symlink_metadata(&self.path)?;
                if self.debug {
                    eprintln!("Debug stat: {:?}", self.path)
                }

                Ok(Box::new(m))
            })
            .map(|p| p.as_ref())
    }

    /// Get the metadata of the file. If the file is a symlink, then query the pointed file
    #[cfg(not(test))]
    pub fn get_pointed_metadata(&self) -> UResult<&dyn FindMetadata> {
        self.symlink_metadata
            .get_or_try_init(|| {
                let m = std::fs::metadata(&self.path)?;
                if self.debug {
                    eprintln!("Debug stat: {:?}", self.path)
                }

                Ok(Box::new(m))
            })
            .map(|p| p.as_ref())
    }

    #[cfg(test)]
    pub fn get_metadata(&self) -> UResult<&dyn FindMetadata> {
        self.metadata
            .get_or_try_init(|| {
                if self.debug {
                    eprintln!("Debug stat: {:?}", self.path)
                }
                UResult::Ok(Box::new(self.forge_metadata.clone()))
            })
            .map(|p| p.as_ref())
    }

    #[cfg(test)]
    pub fn get_pointed_metadata(&self) -> UResult<&dyn FindMetadata> {
        self.symlink_metadata
            .get_or_try_init(|| {
                if self.debug {
                    eprintln!("Debug stat: {:?}", self.path)
                }
                UResult::Ok(Box::new(self.forge_symlink_metadata.clone()))
            })
            .map(|p| p.as_ref())
    }
}

fn get_metadata(file: &FindFile, follow_link: bool) -> UResult<&dyn FindMetadata> {
    if follow_link {
        file.get_pointed_metadata()
    } else {
        file.get_metadata()
    }
}

/// Filters trait.
pub trait FindFilter: Debug {
    ///
    fn filter(&mut self, file: &FindFile) -> UResult<bool>;

    ///
    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        _side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        self.filter(file)
    }

    ///
    fn has_side_effects(&self) -> bool {
        false
    }

    ///
    fn based_on_name(&self) -> bool {
        true
    }
}

/// Trait for construction from the expr args;
pub trait FindConstruct: Sized {
    ///
    fn construct_from_iter(_iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        Err(USimpleError::new(
            1,
            "The program doesn't know how to build this filter",
        ))
    }

    ///
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        _config: &Config,
    ) -> UResult<Self> {
        Self::construct_from_iter(iter)
    }
}

impl<T: FindFilter> FindFilter for Box<T> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        (**self).filter(file)
    }

    fn filter_with_side_effects(
        &mut self,
        file: &FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        (**self).filter_with_side_effects(file, side_effects)
    }

    fn has_side_effects(&self) -> bool {
        (**self).has_side_effects()
    }

    fn based_on_name(&self) -> bool {
        (**self).based_on_name()
    }
}

/// Filters which can modify the configuration both positionally and globally.
pub trait FindOption: FindFilter {
    ///
    fn take_effect(&self, config: &mut Config) -> UResult<()>;
}

/// Enum for side effects. Tell the program how to deal with the current searching path.
pub enum FindInstruction {
    ///
    Prune,

    ///
    Exit(Option<i32>),
}

/// Search the starting points in the config with the given filters.
pub fn search(config: &mut Config, filters: &mut dyn FindFilter) -> UResult<()> {
    let starting_points = if config.from_cli && config.starting_points.is_empty() {
        vec![".".to_string()]
    } else if !config.from_cli && config.starting_points.is_empty() {
        return Err(USimpleError::new(
            1,
            "Search failed: No starting points in files",
        ));
    } else {
        config.starting_points.clone()
    };

    for starting_point in starting_points {
        if let Err(e) = search_starting_point(&starting_point, config, filters) {
            show_warning!("Search starting point {} failed: {e}", starting_point);
        }
    }

    Ok(())
}

/// Search only one starting point.
pub fn search_starting_point(
    starting_point: &str,
    config: &mut Config,
    filters: &mut dyn FindFilter,
) -> UResult<()> {
    let (min_depth, max_depth) = (
        config.global_option.min_depth,
        config.global_option.max_depth,
    );
    let depth = config.global_option.depth;
    let ignore_readdir_race = config.global_option.ignore_readdir_race;
    let xdev = config.global_option.xdev;
    let _no_leaf = config.global_option.no_leaf;

    let root_dev = if xdev {
        let m = std::fs::metadata(starting_point)?;
        Some(m.st_dev())
    } else {
        None
    };

    let mut walker = WalkDir::new(starting_point);
    if let Some(min_depth) = min_depth {
        walker = walker.min_depth(min_depth);
    }
    if let Some(max_depth) = max_depth {
        walker = walker.max_depth(max_depth);
    }
    walker = walker.contents_first(depth);
    let mut it = walker.into_iter();

    while let Some(entry) = it.next() {
        let mut side_effects = vec![];

        match entry {
            Err(e) => {
                if !ignore_readdir_race {
                    show_warning!("Walkdir failed: {e}");
                }
            }
            Ok(entry) => {
                if config.debug_search {
                    eprintln!("Debug search: consider searching {:?}", entry.path());
                }

                if let Err(e) = search_entry(
                    entry.clone(),
                    starting_point,
                    root_dev,
                    filters,
                    &mut side_effects,
                    config,
                ) {
                    show_warning!(
                        "Filter failed when filtering {}: {e}",
                        entry.path().to_string_lossy()
                    );
                }
            }
        }

        for effect in side_effects {
            match effect {
                FindInstruction::Prune => {
                    it.skip_current_dir();
                }
                FindInstruction::Exit(status) => match status {
                    Some(s) => std::process::exit(s),
                    None => std::process::exit(config.status),
                },
            }
        }
    }

    Ok(())
}

/// Apply the filter on one single file.
fn search_entry(
    entry: walkdir::DirEntry,
    starting_point: &str,
    root_dev: Option<u64>,
    filters: &mut dyn FindFilter,
    side_effects: &mut Vec<FindInstruction>,
    config: &Config,
) -> UResult<()> {
    let path = entry.path();
    let depth = entry.depth();
    let file = FindFile::new(path, starting_point, depth, config.debug_stat);

    if let Some(root_dev) = root_dev {
        let metadata = file.get_metadata()?;
        if metadata.st_dev() != root_dev {
            side_effects.push(FindInstruction::Prune);
            return Ok(());
        }
    }

    // Just drop the filter result -- we already append a `print` filter when no other actions
    // exist.
    let _ = filters.filter_with_side_effects(&file, side_effects)?;

    Ok(())
}

#[allow(unused)]
#[macro_export]
///
macro_rules! this_filter_is_based_on_metadata {
    () => {
        fn based_on_name(&self) -> bool {
            false
        }
    };
}

#[allow(unused)]
#[macro_export]
///
macro_rules! this_filter_has_side_effects {
    () => {
        fn has_side_effects(&self) -> bool {
            true
        }
    };
}

#[allow(unused)]
#[macro_export]
///
macro_rules! this_filter_consume_no_args {
    () => {
        fn construct_from_iter(_iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
            Ok(Self::new())
        }
    };
}

#[allow(unused)]
#[macro_export]
///
macro_rules! this_filter_built_with_config {
    () => {
        fn construct_from_iter_with_config(
            _iter: &mut impl Iterator<Item = String>,
            config: &Config,
        ) -> UResult<Self> {
            Ok(Self::new(config))
        }
    };
}

fn get_uname_by_uid(uid: u32) -> Option<String> {
    users::get_user_by_uid(uid).map(|u| u.name().to_string_lossy().to_string())
}

fn get_gname_by_gid(gid: u32) -> Option<String> {
    users::get_group_by_gid(gid).map(|g| g.name().to_string_lossy().to_string())
}
