//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const EC_HWTYPE: HWType = HWType {
    name: "ec",
    title: "Econet",
    typ: libc::ARPHRD_ECONET as i32,
    alen: 0,
    print: None,
    input: None,
    activate: None,
    suppress_null_addr: 0,
};
