//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

///
#[derive(Clone, Debug)]
pub struct Spwd {
    /// Login name.
    pub sp_namp: String,
    /// Hashed passphrase.
    pub sp_pwdp: String,
    /// Date of last change.
    pub sp_lstchg: Option<i64>,
    /// Minimum number of days between changes.
    pub sp_min: Option<i64>,
    /// Maximum number of days between changes.
    pub sp_max: Option<i64>,
    /// Number of days to warn user to change the password.
    pub sp_warn: Option<i64>,
    /// Number of days the account may be inactive.
    pub sp_inact: Option<i64>,
    /// Number of days since 1970-01-01 until account expires.
    pub sp_expire: Option<i64>,
    /// Reserved.
    pub sp_flag: Option<i64>,
}

///
impl Spwd {
    /// Create a new Spwd instance.
    pub fn new() -> Spwd {
        Spwd {
            sp_namp: "".to_string(),
            sp_pwdp: "".to_string(),
            sp_lstchg: None,
            sp_min: None,
            sp_max: None,
            sp_warn: None,
            sp_inact: None,
            sp_expire: None,
            sp_flag: None,
        }
    }
    ///
    pub fn format_optional(value: Option<i64>) -> String {
        match value {
            Some(v) if v != -1 => v.to_string(),
            _ => "".to_string(),
        }
    }
}
