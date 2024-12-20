//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

static ROSE_ERRMSG: std::sync::Mutex<&str> = std::sync::Mutex::new("");

#[repr(C)]
pub struct RoseAddress {
    rose_addr: [i8; 5],
}

#[repr(C)]
pub struct Ax25Address {
    ax25_addr: [i8; 7],
}

#[repr(C)]
pub struct SockaddrRose {
    srose_family: libc::sa_family_t,
    srose_addr: RoseAddress,
    srose_call: Ax25Address,
    srose_ndigis: libc::c_int,
    srose_digi: Ax25Address,
}

use errno::{set_errno, Errno};
use libc::EINVAL;

#[cfg(feature = "HWROSE")]
use crate::net_tools::HWType;

#[cfg(feature = "HWROSE")]
pub const ROSE_HWTYPE: HWType = HWType {
    name: "rose",
    title: "AMPR ROSE",
    typ: libc::ARPHRD_ROSE as i32,
    alen: 10,
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

pub fn print(ptr: Vec<i8>) -> String {
    if ptr.len() >= 5 {
        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}",
            ptr[0] as u8, ptr[1] as u8, ptr[2] as u8, ptr[3] as u8, ptr[4] as u8
        )
    } else {
        String::new()
    }
}

#[cfg(feature = "HWROSE")]
pub fn hinput(bufp: &str, sasp: &mut libc::sockaddr_storage) -> Result<(), i32> {
    let sap = unsafe { &mut *(sasp as *mut libc::sockaddr_storage as *mut libc::sockaddr) };
    if input(0, bufp, sasp).is_err() {
        return Err(-1);
    }
    sap.sa_family = libc::ARPHRD_ROSE;
    Ok(())
}

#[cfg(feature = "AFROSE")]
use crate::net_tools::AFType;

#[cfg(feature = "AFROSE")]
pub const ROSE_AFTYPE: AFType = AFType {
    name: "rose",
    title: "AMPR ROSE",
    af: libc::AF_ROSE,
    alen: 10,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: Some(herror),
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/rose"),
};

#[cfg(feature = "AFROSE")]
pub fn sprint(sasp: &libc::sockaddr_storage, _numeric: i32) -> Option<String> {
    let sap = sasp as *const libc::sockaddr_storage as *const libc::sockaddr;
    unsafe {
        if (*sap).sa_family == 0xFFFF || (*sap).sa_family == 0 {
            return Some("[NONE SET]".to_string());
        }
    }

    let rose_sap = sasp as *const libc::sockaddr_storage as *const SockaddrRose;
    Some(print(unsafe { (*rose_sap).srose_addr.rose_addr.to_vec() }))
}

pub fn input(_typ: i32, bufp: &str, sasp: &mut libc::sockaddr_storage) -> Result<i32, String> {
    let sap = unsafe { &mut *(sasp as *mut libc::sockaddr_storage as *mut libc::sockaddr) };
    sap.sa_family = libc::AF_ROSE as u16;

    if bufp.len() != 10 {
        let mut errmsg = ROSE_ERRMSG.lock().unwrap();
        *errmsg = "Node address must be ten digits";
        set_errno(Errno(EINVAL));
        return Err(String::new());
    }

    let rose_sap = unsafe { &mut *(sasp as *mut libc::sockaddr_storage as *mut SockaddrRose) };
    let mut o;
    for i in 0..5 {
        o = i * 2;
        let byte_value = ((bufp.as_bytes()[o] - b'0') << 4) | (bufp.as_bytes()[o + 1] - b'0');
        rose_sap.srose_addr.rose_addr[i] = byte_value as i8;
    }

    Ok(0)
}

#[cfg(feature = "AFROSE")]
pub fn herror(text: &str) {
    if text.is_empty() {
        eprintln!("{}", ROSE_ERRMSG.lock().unwrap())
    } else {
        eprintln!("{}: {}", text, ROSE_ERRMSG.lock().unwrap())
    }
}
