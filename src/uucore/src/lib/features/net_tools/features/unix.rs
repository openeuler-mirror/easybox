//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use libc::sockaddr_storage;

use crate::net_tools::AFType;

#[cfg(feature = "AFUNIX")]
pub const UNIX_AFTYPE: AFType = AFType {
    name: "unix",
    title: "UNIX Domain",
    af: libc::AF_UNIX,
    alen: 0,
    print: Some(print),
    sprint: Some(sprint),
    input: None,
    herror: None,
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/unix"),
};

#[cfg(feature = "AFUNIX")]
pub fn print(ptr: Vec<i8>) -> String {
    let ptr: Vec<u8> = ptr.iter().map(|&v| v as u8).collect();
    std::ffi::CString::new(ptr)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

#[cfg(feature = "AFUNIX")]
pub fn sprint(sasp: &sockaddr_storage, _numeric: i32) -> Option<String> {
    let sap = sasp as *const _ as *const libc::sockaddr;
    unsafe {
        if (*sap).sa_family == 0xFFFF || (*sap).sa_family == 0 {
            Some(String::from("[NONE SET]"))
        } else {
            Some(print(
                ((*sap).sa_data.iter().map(|&x| x as i8)).collect::<Vec<i8>>(),
            ))
        }
    }
}

pub const UNSPEC_AFTYPE: AFType = AFType {
    name: "unspec",
    title: "UNSPEC",
    af: libc::AF_UNSPEC,
    alen: 0,
    print: Some(unspec_print),
    sprint: Some(unspec_sprint),
    input: None,
    herror: None,
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: None,
};

pub fn unspec_print(_ptr: Vec<i8>) -> String {
    todo!()
}

pub fn unspec_sprint(_sasp: &sockaddr_storage, _numeric: i32) -> Option<String> {
    todo!()
}
