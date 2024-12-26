//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const SIT_HWTYPE: HWType = HWType {
    name: "sit",
    title: "IPv6-in-IPv4",
    typ: libc::ARPHRD_SIT as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};
