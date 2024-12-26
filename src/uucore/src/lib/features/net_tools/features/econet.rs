//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

pub const EC_AFTYPE: AFType = AFType {
    name: "ec",
    title: "Econet",
    af: libc::AF_ECONET,
    alen: 0,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: None,
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/econet"),
};

pub fn sprint(_sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

pub fn input(_typ: i32, _bufp: Vec<std::ffi::c_char>, _sasp: &libc::sockaddr_storage) -> i32 {
    0
}

pub fn herror(_ptr: Vec<std::ffi::c_char>) {}
