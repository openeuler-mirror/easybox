//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
// This mod impls tests in find command.

use crate::find_common::get_metadata;
use crate::this_filter_built_with_config;
use crate::this_filter_consume_no_args;
use crate::this_filter_is_based_on_metadata;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use glob::MatchOptions;
use nix::unistd::{access, AccessFlags};
use once_cell::sync::OnceCell;
use users::get_group_by_gid;
use users::get_user_by_uid;
use uucore::error::USimpleError;

use self::time_type::DateString;

use super::metadata::FindMetadata;
use super::Config;
use super::FindConstruct;
use super::LinkMode;
use super::RegexType;
use super::{FindFile, FindFilter};
use std::collections::HashMap;
use std::fs::read_link;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::{cmp::Ordering, fmt::Debug, marker::PhantomData, str::FromStr, time::UNIX_EPOCH};

use uucore::error::UResult;

///
pub fn filesystem_id_map() -> UResult<&'static HashMap<u64, String>> {
    static FILESYSTEM_ID_MAP: OnceCell<HashMap<u64, String>> = OnceCell::new();
    FILESYSTEM_ID_MAP.get_or_try_init(|| {
        let proc_filesystems = BufReader::new(std::fs::File::open("/proc/filesystems")?);
        let mut filesystems = HashMap::new();

        for (id, line) in proc_filesystems.lines().enumerate() {
            if let Ok(line) = line {
                if !line.starts_with("nodev") {
                    filesystems.insert(id as u64, line.trim().to_string());
                }
            }
        }

        Ok(filesystems)
    })
}

///

pub fn get_filesystem_name(fs_id: u64) -> UResult<String> {
    filesystem_id_map().map(|map| map.get(&fs_id).cloned().unwrap_or("Unknown".to_string()))
}

#[derive(Debug)]
struct CmpHelper<T> {
    target: T,
    ordering: Ordering,
}

impl<T> CmpHelper<T> {
    pub fn new(target: T, ordering: Ordering) -> Self {
        Self { target, ordering }
    }
}

impl<T: FromStr> CmpHelper<T> {
    pub fn construct_from_iter<S: Into<String>>(
        iter: &mut impl Iterator<Item = S>,
    ) -> UResult<Self> {
        match iter.next() {
            None => Err(USimpleError::new(
                1,
                "Cannot build a compare filter because the lack of args",
            )),
            Some(arg) => {
                let arg: String = arg.into();
                let (ordering, n) = {
                    let (ordering, buf) = if let Some(n) = arg.strip_prefix('+') {
                        (Ordering::Greater, n)
                    } else if let Some(n) = arg.strip_prefix('-') {
                        (Ordering::Less, n)
                    } else {
                        (Ordering::Equal, &arg[..])
                    };
                    let n = buf.parse::<T>().map_err(|_| {
                        USimpleError::new(1, format!("Cannot parse {buf} as the filter argument"))
                    })?;
                    (ordering, n)
                };

                Ok(Self::new(n, ordering))
            }
        }
    }
}

impl<T: Ord> CmpHelper<T> {
    pub fn check(&self, value: T) -> bool {
        value.cmp(&self.target) == self.ordering
    }
}

#[derive(Debug)]
///
pub struct True;

impl True {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for True {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for True {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        Ok(true)
    }
}

impl FindConstruct for True {
    this_filter_consume_no_args!();
}

#[derive(Debug)]
///
pub struct False;

impl False {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl Default for False {
    fn default() -> Self {
        Self::new()
    }
}

impl FindFilter for False {
    fn filter(&mut self, _file: &FindFile) -> UResult<bool> {
        Ok(false)
    }
}

impl FindConstruct for False {
    this_filter_consume_no_args!();
}

///
pub const MIN: i64 = 60;

///
pub const HOUR: i64 = MIN * 60;

///
pub const DAY: i64 = HOUR * 24;

///
pub trait TimeType {
    ///
    fn get_time(metadata: &dyn FindMetadata) -> i64;
}

///
pub mod time_type {
    use uucore::error::USimpleError;

    use crate::find_common::metadata::FindMetadata;

    use super::TimeType;

    #[derive(Debug)]
    ///
    pub struct Access;
    #[derive(Debug)]
    ///
    pub struct Change;
    #[derive(Debug)]
    ///
    pub struct Modify;

    impl TimeType for Access {
        fn get_time(metadata: &dyn FindMetadata) -> i64 {
            metadata.st_atime()
        }
    }

    impl TimeType for Change {
        fn get_time(metadata: &dyn FindMetadata) -> i64 {
            metadata.st_ctime()
        }
    }

    impl TimeType for Modify {
        fn get_time(metadata: &dyn FindMetadata) -> i64 {
            metadata.st_mtime()
        }
    }

    #[derive(Debug)]
    ///
    pub struct Birth;

    #[derive(Debug)]
    ///
    pub struct DateString {
        timestamp: i64,
    }

    impl DateString {
        ///
        pub fn create(arg: &str) -> uucore::error::UResult<Self> {
            let datetime =
                dateparser::parse(arg).map_err(|e| USimpleError::new(1, e.to_string()))?;

            let timestamp = datetime.timestamp();

            Ok(Self { timestamp })
        }

