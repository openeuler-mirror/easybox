//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::net::Ipv6Addr;

use libc::sockaddr_storage;

use crate::net_tools::AFType;

pub const INET6_AFTYPE: AFType = AFType {
    name: "inet6",
    title: "IPv6",
    af: libc::AF_INET6,
    alen: std::mem::size_of::<libc::in6_addr>() as i32,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: Some(herror),
    rprint: Some(rprint),
    rinput: Some(rinput),
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/if_inet6"),
};

fn fix_v4_address(ipv6: Ipv6Addr) -> String {
    if ipv6.to_ipv4_mapped().is_some() {
        ipv6.to_ipv4().unwrap().to_string()
    } else {
        ipv6.to_string()
    }
}

pub fn print(ptr: Vec<i8>) -> String {
    let ptr: Vec<u8> = ptr.into_iter().map(|v| v as u8).collect();
    let v: [u8; 16] = ptr.try_into().unwrap();
    fix_v4_address(Ipv6Addr::from(v))
}

pub fn sprint(_sasp: &sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

pub fn input(_typ: i32, _bufp: &str, _sasp: &mut sockaddr_storage) -> Result<i32, String> {
    Ok(0)
}

pub fn herror(_ptr: &str) {}

pub fn rprint(_options: i32) -> i32 {
    0
}

pub fn rinput(_typ: i32, _ext: i32, _argv: *mut *mut libc::c_char) -> i32 {
    0
}
