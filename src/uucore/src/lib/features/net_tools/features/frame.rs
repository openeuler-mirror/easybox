//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const DLCI_HWTYPE: HWType = HWType {
    name: "dlci",
    title: "Frame Relay DLCI",
    typ: libc::ARPHRD_DLCI as i32,
    alen: 3,
    print: Some(print),
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const FRAD_HWTYPE: HWType = HWType {
    name: "frad",
    title: "Frame Relay Access Device",
    typ: libc::ARPHRD_FRAD as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub fn print(ptr: Vec<i8>) -> String {
    let mut buf = [0u8; 2];
    for i in 0..2 {
        buf[i] = ptr[i] as u8;
    }

    i16::from_le_bytes(buf).to_string()
}
