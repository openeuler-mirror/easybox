//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::AFType;

pub const DDP_AFTYPE: AFType = AFType {
    name: "ddp",
    title: "Appletalk DDP",
    af: libc::AF_APPLETALK,
    alen: 0,
    print: Some(print),
    sprint: Some(sprint),
    input: None,
    herror: None,
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/appletalk"),
};

pub fn sprint(_sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

pub fn print(_ptr: Vec<i8>) -> String {
    String::new()
}
