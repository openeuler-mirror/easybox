//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const STRIP_HWTYPE: HWType = HWType {
    name: "strip",
    title: "Metricom Starmode IP",
    typ: libc::ARPHRD_METRICOM as i32,
    alen: 6, // sizeof(MetricomAddress)
    print: Some(print),
    input: Some(hinput),
    activate: Some(activate),
    suppress_null_addr: 0,
};

pub fn print(_ptr: Vec<i8>) -> String {
    String::new()
}

pub fn hinput(_bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    Ok(())
}

pub fn activate(_fd: i32) -> i32 {
    0
}
