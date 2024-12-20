//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

#[cfg(feature = "HWX25")]
use crate::net_tools::HWType;

#[cfg(feature = "HWX25")]
pub const X25_HWTYPE: HWType = HWType {
    name: "x25",
    title: "generic X.25",
    typ: libc::ARPHRD_X25 as i32,
    alen: 16,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

pub fn print(_ptr: Vec<i8>) -> String {
    String::new()
}

#[cfg(feature = "HWX25")]
pub fn hinput(_bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    Ok(())
}

#[cfg(feature = "AFX25")]
use crate::net_tools::AFType;

#[cfg(feature = "AFX25")]
pub const X25_AFTYPE: AFType = AFType {
    name: "x25",
    title: "CCITT X.25",
    af: libc::AF_X25,
    alen: 16,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: Some(herror),
    rprint: Some(rprint),
    rinput: Some(rinput),
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/x25"),
};

#[cfg(feature = "AFX25")]
pub fn sprint(_sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

#[cfg(feature = "AFX25")]
pub fn input(_typ: i32, _bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<i32, String> {
    Ok(0)
}

#[cfg(feature = "AFX25")]
pub fn herror(_ptr: &str) {}

#[cfg(feature = "AFX25")]
pub fn rprint(_options: i32) -> i32 {
    0
}

#[cfg(feature = "AFX25")]
pub fn rinput(_typ: i32, _ext: i32, _argv: *mut *mut libc::c_char) -> i32 {
    0
}
