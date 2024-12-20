//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const UNSPEC_HWTYPE: HWType = HWType {
    name: "unspec",
    title: "UNSPEC",
    typ: -1,
    alen: 0,
    print: Some(pr_unspec),
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const LOOP_HWTYPE: HWType = HWType {
    name: "loop",
    title: "Local Loopback",
    typ: libc::ARPHRD_LOOPBACK as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub fn pr_unspec(ptr: Vec<i8>) -> String {
    let mut result = String::new();
    for byte in ptr {
        result.push_str(&format!("{:02X}-", byte));
    }
    if !result.is_empty() {
        result.pop();
    }
    result
}
