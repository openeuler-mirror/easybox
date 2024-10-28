//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::io::Read;
use std::path::PathBuf;

use crate::this_filter_consume_no_args;

use super::Config;
use super::FindConstruct;
use super::FindFile;
use super::FindFilter;
use super::FindOption;
use super::RegexType;
use uucore::error::UResult;
use uucore::error::USimpleError;

#[macro_export]
///
macro_rules! do_nothing_and_return_true {
    () => {
        fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
            Ok(true)
        }
    };
}

#[macro_export]
///
macro_rules! default_option_configuration {
    () => {
        $crate::this_filter_has_side_effects!();
        $crate::do_nothing_and_return_true!();
    };
}

// Positional Options

#[derive(Debug)]
///
pub struct DayStart;

impl DayStart {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for DayStart {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for DayStart {
    default_option_configuration!();
}

impl FindConstruct for DayStart {
    this_filter_consume_no_args!();
}

impl FindOption for DayStart {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.filter_option.daystart = true;

        Ok(())
    }
}

#[derive(Debug)]

///

pub struct Follow;

impl Follow {
    ///

    pub fn new() -> Self {
        Self
    }
}

impl Default for Follow {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for Follow {
    default_option_configuration!();
}

impl FindConstruct for Follow {
    this_filter_consume_no_args!();
}

impl FindOption for Follow {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.filter_option.follow_link = true;

        Ok(())
    }
}

#[derive(Debug)]
///
pub struct WarnSetting<const F: bool>;

impl<const F: bool> WarnSetting<F> {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl<const F: bool> Default for WarnSetting<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const F: bool> FindFilter for WarnSetting<F> {
    default_option_configuration!();
}

impl<const F: bool> FindConstruct for WarnSetting<F> {
    this_filter_consume_no_args!();
}

impl<const F: bool> FindOption for WarnSetting<F> {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.filter_option.warn = F;

        Ok(())
    }
}

///
pub type Warn = WarnSetting<true>;

///
pub type NoWarn = WarnSetting<false>;

#[derive(Debug)]

///
pub struct RegexTypeSetting {
    regex_type: RegexType,
}

impl RegexTypeSetting {
    ///
    pub fn new(regex_type: RegexType) -> Self {
        Self { regex_type }
    }
}

impl FindFilter for RegexTypeSetting {
    default_option_configuration!();
}

impl FindConstruct for RegexTypeSetting {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for regextype"))
            .and_then(|arg| {
                let regex_type = RegexType::try_from(arg.as_str())?;
                Ok(Self::new(regex_type))
            })
    }
}

impl FindOption for RegexTypeSetting {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.filter_option.regex_type = self.regex_type;

        Ok(())
    }
}

// Global Options

#[derive(Debug)]

///
pub struct Depth;

impl Depth {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for Depth {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for Depth {
    default_option_configuration!();
}

impl FindConstruct for Depth {
    this_filter_consume_no_args!();
}

impl FindOption for Depth {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.global_option.depth = true;

        Ok(())
    }
}

#[derive(Debug)]
enum FilesSource {
    Stdin,
    File(PathBuf),
}

#[derive(Debug)]

///
pub struct Files0From {
    file: FilesSource,
}

impl Files0From {
    ///
    pub fn new_file(file: PathBuf) -> Self {
        Self {
            file: FilesSource::File(file),
        }
    }

    ///
    pub fn new_stdin() -> Self {
        Self {
            file: FilesSource::Stdin,
        }
    }
}

impl FindFilter for Files0From {
    default_option_configuration!();
}

impl FindConstruct for Files0From {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        if config.has_ok {
            return Err(USimpleError::new(1, "Cannot combine -files0-from with -ok"));
        }

        iter.next()
            .ok_or(USimpleError::new(1, "No arg for files0-from"))
            .map(|arg| match arg.as_str() {
                "-" => Self::new_stdin(),
                path => Self::new_file(path.into()),
            })
    }
}

impl FindOption for Files0From {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        let mut buf = String::new();

        match &self.file {
            FilesSource::Stdin => {
                let mut stdin = std::io::stdin();
                stdin.read_to_string(&mut buf)?;
            }
            FilesSource::File(file) => {
                let mut file = std::fs::File::open(file)?;
                file.read_to_string(&mut buf)?;
            }
        }

        let mut starting_points: Vec<String> = buf.split('\0').map(|s| s.to_owned()).collect();
        let len = starting_points.len();
        if len > 1 && starting_points[len - 1].is_empty() {
            starting_points.remove(len - 1);
        }

        if config.from_cli && !config.starting_points.is_empty() {
            return Err(USimpleError::new(
                1,
                "Cannot specify starting points both in arguments and in -files0-from option",
            ));
        } else {
            config.from_cli = false;
            config.starting_points.extend(starting_points);
        };