        ///
        pub fn get_time(&self) -> i64 {
            self.timestamp
        }
    }
}

/// Get the timestamp of now or the start of the day, depending on the config `daystart` option.
fn get_now_timestamp(config: &Config) -> i64 {
    let now = std::time::SystemTime::now();
    let now = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    if config.filter_option.daystart {
        get_daystart(now)
    } else {
        now
    }
}

fn get_daystart(time: i64) -> i64 {
    let datetime = DateTime::from_timestamp(time, 0).expect("Invalid timestamp");
    let local_datetime = Local.from_utc_datetime(&datetime.naive_utc());
    let midnight = local_datetime.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let midnight = Local.from_local_datetime(&midnight).unwrap();

    // BUG: https://savannah.gnu.org/bugs/index.php?printer=1&func=detailitem&item_id=23065
    // -daystart measures from start of tomorrow.
    // To mimic this bug, uncomment the following line.
    //
    // let midnight = midnight.checked_add_days(Days::new(1)).unwrap();

    midnight.timestamp()
}

///
pub type AccessMin = DurationToNow<time_type::Access, MIN>;

///
pub type ChangeMin = DurationToNow<time_type::Change, MIN>;

///
pub type ModifyMin = DurationToNow<time_type::Modify, MIN>;

///
pub type AccessTime = DurationToNow<time_type::Access, DAY>;

///
pub type ChangeTime = DurationToNow<time_type::Change, DAY>;

///
pub type ModifyTime = DurationToNow<time_type::Modify, DAY>;

#[derive(Debug)]
///
pub struct DurationToNow<T: TimeType + Debug, const UNIT: i64> {
    inner: CmpHelper<i64>,
    now: i64,
    follow_link: bool,
    _time_type: PhantomData<T>,
}

impl<T: TimeType + Debug, const UNIT: i64> DurationToNow<T, UNIT> {
    ///
    pub fn new(target: i64, ordering: Ordering, config: &Config) -> Self {
        Self {
            inner: CmpHelper::new(target, ordering),
            now: get_now_timestamp(config),
            follow_link: is_follow_link_enabled_when_filter(config),
            _time_type: PhantomData,
        }
    }
}

impl<T: TimeType + Debug, const UNIT: i64> FindFilter for DurationToNow<T, UNIT> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let time = T::get_time(m);
            let duration = (self.now - time).max(0) / UNIT;
            self.inner.check(duration)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl<T: TimeType + Debug, const UNIT: i64> FindConstruct for DurationToNow<T, UNIT> {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        CmpHelper::construct_from_iter(iter).map(|inner| Self {
            inner,
            now: get_now_timestamp(config),
            follow_link: is_follow_link_enabled_when_filter(config),
            _time_type: PhantomData,
        })
    }
}

#[derive(Debug)]
///
pub struct NewerXY<X, Y> {
    inner: CmpHelper<i64>,
    follow_link: bool,
    _px: PhantomData<X>,
    _py: PhantomData<Y>,
}

impl<X, Y: TimeType> NewerXY<X, Y> {
    ///
    pub fn new(path: &str, config: &Config) -> UResult<Self> {
        let follow_link = is_follow_link_enabled_when_build(config);
        let file = FindFile::new(path, "/", 0, config.debug_stat);
        let metadata = get_metadata(&file, follow_link)?;

        let target = Y::get_time(metadata);
        Ok(Self {
            inner: CmpHelper::new(target, Ordering::Greater),
            follow_link: is_follow_link_enabled_when_filter(config),
            _px: PhantomData,
            _py: PhantomData,
        })
    }
}

impl<X> NewerXY<X, DateString> {
    ///
    pub fn new(date: &str, config: &Config) -> UResult<Self> {
        let target = DateString::create(date)?.get_time();

        Ok(Self {
            inner: CmpHelper::new(target, Ordering::Greater),
            follow_link: is_follow_link_enabled_when_filter(config),
            _px: PhantomData,
            _py: PhantomData,
        })
    }
}

impl<X: TimeType + Debug, Y: TimeType + Debug> FindFilter for NewerXY<X, Y> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let time = X::get_time(m);
            self.inner.check(time)
        })
    }
}

impl<X: TimeType + Debug, Y: TimeType + Debug> FindConstruct for NewerXY<X, Y> {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "Lack an arg for newerxy"))
            .and_then(|arg| Self::new(&arg, config))
    }
}

impl<X: TimeType + Debug> FindFilter for NewerXY<X, DateString> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let time = X::get_time(m);
            self.inner.check(time)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl<X: TimeType + Debug> FindConstruct for NewerXY<X, DateString> {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "Lack an arg for newerxy"))
            .and_then(|arg| Self::new(&arg, config))
    }
}

///
pub type AccessNewer = NewerXY<time_type::Access, time_type::Access>;

///
pub type ModifyNewer = NewerXY<time_type::Modify, time_type::Modify>;

///
pub type ChangeNewer = NewerXY<time_type::Change, time_type::Change>;

#[derive(Debug)]
///
pub struct Used {
    inner: CmpHelper<i64>,
    follow_link: bool,
}

impl Used {
    ///
    pub fn new(target: i64, ordering: Ordering, config: &Config) -> Self {
        Self {
            inner: CmpHelper::new(target, ordering),
            follow_link: is_follow_link_enabled_when_filter(config),
        }
    }
}

impl FindFilter for Used {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let access_time = time_type::Access::get_time(m);
            let change_time = time_type::Change::get_time(m);

            // According to used.sh in gnu findutils, files with the same atime and ctime cannot
            // pass `-used 0`.
            if access_time == change_time
                && self.inner.target == 0
                && self.inner.ordering == Ordering::Equal
            {
                return false;
            }

            let duration = access_time - change_time;
            let days = duration / DAY;
            self.inner.check(days)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Used {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        CmpHelper::construct_from_iter(iter).map(|inner| Self {
            inner,
            follow_link: is_follow_link_enabled_when_filter(config),
        })
    }
}

#[derive(Debug)]
///
pub struct Empty {
    follow_link: bool,
}

impl Empty {
    ///
    pub fn new(config: &Config) -> Self {
        Self {
            follow_link: is_follow_link_enabled_when_filter(config),
        }
    }
}

impl FindFilter for Empty {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| m.st_len() == 0)
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Empty {
    this_filter_built_with_config!();
}

#[derive(Debug)]
///
pub struct FileSystemType {
    fs: String,
    follow_link: bool,
}

impl FileSystemType {
    ///
    pub fn new(fs: &str, config: &Config) -> Self {
        Self {
            fs: fs.to_string(),
            follow_link: is_follow_link_enabled_when_filter(config),
        }
    }
}

