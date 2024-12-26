//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

#[cfg(feature = "HWAX25")]
use crate::net_tools::HWType;

#[cfg(feature = "HWAX25")]
pub const AX25_HWTYPE: HWType = HWType {
    name: "ax25",
    title: "AMPR AX.25",
    typ: libc::ARPHRD_AX25 as i32,
    alen: 7,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

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

#[cfg(feature = "HWAX25")]
pub fn hinput(_bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    Ok(())
}

#[cfg(feature = "AFAX25")]
use crate::net_tools::AFType;

#[cfg(feature = "AFAX25")]
pub const AX25_AFTYPE: AFType = AFType {
    name: "ax25",
    title: "AMPR AX.25",
    af: libc::AF_AX25,
    alen: 7,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: Some(herror),
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/ax25"),
};

#[cfg(feature = "AFAX25")]
pub fn sprint(_sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

#[cfg(feature = "AFAX25")]
pub fn input(_typ: i32, _bufp: &str, _sasp: &mut libc::sockaddr_storage) -> Result<i32, String> {
    Ok(0)
}

#[cfg(feature = "AFAX25")]
pub fn herror(_ptr: &str) {}
