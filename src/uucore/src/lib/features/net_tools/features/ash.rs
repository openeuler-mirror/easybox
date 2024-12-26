//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

#[cfg(feature = "HWASH")]
use crate::net_tools::HWType;

#[cfg(feature = "HWASH")]
pub const ASH_HWTYPE: HWType = HWType {
    name: "ash",
    title: "Ash",
    typ: libc::ARPHRD_ASH as i32,
    alen: 64,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 1,
};

pub fn print(_ptr: Vec<i8>) -> String {
    String::new()
}

#[cfg(feature = "HWASH")]
pub fn hinput(_bufp: &str, _saspp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    Ok(())
}

#[cfg(feature = "AFASH")]
use crate::net_tools::AFType;

#[cfg(feature = "AFASH")]
pub const ASH_AFTYPE: AFType = AFType {
    name: "ash",
    title: "Ash",
    af: libc::AF_ASH,
    alen: 0,
    print: Some(print),
    sprint: Some(sprint),
    input: None,
    herror: None,
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/sys/net/ash"),
};

#[cfg(feature = "AFASH")]
pub fn sprint(_sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}