impl FindFilter for FileSystemType {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).and_then(|m| {
            let dev = m.st_dev();
            let fstype = get_filesystem_name(dev)?;
            Ok(fstype == self.fs)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for FileSystemType {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .map(|arg| Self::new(&arg, config))
            .ok_or(USimpleError::new(1, "Lack an arg for -fstype"))
    }
}

#[derive(Debug)]
///
pub struct GroupId {
    inner: CmpHelper<u32>,
    follow_link: bool,
}

impl GroupId {
    ///
    pub fn new(gid: u32, ordering: Ordering, config: &Config) -> Self {
        Self {
            inner: CmpHelper::new(gid, ordering),
            follow_link: is_follow_link_enabled_when_filter(config),
        }
    }
}

impl FindFilter for GroupId {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let group = m.st_gid();
            self.inner.check(group)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for GroupId {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        CmpHelper::construct_from_iter(iter).map(|inner| Self {
            inner,
            follow_link: is_follow_link_enabled_when_filter(config),
        })
    }
}

#[derive(Debug)]
///
pub struct LinkedName {
    pattern: glob::Pattern,
    follow_link: bool,
}

impl LinkedName {
    ///
    pub fn new(pattern: &str, config: &Config) -> UResult<Self> {
        glob::Pattern::new(pattern)
            .map_err(|e| {
                USimpleError::new(1, format!("Cannot build pattern {pattern} for lname: {e}"))
            })
            .map(|pattern| Self {
                pattern,
                follow_link: is_follow_link_enabled_when_filter(config),
            })
    }
}

impl FindFilter for LinkedName {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        if self.follow_link {
            return Ok(false);
        }
        let path = file.get_path();
        let pointee_path = read_link(path)?;
        let pointee_path = pointee_path.to_string_lossy();

        Ok(self.pattern.matches(&pointee_path))
    }
}

impl FindConstruct for LinkedName {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for lname"))
            .and_then(|arg| Self::new(&arg, config))
    }
}

#[derive(Debug)]
///
pub struct InsensitiveLinkedName {
    pattern: glob::Pattern,
    follow_link: bool,
}

impl InsensitiveLinkedName {
    ///
    pub fn new(pattern: &str, config: &Config) -> UResult<Self> {
        glob::Pattern::new(pattern)
            .map_err(|e| {
                USimpleError::new(1, format!("Cannot build pattern {pattern} for lname: {e}"))
            })
            .map(|pattern| Self {
                pattern,
                follow_link: is_follow_link_enabled_when_filter(config),
            })
    }
}

impl FindFilter for InsensitiveLinkedName {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        if self.follow_link {
            return Ok(false);
        }
        let path = file.get_path();
        let pointee_path = read_link(path)?;
        let pointee_path = pointee_path.to_string_lossy();

        Ok(self.pattern.matches_with(
            &pointee_path,
            MatchOptions {
                case_sensitive: false,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            },
        ))
    }
}

impl FindConstruct for InsensitiveLinkedName {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for lname"))
            .and_then(|arg| Self::new(&arg, config))
    }
}

#[derive(Debug)]
///
pub struct Name {
    pattern: glob::Pattern,
}

impl Name {
    ///
    pub fn new(pattern: &str) -> UResult<Self> {
        Ok(Self {
            pattern: glob::Pattern::new(pattern)
                .map_err(|e| USimpleError::new(1, e.to_string()))?,
        })
    }

    fn matches(&self, name: &str) -> bool {
        self.pattern.matches(name)
    }

    fn matches_with(&self, name: &str, options: MatchOptions) -> bool {
        self.pattern.matches_with(name, options)
    }
}

impl FindFilter for Name {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        if self.pattern.as_str() == "/" && file.get_path().as_os_str() == "/" {
            return Ok(true);
        }

        let name = file.get_path().file_name();
        let name = name.map(|n| n.to_string_lossy());
        if name.is_none() {
            return Ok(false);
        }

        let name = name.unwrap();
        Ok(self.matches(&name))
    }
}

impl FindConstruct for Name {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        match iter.next() {
            None => Err(USimpleError::new(
                1,
                "Cannot build the filter because of lack of the args",
            )),
            Some(pattern) => Ok(Self::new(&pattern)?),
        }
    }
}

#[derive(Debug)]
///
pub struct InsensitiveName {
    inner: Name,
}

impl InsensitiveName {
    ///
    pub fn new(pattern: &str) -> UResult<Self> {
        Name::new(pattern).map(|inner| Self { inner })
    }
}

impl FindFilter for InsensitiveName {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let name = file.get_path().file_name();
        let name = name.map(|n| n.to_string_lossy());
        if name.is_none() {
            return Ok(false);
        }

        let name = name.unwrap();
        Ok(self.inner.matches_with(
            &name,
            MatchOptions {
                case_sensitive: false,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            },
        ))
    }
}

impl FindConstruct for InsensitiveName {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        match iter.next() {
            None => Err(USimpleError::new(
                1,
                "Cannot build the filter because of lack of the args",
            )),
            Some(pattern) => Ok(Self::new(&pattern)?),
        }
    }
}

#[derive(Debug)]
///
pub struct Inode {
    inner: CmpHelper<u64>,
    follow_link: bool,
}

impl Inode {
    ///
    pub fn new(inode: u64, ordering: Ordering, follow_link: bool) -> Self {
        Self {
            inner: CmpHelper::new(inode, ordering),
            follow_link,
        }
    }
}

impl FindFilter for Inode {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let inode = m.st_ino();
            self.inner.check(inode)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Inode {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        let follow_link = is_follow_link_enabled_when_filter(config);
        CmpHelper::construct_from_iter(iter).map(|inner| Self { inner, follow_link })
    }
}

#[derive(Debug)]
///
pub struct FilterPath {
    pattern: glob::Pattern,
}

impl FilterPath {
    ///
    pub fn new(pattern: &str) -> UResult<Self> {
        Ok(Self {
            pattern: glob::Pattern::new(pattern)
                .map_err(|e| USimpleError::new(1, e.to_string()))?,
        })
    }

    fn matches(&self, path: &Path) -> bool {
        self.pattern.matches_path(path)
    }

    fn matches_with(&self, path: &Path, options: MatchOptions) -> bool {
        self.pattern.matches_path_with(path, options)
    }
}

impl FindFilter for FilterPath {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let path = file.get_path();
        Ok(self.matches(path))
    }
}

impl FindConstruct for FilterPath {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        match iter.next() {
            None => Err(USimpleError::new(
                1,
                "Cannot build the filter because of lack of the args",
            )),
            Some(pattern) => Ok(Self::new(&pattern)?),
        }
    }
}

#[derive(Debug)]
///
pub struct InsensitivePath {
    inner: FilterPath,
}

impl InsensitivePath {
    ///
    pub fn new(pattern: &str) -> UResult<Self> {
        FilterPath::new(pattern).map(|inner| Self { inner })
    }
}

impl FindFilter for InsensitivePath {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let path = file.get_path();
        Ok(self.inner.matches_with(
            path,
            MatchOptions {
                case_sensitive: false,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            },
        ))
    }
}

