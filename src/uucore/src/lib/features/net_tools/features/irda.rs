//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const IRDA_HWTYPE: HWType = HWType {
    name: "irda",
    title: "IrLAP",
    typ: libc::ARPHRD_IRDA as i32,
    alen: 2,
    print: Some(print),
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub fn print(ptr: Vec<i8>) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}",
        ptr[3], ptr[2], ptr[1], ptr[0]
    )
}
