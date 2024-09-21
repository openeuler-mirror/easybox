//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

///
#[derive(Clone, Debug)]
pub struct Passwd {
    ///
    pub pw_name: String,
    ///
    pub pw_passwd: Option<String>,
    ///
    pub pw_uid: u32,
    ///
    pub pw_gid: u32,
    ///
    pub pw_gecos: Option<String>,
    ///
    pub pw_dir: Option<String>,
    ///
    pub pw_shell: Option<String>,
}

impl Passwd {
    /// Create a new passwd instance.
    pub fn new() -> Passwd {
        Passwd {
            pw_name: "".to_string(),
            pw_passwd: None,
            pw_uid: 0,
            pw_gid: 0,
            pw_gecos: None,
            pw_dir: None,
            pw_shell: None,
        }
    }
    ///
    pub fn format_optional(value: Option<String>) -> String {
        match value {
            Some(v) => v,
            None => "".to_string(),
        }
    }
}