impl FindConstruct for InsensitivePath {
    fn construct_from_iter(iter: &mut impl Iterator<Item = String>) -> UResult<Self> {
        match iter.next() {
            None => Err(USimpleError::new(
                1,
                "Cannot build the filter because of lack of the args",
            )),
            Some(pattern) => Ok(Self::new(&pattern)?),
        }
    }
}

///
pub type WholeName = FilterPath;

///
pub type InsensitiveWholeName = InsensitivePath;

#[derive(Debug)]
///
pub struct Regex {
    re: regex::Regex,
}

impl Regex {
    ///
    pub fn new(pattern: &str, regex_type: RegexType, case_insensitive: bool) -> UResult<Self> {
        let re = regex_type.create_re(pattern, case_insensitive)?;

        Ok(Self { re })
    }

    ///
    pub fn matches(&self, file_name: &str) -> bool {
        self.re.is_match(file_name)
    }
}

impl FindFilter for Regex {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let path = file.get_path();
        let name = path
            .file_name()
            .ok_or(USimpleError::new(
                1,
                format!("Cannot get the file name of {}", path.to_string_lossy()),
            ))?
            .to_string_lossy();

        Ok(self.matches(&name))
    }
}

impl FindConstruct for Regex {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for regex"))
            .and_then(|arg| Self::new(&arg, config.filter_option.regex_type, false))
    }
}

#[derive(Debug)]
///
pub struct InsensitiveRegex {
    inner: Regex,
}

impl InsensitiveRegex {
    ///
    pub fn new(pattern: &str, regex_type: RegexType) -> UResult<Self> {
        let inner = Regex::new(pattern, regex_type, true)?;
        Ok(Self { inner })
    }

    ///
    pub fn matches(&self, file_name: &str) -> bool {
        self.inner.matches(file_name)
    }
}

impl FindFilter for InsensitiveRegex {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let path = file.get_path();
        let name = path
            .file_name()
            .ok_or(USimpleError::new(
                1,
                format!("Cannot get the file name of {}", path.to_string_lossy()),
            ))?
            .to_string_lossy();

        Ok(self.matches(&name))
    }
}

impl FindConstruct for InsensitiveRegex {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for iregex"))
            .and_then(|arg| Self::new(&arg, config.filter_option.regex_type))
    }
}

#[derive(Debug)]
///
pub struct HardLinkCount {
    inner: CmpHelper<u64>,
    follow_link: bool,
}

impl HardLinkCount {
    ///
    pub fn new(hard_link_count: u64, ordering: Ordering, follow_link: bool) -> Self {
        Self {
            inner: CmpHelper::new(hard_link_count, ordering),
            follow_link,
        }
    }
}

impl FindFilter for HardLinkCount {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let inode = m.st_ino();
            self.inner.check(inode)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for HardLinkCount {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        let follow_link = is_follow_link_enabled_when_filter(config);
        CmpHelper::construct_from_iter(iter).map(|inner| Self { inner, follow_link })
    }
}

#[derive(Debug)]
///
pub struct NoGroup {
    follow_link: bool,
}

impl NoGroup {
    ///
    pub fn new(config: &Config) -> Self {
        Self {
            follow_link: is_follow_link_enabled_when_filter(config),
        }
    }
}

impl FindFilter for NoGroup {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let group = m.st_gid();
            get_group_by_gid(group).is_none()
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for NoGroup {
    this_filter_built_with_config!();
}

#[derive(Debug)]
///
pub struct NoUser {
    follow_link: bool,
}

impl NoUser {
    ///
    pub fn new(config: &Config) -> Self {
        Self {
            follow_link: is_follow_link_enabled_when_filter(config),
        }
    }
}

impl FindFilter for NoUser {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let user = m.st_uid();
            get_user_by_uid(user).is_none()
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for NoUser {
    this_filter_built_with_config!();
}

#[derive(Debug, Clone, Copy)]
///
pub enum PermMaskType {
    ///
    Exact,

    ///
    All,

    ///
    Any,
}

#[derive(Debug)]
///
pub struct Perm {
    perm: u32,
    mask_type: PermMaskType,
    follow_link: bool,
}

impl Perm {
    ///
    pub fn new(perm: u32, mask_type: PermMaskType, follow_link: bool) -> Self {
        Self {
            perm,
            mask_type,
            follow_link,
        }
    }
}

const PERM_BITS: u32 = 0b111_111_111;

impl FindFilter for Perm {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let mode = m.st_mode() & PERM_BITS;
            match self.mask_type {
                PermMaskType::Exact => mode == self.perm,
                PermMaskType::All => mode & self.perm == self.perm,
                PermMaskType::Any => (mode & self.perm != 0) || (self.perm == 0),
            }
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Perm {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        match iter.next() {
            None => Err(USimpleError::new(
                1,
                "Cannot build the filter because of the lack of the args",
            )),
            Some(arg) => {
                let (mask_type, buf) = if let Some(buf) = arg.strip_prefix('-') {
                    (PermMaskType::All, buf)
                } else if let Some(buf) = arg.strip_prefix('/') {
                    (PermMaskType::Any, buf)
                } else if let Some(buf) = arg.strip_prefix('+') {
                    (PermMaskType::Any, buf)
                } else {
                    (PermMaskType::Exact, &*arg)
                };

                let perm = string_to_mode(buf)?;
                Ok(Self::new(
                    perm,
                    mask_type,
                    is_follow_link_enabled_when_filter(config),
                ))
            }
        }
    }
}

fn string_to_mode(mode: &str) -> UResult<u32> {
    if mode.is_empty() {
        return Err(USimpleError::new(1, "The mode string is empty"));
    }

    let symbolic_re = regex::Regex::new(r"([ugoa]=[rwx]+,)*([ugoa]=[rwx]+)").unwrap();
    if symbolic_re.is_match(mode) {
        parse_mode_string(mode)
    } else {
        parse_octal_mode(mode)
    }
}

fn parse_octal_mode(octal: &str) -> UResult<u32> {
    u32::from_str_radix(octal, 8)
        .map_err(|_e| USimpleError::new(1, format!("Cannot parse the octal mode string: {octal}")))
}

fn parse_mode_string(expr: &str) -> UResult<u32> {
    let mode_re = regex::Regex::new(r"(u|g|o|a)=([rwx]+)").unwrap();
    let mut mode = 0;

    for capture in mode_re.captures_iter(expr) {
        let entity = &capture[1];
        let perms = &capture[2];

        let entity_offset = match entity.chars().next().unwrap() {
            'u' => 6,
            'g' => 3,
            'o' | 'a' => 0,
            _ => unreachable!(),
        };

        for perm in perms.chars() {
            let perm_offset = match perm {
                'r' => 2,
                'w' => 1,
                'x' => 0,
                _ => unreachable!(),
            };

            mode |= 1 << (entity_offset + perm_offset);
        }
    }

    Ok(mode)
}

#[derive(Debug)]
///
pub struct Accessibility<const MODE_BITS: i32>;

impl<const MODE_BITS: i32> Accessibility<MODE_BITS> {
    ///
    pub fn new() -> Self {
        Self
    }
}

impl<const MODE_BITS: i32> Default for Accessibility<MODE_BITS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MODE_BITS: i32> FindFilter for Accessibility<MODE_BITS> {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let cflag: libc::c_int = MODE_BITS;
        let access_flag = AccessFlags::from_bits_truncate(cflag);
        let path = file.get_path();
        Ok(access(path, access_flag).is_ok())
    }

