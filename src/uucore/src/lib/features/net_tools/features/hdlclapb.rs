//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const HDLC_HWTYPE: HWType = HWType {
    name: "hdlc",
    title: "(Cisco)-HDLC",
    typ: libc::ARPHRD_HDLC as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const LAPB_HWTYPE: HWType = HWType {
    name: "lapb",
    title: "LAPB",
    typ: libc::ARPHRD_LAPB as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};