        Ok(())
    }
}

#[derive(Debug)]

///
pub struct IgnoreReaddirRaceSetting<const F: bool>;

impl<const F: bool> Default for IgnoreReaddirRaceSetting<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const F: bool> IgnoreReaddirRaceSetting<F> {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl<const F: bool> FindFilter for IgnoreReaddirRaceSetting<F> {
    default_option_configuration!();
}

impl<const F: bool> FindConstruct for IgnoreReaddirRaceSetting<F> {
    this_filter_consume_no_args!();
}

impl<const F: bool> FindOption for IgnoreReaddirRaceSetting<F> {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.global_option.ignore_readdir_race = F;
        Ok(())
    }
}

///

pub type IgnoreReaddirRace = IgnoreReaddirRaceSetting<true>;

///
pub type NoIgnoreReaddirRace = IgnoreReaddirRaceSetting<false>;

#[derive(Debug)]

///
pub struct MaxDepth {
    depth: usize,
}

impl MaxDepth {
    ///
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

impl FindFilter for MaxDepth {
    default_option_configuration!();
}

impl FindConstruct for MaxDepth {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -maxdepth"))
            .and_then(|arg| {
                let depth = arg
                    .parse::<usize>()
                    .map_err(|e| USimpleError::new(1, e.to_string()))?;
                Ok(Self::new(depth))
            })
    }
}

impl FindOption for MaxDepth {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.global_option.max_depth = Some(self.depth);
        Ok(())
    }
}

#[derive(Debug)]
///
pub struct MinDepth {
    depth: usize,
}

impl MinDepth {
    ///
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

impl FindFilter for MinDepth {
    default_option_configuration!();
}

impl FindConstruct for MinDepth {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -mindepth"))
            .and_then(|arg| {
                let depth = arg
                    .parse::<usize>()
                    .map_err(|e| USimpleError::new(1, e.to_string()))?;
                Ok(Self::new(depth))
            })
    }
}

impl FindOption for MinDepth {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.global_option.min_depth = Some(self.depth);
        Ok(())
    }
}

#[derive(Debug)]
///
pub struct XDev;

impl XDev {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for XDev {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for XDev {
    default_option_configuration!();
}

impl FindConstruct for XDev {
    this_filter_consume_no_args!();
}

impl FindOption for XDev {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.global_option.xdev = true;
        Ok(())
    }
}

///
pub type Mount = XDev;

#[derive(Debug)]

///
pub struct NoLeaf;

impl NoLeaf {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoLeaf {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for NoLeaf {
    default_option_configuration!();
}

impl FindConstruct for NoLeaf {
    this_filter_consume_no_args!();
}

impl FindOption for NoLeaf {
    fn take_effect(&self, config: &mut super::Config) -> UResult<()> {
        config.global_option.no_leaf = true;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::find_common::{
        options::{
            Depth, Follow, IgnoreReaddirRace, NoIgnoreReaddirRace, NoWarn, RegexTypeSetting, Warn,
            XDev,
        },
        Config, FindConstruct, FindOption, RegexType,
    };

    use super::DayStart;

    #[test]
    fn daystart() {
        let mut config = Config::default();
        DayStart::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(config.filter_option.daystart);
    }

    #[test]
    fn follow() {
        let mut config = Config::default();
        Follow::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(config.filter_option.follow_link);
    }

    #[test]
    fn warn() {
        let mut config = Config::default();
        Warn::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(config.filter_option.warn);
    }

    #[test]
    fn no_warn() {
        let mut config = Config::default();
        NoWarn::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(!config.filter_option.warn);
    }

    #[test]
    fn set_regex_type_to_rust() {
        let mut config = Config::default();
        RegexTypeSetting::construct_from_iter_with_config(
            &mut vec!["rust".to_string()].into_iter(),
            &config,
        )
        .unwrap()
        .take_effect(&mut config)
        .unwrap();
        assert_eq!(config.filter_option.regex_type, RegexType::Rust);
    }

    #[test]
    fn depth_first() {
        let mut config = Config::default();
        Depth::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(config.global_option.depth);
    }

    #[test]
    fn ignore_readdir_race() {
        let mut config = Config::default();
        IgnoreReaddirRace::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(config.global_option.ignore_readdir_race);
    }

    #[test]
    fn no_ignore_readdir_race() {
        let mut config = Config::default();
        NoIgnoreReaddirRace::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(!config.global_option.ignore_readdir_race);
    }

    #[test]
    fn xdev() {
        let mut config = Config::default();
        XDev::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
            .unwrap()
            .take_effect(&mut config)
            .unwrap();
        assert!(config.global_option.xdev);
    }
}