    this_filter_is_based_on_metadata!();
}

impl<const MODE_BITS: i32> FindConstruct for Accessibility<MODE_BITS> {
    this_filter_consume_no_args!();
}

///
pub const READ: i32 = 4;

///
pub const WRITE: i32 = 2;

///
pub const EXECUTE: i32 = 1;

///
pub type Readable = Accessibility<READ>;

///
pub type Writable = Accessibility<WRITE>;

///
pub type Executable = Accessibility<EXECUTE>;

#[derive(Debug)]
///
pub struct FileSize {
    count: u64,
    unit: u64,
}

impl FileSize {
    ///
    pub fn new(count: u64, unit: u64) -> Self {
        Self { count, unit }
    }
}

/// Size filter need to round up the size to the next unit, So the size comparison is not
/// TOTAL-ORDERED!
#[derive(Debug)]
pub struct Size {
    target: FileSize,
    ordering: Ordering,
    follow_link: bool,
}

impl Size {
    ///
    pub fn new(count: u64, unit: u64, ordering: Ordering, follow_link: bool) -> Self {
        Self {
            target: FileSize::new(count, unit),
            ordering,
            follow_link,
        }
    }
}

impl FindFilter for Size {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let size = m.st_len();
            let unit = self.target.unit;
            let count = (size + unit - 1) / unit;
            count.cmp(&self.target.count) == self.ordering
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Size {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .and_then(|arg| {
                let (ordering, buf) = if let Some(buf) = arg.strip_prefix('+') {
                    (Ordering::Greater, buf)
                } else if let Some(buf) = arg.strip_prefix('-') {
                    (Ordering::Less, buf)
                } else {
                    (Ordering::Equal, &*arg)
                };

                let (unit, buf) = if let Some(buf) = buf.strip_suffix('c') {
                    (1, buf)
                } else if let Some(buf) = buf.strip_suffix('w') {
                    (2, buf)
                } else if let Some(buf) = buf.strip_suffix('k') {
                    (1 << 10, buf)
                } else if let Some(buf) = buf.strip_suffix('M') {
                    (1 << 20, buf)
                } else if let Some(buf) = buf.strip_suffix('G') {
                    (1 << 30, buf)
                } else if let Some(buf) = buf.strip_suffix('b') {
                    (512, buf)
                } else {
                    (512, buf)
                };

                buf.parse::<u64>().ok().map(|count| {
                    Self::new(
                        count,
                        unit,
                        ordering,
                        is_follow_link_enabled_when_filter(config),
                    )
                })
            })
            .ok_or(USimpleError::new(
                1,
                "Cannot build the file-size filter: invalid size",
            ))
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]

///
pub enum FindFileType {
    ///
    Block,

    ///
    Char,

    ///
    Directory,

    ///
    Pipe,

    ///
    Regular,

    ///
    Link, // This will never be true when -L or -follow enabled, unless the link is broken,

    ///
    Socket,

    ///
    Door, // Only for solaris
}

impl TryFrom<char> for FindFileType {
    type Error = USimpleError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'b' => Ok(Self::Block),
            'c' => Ok(Self::Char),
            'd' => Ok(Self::Directory),
            'p' => Ok(Self::Pipe),
            'f' => Ok(Self::Regular),
            'l' => Ok(Self::Link),
            's' => Ok(Self::Socket),
            'D' => Ok(Self::Door),
            _ => Err(USimpleError {
                code: 1,
                message: format!("No such file type: {value}"),
            }),
        }
    }
}

impl From<FindFileType> for char {
    fn from(val: FindFileType) -> Self {
        match val {
            FindFileType::Block => 'b',
            FindFileType::Char => 'c',
            FindFileType::Directory => 'd',
            FindFileType::Pipe => 'p',
            FindFileType::Regular => '-',
            FindFileType::Link => 'l',
            FindFileType::Socket => 's',
            FindFileType::Door => 'D',
        }
    }
}

impl FindFileType {
    ///
    pub fn from_mode_bits(mode: u32) -> UResult<Self> {
        match mode & libc::S_IFMT {
            libc::S_IFREG => Ok(Self::Regular),
            libc::S_IFDIR => Ok(Self::Directory),
            libc::S_IFLNK => Ok(Self::Link),
            libc::S_IFBLK => Ok(Self::Block),
            libc::S_IFCHR => Ok(Self::Char),
            libc::S_IFIFO => Ok(Self::Pipe),
            libc::S_IFSOCK => Ok(Self::Socket),
            _ => Err(USimpleError::new(1, "Unknown file type bits")),
        }
    }

    ///
    pub fn to_mode_bits(&self) -> UResult<u32> {
        Ok(match self {
            FindFileType::Block => libc::S_IFBLK,
            FindFileType::Char => libc::S_IFCHR,
            FindFileType::Directory => libc::S_IFDIR,
            FindFileType::Pipe => libc::S_IFIFO,
            FindFileType::Regular => libc::S_IFREG,
            FindFileType::Link => libc::S_IFLNK,
            FindFileType::Socket => libc::S_IFSOCK,
            FindFileType::Door => Err(USimpleError::new(1, "No mode bits for Door"))?,
        })
    }
}

#[derive(Debug)]
///
pub struct Type {
    file_type: FindFileType,
    follow_link: bool,
}

