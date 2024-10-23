//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

/// A record in the shadow database.
#[derive(Clone, Debug)]
pub struct Spwd {
    ///
    pub sp_namp: String,
    ///
    pub sp_pwdp: String,
    ///
    pub sp_lstchg: Option<i64>,
    ///
    pub sp_min: Option<i64>,
    ///
    pub sp_max: Option<i64>,
    ///
    pub sp_warn: Option<i64>,
    ///
    pub sp_inact: Option<i64>,
    ///
    pub sp_expire: Option<i64>,
    ///
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
