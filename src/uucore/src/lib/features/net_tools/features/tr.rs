//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const TR_HWTYPE: HWType = HWType {
    name: "tr",
    title: "16/4 Mbps Token Ring",
    typ: libc::ARPHRD_IEEE802 as i32,
    alen: 0,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

#[cfg(feature = "ARPHRD_IEEE802_TR")]
pub const TR_HWTYPE1: HWType = HWType {
    name: "tr",
    title: "16/4 Mbps Token Ring (New)",
    typ: libc::ARPHRD_IEEE802_TR as i32,
    alen: 0,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

pub fn print(_ptr: Vec<i8>) -> String {
    String::new()
}

pub fn hinput(_bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    Ok(())
}