impl Type {
    ///
    pub fn new(file_type: char, config: &Config) -> UResult<Self> {
        Ok(file_type.try_into().map(|file_type| Self {
            file_type,
            follow_link: is_follow_link_enabled_when_filter(config),
        })?)
    }
}

fn is_symlink_broken(path: &Path) -> bool {
    if let Ok(target_path) = std::fs::read_link(path) {
        if target_path.is_absolute() {
            !target_path.exists()
        } else {
            let mut path = path.to_path_buf();
            path.pop();
            path.push(target_path);
            !path.exists()
        }
    } else {
        true
    }
}

fn is_symlink(file: &FindFile) -> UResult<bool> {
    Ok(FindFileType::from_mode_bits(file.get_metadata()?.st_mode())? == FindFileType::Link)
}

impl FindFilter for Type {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        let ftype = if is_symlink(file)? && self.follow_link && is_symlink_broken(file.get_path()) {
            FindFileType::Link
        } else {
            get_metadata(file, self.follow_link)
                .and_then(|m| FindFileType::from_mode_bits(m.st_mode()))?
        };

        Ok(ftype == self.file_type)
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Type {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -type"))
            .and_then(|arg| {
                if arg.len() != 1 {
                    return Err(USimpleError::new(
                        1,
                        format!("`{arg}` is not a valid file type for -type"),
                    ));
                }
                let c = arg.chars().next().unwrap();
                Self::new(c, config)
            })
    }
}

#[derive(Debug)]
///
pub struct XType {
    inner: Type,
}

impl XType {
    ///
    pub fn new(file_type: char, config: &Config) -> UResult<Self> {
        Ok(FindFileType::try_from(file_type)
            .map(|file_type| Type {
                file_type,
                follow_link: !is_follow_link_enabled_when_filter(config),
            })
            .map(|inner| Self { inner })?)
    }
}

// BUG: Need testing
impl FindFilter for XType {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        self.inner.filter(file)
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for XType {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -xtype"))
            .and_then(|arg| {
                if arg.len() != 1 {
                    return Err(USimpleError::new(
                        1,
                        format!("`{arg}` is not a valid file type for -xtype"),
                    ));
                }
                let c = arg.chars().next().unwrap();
                Self::new(c, config)
            })
    }
}

#[derive(Debug)]
///
pub struct UserId {
    inner: CmpHelper<u32>,
    follow_link: bool,
}

impl UserId {
    ///
    pub fn new(user_id: u32, ordering: Ordering, follow_link: bool) -> Self {
        Self {
            inner: CmpHelper::new(user_id, ordering),
            follow_link,
        }
    }
}

impl FindFilter for UserId {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let user_id = m.st_uid();
            self.inner.check(user_id)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for UserId {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        CmpHelper::construct_from_iter(iter).map(|inner| Self {
            inner,
            follow_link: is_follow_link_enabled_when_filter(config),
        })
    }
}

#[derive(Debug)]
///
pub enum Either<L, R> {
    ///
    L(L),

    ///
    R(R),
}

impl<L: FromStr, R: FromStr> FromStr for Either<L, R> {
    type Err = USimpleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<L>()
            .map(Either::L)
            .or_else(|_| s.parse::<R>().map(Either::R))
            .map_err(|_| USimpleError {
                code: 1,
                message: format!("Cannot build Either from {}", s),
            })
    }
}

#[derive(Debug)]
///
pub struct User {
    user: Either<u32, String>,
    follow_link: bool,
}

impl User {
    ///
    pub fn new(user: Either<u32, String>, follow_link: bool) -> Self {
        Self { user, follow_link }
    }
}

impl FindFilter for User {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let id = m.st_uid();
            match self.user {
                Either::L(uid) => id == uid,
                Either::R(ref uname) => m.st_uname().map(|u| u == *uname).unwrap_or(false),
            }
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for User {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -user"))
            .and_then(|arg| {
                let user = arg
                    .parse::<Either<_, _>>()
                    .map_err(|_| USimpleError::new(1, "`{}` is not valid for -user"))?;
                Ok(Self::new(user, is_follow_link_enabled_when_filter(config)))
            })
    }
}

#[derive(Debug)]
///
pub struct Group {
    group: Either<u32, String>,
    follow_link: bool,
}

impl Group {
    ///
    pub fn new(group: Either<u32, String>, follow_link: bool) -> Self {
        Self { group, follow_link }
    }
}

impl FindFilter for Group {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let id = m.st_gid();
            match self.group {
                Either::L(gid) => id == gid,
                Either::R(ref gname) => m.st_gname().map(|g| g == *gname).unwrap_or(false),
            }
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for Group {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -group"))
            .and_then(|arg| {
                let user = arg
                    .parse::<Either<_, _>>()
                    .map_err(|_| USimpleError::new(1, "`{}` is not valid for -user"))?;
                Ok(Self::new(user, is_follow_link_enabled_when_filter(config)))
            })
    }
}

// #[cfg(feature = "selinux")]
// #[derive(Debug)]
// pub struct SELinuxContext {
//     pattern: glob::Pattern,
//     follow_link: bool,
// }
//
// #[cfg(feature = "selinux")]
// impl SELinuxContext {
//     pub fn new(pattern: &str, follow_link: bool) -> UResult<Self> {
//         Ok(Self {
//             pattern: glob::Pattern::new(pattern)
//                 .map_err(|e| USimpleError::new(1, e.to_string()))?,
//             follow_link,
//         })
//     }
// }
//
// #[cfg(feature = "selinux")]
// impl FindFilter for SELinuxContext {
//     fn filter(&mut self, file: &FindFile) -> UResult<bool> {
//         let path = file.get_path();
//         let ctx = selinux::SecurityContext::of_path(path, self.follow_link, false)
//             .map_err(|e| USimpleError::new(1, e.to_string()))?;
//
//         if let Some(ctx) = ctx {
//             let ctx_str = ctx
//                 .to_c_string()
//                 .map_err(|e| USimpleError::new(1, e.to_string()))?;
//             if ctx_str.is_none() {
//                 return Ok(false);
//             }
//             let ctx_str = ctx_str.unwrap();
//             Ok(self.pattern.matches(&ctx_str.clone().to_string_lossy()))
//         } else {
//             Ok(false)
//         }
//     }
//
//     this_filter_is_based_on_metadata!();
// }
//
// #[cfg(feature = "selinux")]
// impl FindConstruct for SELinuxContext {
//     fn construct_from_iter_with_config(
//         iter: &mut impl Iterator<Item = String>,
//         config: &super::Config,
//     ) -> UResult<Self> {
//         if let Some(arg) = iter.next() {
//             Self::new(&arg, is_follow_link_enabled_when_filter(config))
//         } else {
//             Err(USimpleError::new(1, "No pattern for SELinuxContext filter"))
//         }
//     }
// }

