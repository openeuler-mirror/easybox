//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

#[cfg(feature = "HWNETROM")]
use crate::net_tools::HWType;

#[cfg(feature = "HWNETROM")]
pub const NETROM_HWTYPE: HWType = HWType {
    name: "netrom",
    title: "AMPR NET/ROM",
    typ: libc::ARPHRD_NETROM as i32,
    alen: 7,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

#[cfg(any(feature = "HWNETROM", feature = "AFNETROM"))]
pub fn print(ptr: Vec<i8>) -> String {
    let mut buff: [u8; 8] = [0; 8];
    for i in 0..6 {
        buff[i] = (ptr[i] as u8) >> 1;
        if buff[i] == b' ' {
            buff[i] = b'\0'
        }
    }
    let mut call_str: String = std::ffi::CStr::from_bytes_with_nul(&buff)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let tmp: u8 = (ptr[6] as u8 & 0x1E) >> 1;
    if tmp != 0 {
        call_str.push_str(format!("-{}", tmp).as_str());
    }
    call_str
}

#[cfg(feature = "HWNETROM")]
pub fn hinput(_bufp: &str, _saspp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    Ok(())
}

#[cfg(feature = "AFNETROM")]
use crate::net_tools::AFType;

#[cfg(feature = "AFNETROM")]
pub const NETROM_AFTYPE: AFType = AFType {
    name: "netrom",
    title: "AMPR NET/ROM",
    af: libc::AF_NETROM,
    alen: 7,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: Some(herror),
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/nr"),
};

#[cfg(feature = "AFNETROM")]
pub fn sprint(_sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

#[cfg(feature = "AFNETROM")]
pub fn input(_typ: i32, _bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<i32, String> {
    Ok(0)
}

#[cfg(feature = "AFNETROM")]
pub fn herror(_ptr: &str) {}
