//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::net_tools::HWType;

pub const PPP_HWTYPE: HWType = HWType {
    name: "ppp",
    title: "Point-Point Protocol",
    typ: libc::ARPHRD_PPP as i32,
    alen: 0,
    print: None,
    input: None,
    activate: Some(activate),
    suppress_null_addr: 0,
};

pub fn activate(_fd: i32) -> i32 {
    eprintln!("You cannot start PPP with this program.");
    -1
}