#[derive(Debug)]
///
pub struct SameFile {
    inner: CmpHelper<u64>,
    follow_link: bool,
}

impl SameFile {
    ///
    pub fn new(reference: &str, config: &Config) -> UResult<Self> {
        let reference_metadata = if is_follow_link_enabled_when_build(config) {
            std::fs::metadata(reference)?
        } else {
            std::fs::symlink_metadata(reference)?
        };

        let inode = reference_metadata.st_ino();
        let inner = CmpHelper::new(inode, Ordering::Equal);

        Ok(Self {
            inner,
            follow_link: is_follow_link_enabled_when_filter(config),
        })
    }
}

impl FindFilter for SameFile {
    fn filter(&mut self, file: &FindFile) -> UResult<bool> {
        get_metadata(file, self.follow_link).map(|m| {
            let ino = m.st_ino();
            self.inner.check(ino)
        })
    }

    this_filter_is_based_on_metadata!();
}

impl FindConstruct for SameFile {
    fn construct_from_iter_with_config(
        iter: &mut impl Iterator<Item = String>,
        config: &Config,
    ) -> UResult<Self> {
        iter.next()
            .ok_or(USimpleError::new(1, "No arg for -samefile"))
            .and_then(|arg| Self::new(&arg, config))
    }
}

///
pub fn is_follow_link_enabled_when_filter(config: &Config) -> bool {
    config.link_mode == LinkMode::L || config.filter_option.follow_link
}

