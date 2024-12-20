//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const SLIP_HWTYPE: HWType = HWType {
    name: "slip",
    title: "Serial Line IP",
    typ: libc::ARPHRD_SLIP as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const CSLIP_HWTYPE: HWType = HWType {
    name: "cslip",
    title: "VJ Serial Line IP",
    typ: libc::ARPHRD_CSLIP as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const SLIP6_HWTYPE: HWType = HWType {
    name: "slip6",
    title: "6-bit Serial Line IP",
    typ: libc::ARPHRD_SLIP6 as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const CSLIP6_HWTYPE: HWType = HWType {
    name: "cslip6",
    title: "VJ 6-bit Serial Line IP",
    typ: libc::ARPHRD_CSLIP6 as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};

pub const ADAPTIVE_HWTYPE: HWType = HWType {
    name: "adaptive",
    title: "Adaptive Serial Line IP",
    typ: libc::ARPHRD_ADAPT as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};
