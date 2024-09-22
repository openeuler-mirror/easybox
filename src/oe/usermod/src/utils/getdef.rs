//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::{Mutex, Once};
use uucore::error::{UResult, USimpleError};

///
const ERROR_OPEN_FILE: i32 = 1;
///
const ERROR_READ_LINE: i32 = 2;
///
const ERROR_UNKNOWN_ITEM: i32 = 3;

lazy_static! {
    static ref DEF_TABLE: Mutex<HashMap<String, Option<String>>> = {
        let mut table = HashMap::new();
        let default_items = vec![
            "CHFN_RESTRICT",
            "CONSOLE_GROUPS",
            "CONSOLE",
            "CREATE_HOME",
            "DEFAULT_HOME",
            "ENCRYPT_METHOD",
            "ENV_PATH",
            "ENV_SUPATH",
            "ERASECHAR",
            "FAIL_DELAY",
            "FAKE_SHELL",
            "GID_MAX",
            "GID_MIN",
            "HOME_MODE",
            "HUSHLOGIN_FILE",
            "KILLCHAR",
            "LASTLOG_UID_MAX",
            "LOGIN_RETRIES",
            "LOGIN_TIMEOUT",
            "LOG_OK_LOGINS",
            "LOG_UNKFAIL_ENAB",
            "MAIL_DIR",
            "MAIL_FILE",
            "MAX_MEMBERS_PER_GROUP",
            "MD5_CRYPT_ENAB",
            "NONEXISTENT",
            "PASS_MAX_DAYS",
            "PASS_MIN_DAYS",
            "PASS_WARN_AGE",
            "SUB_GID_COUNT",
            "SUB_GID_MAX",
            "SUB_GID_MIN",
            "SUB_UID_COUNT",
            "SUB_UID_MAX",
            "SUB_UID_MIN",
            "SULOG_FILE",
            "SU_NAME",
            "SYS_GID_MAX",
            "SYS_GID_MIN",
            "SYS_UID_MAX",
            "SYS_UID_MIN",
            "TTYGROUP",
            "TTYPERM",
            "TTYTYPE_FILE",
            "UID_MAX",
            "UID_MIN",
            "UMASK",
            "USERDEL_CMD",
            "USERGROUPS_ENAB",
            "SYSLOG_SG_ENAB",
            "SYSLOG_SU_ENAB",
            "FORCE_SHADOW",
            "GRANT_AUX_GROUP_SUBIDS",
            "PREVENT_NO_AUTH",
        ];
        for &item in default_items.iter() {
            table.insert(item.to_string(), None);
        }
        Mutex::new(table)
    };
}

static INIT: Once = Once::new();
static mut DEF_FILE: &str = "/etc/login.defs";

/// Loading a Configuration File
pub fn load_def_file() -> UResult<()> {
    let file = File::open(unsafe { DEF_FILE }).map_err(|e| {
        USimpleError::new(
            ERROR_OPEN_FILE,
            format!("Failed to open {}: {}", unsafe { DEF_FILE }, e),
        )
    })?;
    let reader = BufReader::new(file);
    let mut def_table = DEF_TABLE.lock().unwrap();

    for line in reader.lines() {
        let line = line.map_err(|e| {
            USimpleError::new(
                ERROR_READ_LINE,
                format!("Failed to read line in {}: {}", unsafe { DEF_FILE }, e),
            )
        })?;
        let trimmed_line = line.trim().to_string();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        let mut parts = trimmed_line.split_whitespace();
        if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
            def_table.insert(name.to_string(), Some(value.to_string()));
        }
    }

    Ok(())
}

/// Make sure the configuration is loaded
pub fn ensure_loaded() -> UResult<()> {
    let mut result = Ok(());
    INIT.call_once(|| {
        result = load_def_file();
    });
    result
}

/// Get string value
pub fn getdef_str(item: &str) -> Option<String> {
    ensure_loaded().ok()?;
    DEF_TABLE.lock().unwrap().get(item).cloned().unwrap_or(None)
}

/// Get Boolean Value
pub fn getdef_bool(item: &str) -> bool {
    match getdef_str(item) {
        Some(value) => value.eq_ignore_ascii_case("yes"),
        None => false,
    }
}

/// Get the value type
pub fn getdef_num<T>(item: &str, default: T) -> T
where
    T: FromStr + PartialOrd + Into<i64>,
    <T as FromStr>::Err: Debug,
{
    match getdef_str(item) {
        Some(value) => match value.parse::<T>() {
            Ok(val) => {
                if let Ok(minus_one) = T::from_str("-1") {
                    if val < minus_one || val > T::from_str(&i64::MAX.to_string()).unwrap() {
                        println!(
                            "configuration error - {} value out of range: '{}'",
                            item, value
                        );
                        return default;
                    }
                }
                val
            }
            Err(_) => {
                println!(
                    "configuration error - cannot parse {} value: '{}'",
                    item, value
                );
                default
            }
        },
        None => default,
    }
}

/// Setting Configuration Items
pub fn putdef_str(name: &str, value: &str) -> UResult<()> {
    ensure_loaded()?;
    let mut def_table = DEF_TABLE.lock().unwrap();
    if let Some(item) = def_table.get_mut(name) {
        *item = Some(value.to_string());
        Ok(())
    } else {
        Err(USimpleError::new(ERROR_UNKNOWN_ITEM, format!("Unknown item: {}", name)).into())
    }
}

/// Get unsigned long value
pub fn getdef_ulong(item: &str, default: u32) -> u32 {
    match getdef_str(item) {
        Some(value) => {
            if value.starts_with("0x") {
                parse_with_radix(&value[2..], 16, item, default)
            } else if value.starts_with('0') && value.len() > 1 {
                parse_with_radix(&value[1..], 8, item, default)
            } else if let Ok(val) = value.parse::<u32>() {
                val
            } else {
                eprintln!(
                    "configuration error - cannot parse {} value: '{}'",
                    item, value
                );
                default
            }
        }
        None => default,
    }
}

///
fn parse_with_radix(value: &str, radix: u32, item: &str, default: u32) -> u32 {
    match u32::from_str_radix(value, radix) {
        Ok(val) => val,
        Err(_) => {
            print!(
                "configuration error - cannot parse {} value: '{}'",
                item, value
            );
            default
        }
    }
}