///
pub fn is_follow_link_enabled_when_build(config: &Config) -> bool {
    is_follow_link_enabled_when_filter(config) || config.link_mode == LinkMode::H
}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::{DateTime, Local};

    use crate::find_common::{
        tests::{
            time_type::{Access, DateString},
            Empty, FilterPath, Group, GroupId, InsensitiveName, InsensitivePath, InsensitiveRegex,
            Name, NewerXY, NoGroup, NoUser, Perm, Regex, Size, Type, User, UserId, XType,
        },
        Config, FindConstruct, FindFile, FindFilter,
    };

    use super::{DurationToNow, FindFileType, MIN};

    #[test]
    fn duration_to_now_less() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();
        let system_time = SystemTime::now();
        let duration_since_epoch = system_time.duration_since(UNIX_EPOCH).unwrap();
        let now = duration_since_epoch.as_secs() as i64;

        let mins = 30 * MIN;
        let time_ago = now - mins;
        let atime = time_ago + 2 * MIN;
        file.forge_metadata.atime = atime;
        assert!(
            DurationToNow::<Access, MIN>::construct_from_iter_with_config(
                &mut vec!["-30".to_string()].into_iter(),
                &config
            )
            .unwrap()
            .filter_with_side_effects(&file, &mut v)
            .unwrap()
        )
    }

    #[test]
    fn duration_to_now_more() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();
        let system_time = SystemTime::now();
        let duration_since_epoch = system_time.duration_since(UNIX_EPOCH).unwrap();
        let now = duration_since_epoch.as_secs() as i64;

        let mins = 30 * MIN;
        let time_ago = now - mins;
        let atime = time_ago - 2 * MIN;
        file.forge_metadata.atime = atime;
        assert!(
            DurationToNow::<Access, MIN>::construct_from_iter_with_config(
                &mut vec!["+30".to_string()].into_iter(),
                &config
            )
            .unwrap()
            .filter_with_side_effects(&file, &mut v)
            .unwrap()
        )
    }

    #[test]
    fn duration_to_now_equal() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();
        let system_time = SystemTime::now();
        let duration_since_epoch = system_time.duration_since(UNIX_EPOCH).unwrap();
        let now = duration_since_epoch.as_secs() as i64;

        let mins = 30 * MIN;
        let time_ago = now - mins;
        let atime = time_ago;
        file.forge_metadata.atime = atime;
        assert!(
            DurationToNow::<Access, MIN>::construct_from_iter_with_config(
                &mut vec!["30".to_string()].into_iter(),
                &config
            )
            .unwrap()
            .filter_with_side_effects(&file, &mut v)
            .unwrap()
        )
    }

    #[test]
    fn empty() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let args: Vec<String> = vec![];
        let config = Config::default();

        file.forge_metadata.len = 0;

        assert!(
            Empty::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        )
    }

    #[test]
    fn not_empty() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let args: Vec<String> = vec![];
        let config = Config::default();

        file.forge_metadata.len = 256;

        assert!(
            !Empty::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        )
    }

    // `-executable`, `-readable`, `-writable` depends on libc's `access` API.
    // No need to test them here.
    // #[test]
    // fn access() {
    //     let mut file = FindFile::new("/test", "/", 1);
    //     let mut v = vec![];
    //     let args: Vec<String> = vec![];
    //     let config = Config::default();
    //
    //     file.forge_metadata.mode = 1 << 8;
    //
    //     assert!(Executable::construct_from_iter_with_config(
    //         &mut args.clone().into_iter(),
    //         &config
    //     )
    //     .unwrap()
    //     .filter_with_side_effects(&file, &mut v)
    //     .unwrap());
    //
    //     let mut file = FindFile::new("/test", "/", 1);
    //     let mut v = vec![];
    //     file.forge_metadata.mode = 1 << 4;
    //
    //     assert!(Executable::construct_from_iter_with_config(
    //         &mut args.clone().into_iter(),
    //         &config
    //     )
    //     .unwrap()
    //     .filter_with_side_effects(&file, &mut v)
    //     .unwrap());
    //
    //     let mut file = FindFile::new("/test", "/", 1);
    //     let mut v = vec![];
    //     file.forge_metadata.mode = 1;
    //
    //     assert!(Executable::construct_from_iter_with_config(
    //         &mut args.clone().into_iter(),
    //         &config
    //     )
    //     .unwrap()
    //     .filter_with_side_effects(&file, &mut v)
    //     .unwrap());
    //
    //     let mut file = FindFile::new("/test", "/", 1);
    //     let mut v = vec![];
    //     file.forge_metadata.mode = 0;
    //
    //     assert!(!Executable::construct_from_iter_with_config(
    //         &mut args.clone().into_iter(),
    //         &config
    //     )
    //     .unwrap()
    //     .filter_with_side_effects(&file, &mut v)
    //     .unwrap());
    // }

    #[test]
    fn permission_exact() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["744".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.mode = 0o744;

        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o740;
        assert!(
            !Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o766;
        assert!(
            !Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn permission_any() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["/744".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.mode = 0o744;

        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o740;
        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o766;
        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn permission_all() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["-744".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.mode = 0o744;

        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o740;
        assert!(
            !Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o766;
        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.mode = 0o777;
        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn permission_parse() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["u=rx".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.mode = 0o500;

        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["u=r,g=r,o=x,u=wx".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.mode = 0o741;

        assert!(
            Perm::construct_from_iter_with_config(&mut args.clone().into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn gid() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["-1000".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.gid = 999;

        assert!(
            GroupId::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["1000".to_string()];
        file.forge_metadata.gid = 1000;
        assert!(
            GroupId::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["+1000".to_string()];
        file.forge_metadata.gid = 1001;
        assert!(
            GroupId::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn group() {
        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.gid = 1000;
        file.forge_metadata.gname = "finder".to_string();
        let mut v = vec![];
        let config = Config::default();

        let args = vec!["1000".to_string()];
        assert!(
            Group::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["finder".to_string()];
        assert!(
            Group::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn uid() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["-1000".to_string()];
        let mut v = vec![];
        let config = Config::default();
        file.forge_metadata.uid = 999;

        assert!(
            UserId::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["1000".to_string()];
        file.forge_metadata.uid = 1000;
        assert!(
            UserId::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        let args = vec!["+1000".to_string()];
        file.forge_metadata.uid = 1001;
        assert!(
            UserId::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn user() {
        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.uid = 1000;
        file.forge_metadata.uname = "finder".to_string();
        let mut v = vec![];
        let config = Config::default();

        let args = vec!["1000".to_string()];
        assert!(
            User::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["finder".to_string()];
        assert!(
            User::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn name() {
        let file = FindFile::new("/Open/Euler", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();

        let args = vec!["euler".to_string()];
        assert!(
            !Name::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["euler".to_string()];
        assert!(
            InsensitiveName::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["*ler".to_string()];
        assert!(
            Name::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["Open".to_string()];
        assert!(
            !Name::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn path() {
        let file = FindFile::new("/Open/Euler", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();

        let args = vec!["/open/euler".to_string()];
        assert!(
            !FilterPath::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["/open/euler".to_string()];
        assert!(
            InsensitivePath::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["*ler".to_string()];
        assert!(
            FilterPath::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["/Open/*".to_string()];
        assert!(
            FilterPath::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn newer() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();
        let system_time = SystemTime::now();
        let date_time = DateTime::<Local>::from(system_time);
        let date_string = date_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let timestamp = system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let arg = vec![date_string];

        file.forge_metadata.atime = timestamp + 60;
        assert!(
            NewerXY::<Access, DateString>::construct_from_iter_with_config(
                &mut arg.clone().into_iter(),
                &config
            )
            .unwrap()
            .filter_with_side_effects(&file, &mut v)
            .unwrap()
        );

        let mut file = FindFile::new("/test", "/", 1, false);
        file.forge_metadata.atime = timestamp - 60;
        assert!(
            !NewerXY::<Access, DateString>::construct_from_iter_with_config(
                &mut arg.clone().into_iter(),
                &config
            )
            .unwrap()
            .filter_with_side_effects(&file, &mut v)
            .unwrap()
        );
    }

    #[test]
    fn no_user_or_group() {
        let mut file = FindFile::new("/test", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();
        // This file must be owned by root.
        file.forge_metadata.uid = 0;
        file.forge_metadata.gid = 0;

        assert!(
            !NoUser::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        assert!(
            !NoGroup::construct_from_iter_with_config(&mut vec![].into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn nouser_or_group() {
        let file = FindFile::new("/Open/Euler", "/", 1, false);
        let mut v = vec![];
        let config = Config::default();

        let args = vec!["euler".to_string()];
        assert!(
            !Regex::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["euler".to_string()];
        assert!(
            InsensitiveRegex::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec![".*ler".to_string()];
        assert!(
            Regex::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );

        let args = vec!["Open|Euler".to_string()];
        assert!(
            Regex::construct_from_iter_with_config(&mut args.into_iter(), &config)
                .unwrap()
                .filter_with_side_effects(&file, &mut v)
                .unwrap()
        );
    }

    #[test]
    fn size() {
        let mut file = FindFile::new("/Open/Euler", "/", 1, false);
        file.forge_metadata.len = 1024 * 1024;
        let mut v = vec![];
        let config = Config::default();

        assert!(Size::construct_from_iter_with_config(
            &mut vec!["+1023k".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());

        assert!(!Size::construct_from_iter_with_config(
            &mut vec!["1023k".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());
    }

    #[test]
    fn size_round_up() {
        let mut file = FindFile::new("/Open/Euler", "/", 1, false);
        file.forge_metadata.len = 1024 + 1;
        let mut v = vec![];
        let config = Config::default();

        assert!(!Size::construct_from_iter_with_config(
            &mut vec!["1k".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());

        assert!(Size::construct_from_iter_with_config(
            &mut vec!["+1k".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());

        assert!(Size::construct_from_iter_with_config(
            &mut vec!["2k".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());
    }

    #[test]
    fn file_type() {
        let mut file = FindFile::new("/Open/Euler", "/", 1, false);
        file.forge_metadata.mode = FindFileType::Regular.to_mode_bits().unwrap();
        file.forge_symlink_metadata.mode = FindFileType::Regular.to_mode_bits().unwrap();
        let mut v = vec![];
        let config = Config::default();

        assert!(Type::construct_from_iter_with_config(
            &mut vec!["f".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());

        assert!(!Type::construct_from_iter_with_config(
            &mut vec!["d".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());

        assert!(XType::construct_from_iter_with_config(
            &mut vec!["f".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());

        assert!(!XType::construct_from_iter_with_config(
            &mut vec!["d".to_string()].into_iter(),
            &config
        )
        .unwrap()
        .filter_with_side_effects(&file, &mut v)
        .unwrap());
    }
}
