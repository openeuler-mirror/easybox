//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use libc::{c_char, c_void, sockaddr_in, sockaddr_storage, AF_INET, INADDR_ANY};
use std::{
    ffi::{CStr, CString},
    mem,
    net::Ipv4Addr,
    str::FromStr,
};

use crate::net_tools::AFType;

pub const INET_AFTYPE: AFType = AFType {
    name: "inet",
    title: "DARPA Internet",
    af: libc::AF_INET,
    alen: std::mem::size_of::<libc::c_ulong>() as i32,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: Some(herror),
    rprint: None,
    rinput: None,
    getmask: Some(getnetmask),
    fd: -1,
    flag_file: None,
};

#[link(name = "c")]
extern "C" {
    fn getnetbyaddr(net: u32, addr_type: libc::c_int) -> *mut NetworkEntry;
    fn getnetbyname(name: *const c_char) -> *mut NetworkEntry;
    fn gethostbyaddr(
        addr: *const c_void,
        len: libc::socklen_t,
        addr_type: libc::c_int,
    ) -> *mut HostEntry;
    fn gethostbyname(name: *const c_char) -> *mut HostEntry;
}

#[repr(C)]
struct NetworkEntry {
    n_name: *mut c_char,
    n_aliases: *mut *mut c_char,
    n_addrtype: i32,
    n_net: u32,
}

#[repr(C)]
struct HostEntry {
    h_name: *mut c_char,
    h_aliases: *mut *mut c_char,
    h_addrtype: i32,
    h_length: i32,
    h_addr_list: *mut *mut c_char,
}

pub fn print(ptr: Vec<i8>) -> String {
    let ptr: Vec<u8> = ptr.into_iter().map(|v| v as u8).collect();
    let v: [u8; 4] = ptr.try_into().unwrap();
    Ipv4Addr::from(v).to_string()
}

pub fn rrsolve(sasp: &sockaddr_storage, numeric: i32, netmask: u32) -> Result<String, String> {
    let sin: &sockaddr_in = unsafe { mem::transmute(sasp) };

    if sin.sin_family != AF_INET as u16 {
        // #[cfg(debug_assertions)]
        // eprintln!("rresolve: unsupport address family {}", sin.sin_family);
        return Err(String::new());
    }
    let ad: u32 = sin.sin_addr.s_addr;
    // #[cfg(debug_assertions)]
    // eprintln!(
    //     "rresolve: {:8x}, mask {:8x}, num {:8x}",
    //     ad, netmask, numeric
    // );

    // if no symbolic names are requested we shortcut with ntoa
    // if numeric & 0x0FFF != 0 {
    //     dbg!(Ipv4Addr::from(u32::from_be(ad)));
    //     return Ok(Ipv4Addr::from(u32::from_be(ad)).to_string());
    // }

    // we skip getnetbyaddr for 0.0.0.0/0 and 0.0.0.0/~0
    if ad == INADDR_ANY {
        if netmask == INADDR_ANY {
            // for 0.0.0.0/0 we hardcode symbolic name
            let name = if numeric & 0x8000 != 0 {
                String::from("default")
            } else {
                String::from("*")
            };
            return Ok(name);
        } else {
            // for 0.0.0.0/1 we skip getnetbyname()
            return Ok(String::from("0.0.0.0"));
        }
    }

    // it is a host address if flagged or any host bits set
    let host = ad & !netmask != 0 || numeric & 0x4000 != 0;

    if host {
        // #[cfg(debug_assertions)]
        // eprintln!("gethostbyaddr ({:08x})", ad);
        let host_entry = unsafe { gethostbyaddr(&ad as *const _ as *const c_void, 4, AF_INET) };
        if !host_entry.is_null() {
            let name = unsafe { CStr::from_ptr((*host_entry).h_name) };
            return Ok(name.to_string_lossy().into_owned());
        }
    } else {
        let host_ad = u32::from_be(ad);
        // #[cfg(debug_assertions)]
        // eprintln!("getnetbyaddr ({:08x})", host_ad);
        let net_entry = unsafe { getnetbyaddr(host_ad, AF_INET) };
        if !net_entry.is_null() {
            let name = unsafe { CStr::from_ptr((*net_entry).n_name) };
            return Ok(name.to_string_lossy().into_owned());
        }
    }

    Ok(Ipv4Addr::from(u32::from_be(ad)).to_string())
}

pub fn sprint(sasp: &sockaddr_storage, numeric: i32) -> Option<String> {
    if sasp.ss_family == 0xFFFF || sasp.ss_family == 0 {
        return Some(String::from("[NONE SET]"));
    }
    match rrsolve(sasp, numeric, 0xFFFFFF00) {
        Ok(name) => Some(name),
        Err(_) => None,
    }
}

fn ipv4_to_s_addr(ip: Ipv4Addr) -> u32 {
    u32::from_le_bytes(ip.octets())
}

pub fn resolve(name: &str, sasp: &mut sockaddr_storage, hostfirst: i32) -> Result<i32, String> {
    let sin: &mut sockaddr_in = unsafe { mem::transmute(sasp) };
    sin.sin_family = AF_INET as u16;
    sin.sin_port = 0;

    /* Default is special, meaning 0.0.0.0. */
    if name.eq("default") {
        sin.sin_addr.s_addr = libc::INADDR_ANY;
        return Ok(1);
    }
    /* Look to see if it's a dotted quad. */
    if let Ok(ip) = Ipv4Addr::from_str(name) {
        sin.sin_addr.s_addr = ipv4_to_s_addr(ip);
        return Ok(0);
    };
    let name_cstring = CString::new(name).expect("CString::new failed");
    /* If we expect this to be a hostname, try hostname database first */
    if hostfirst != 0 {
        // #[cfg(debug_assertions)]
        // eprintln!("gethostbyname {}", name);

        let host_entry = unsafe { gethostbyname(name_cstring.as_ptr()) };
        if !host_entry.is_null() {
            let addr_ptr = unsafe { *((*host_entry).h_addr_list).offset(0) };
            let addr = unsafe { *(addr_ptr as *mut u32) };
            let ipv4_addr = Ipv4Addr::from(addr.to_be());
            sin.sin_addr.s_addr = ipv4_to_s_addr(ipv4_addr);
            return Ok(0);
        }
    }
    /* Try the NETWORKS database to see if this is a known network. */
    // #[cfg(debug_assertions)]
    // eprintln!("getnetbyname {}", name);

    let net_entry = unsafe { getnetbyname(name_cstring.as_ptr()) };
    if !net_entry.is_null() {
        let addr = unsafe { (*net_entry).n_net };
        let ipv4_addr = Ipv4Addr::from(addr.to_le());
        sin.sin_addr.s_addr = ipv4_to_s_addr(ipv4_addr);
        return Ok(1);
    }
    if hostfirst != 0 {
        /* Don't try again */
        return Err(format!("{}: Unknown host", name));
    }

    // #[cfg(debug_assertions)]
    // eprintln!("gethostbyname {}", name);

    let host_entry = unsafe { gethostbyname(name_cstring.as_ptr()) };
    if host_entry.is_null() {
        return Err(format!("{}: Unknown host", name));
    }
    let addr_ptr = unsafe { *((*host_entry).h_addr_list).offset(0) };
    let addr = unsafe { *(addr_ptr as *mut u32) };
    let ipv4_addr = Ipv4Addr::from(addr.to_be());
    sin.sin_addr.s_addr = ipv4_to_s_addr(ipv4_addr);

    Ok(0)
}

pub fn getsock(bufp: &str, sasp: &mut sockaddr_storage) -> Result<i32, String> {
    let sin: &mut sockaddr_in = unsafe { mem::transmute(sasp) };
    sin.sin_family = AF_INET as u16;
    sin.sin_port = 0;

    let mut val = 0;
    let mut sp = bufp.chars();
    for _ in 0..4 {
        let mut byte: u8 = 0;

        for _ in 0..2 {
            if let Some(ch) = sp.next() {
                byte <<= 4;

                let digit = match ch.to_ascii_uppercase() {
                    '0'..='9' => ch as u8 - b'0',
                    'A'..='F' => ch as u8 - b'A' + 10,
                    _ => return Err(format!("Invalid hexadecimal character: {}", ch)),
                };

                byte |= digit;
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }

        val = (val << 8) | byte as u32;
    }
    sin.sin_addr.s_addr = u32::from_be(val);

    Ok(mem::size_of::<libc::in_addr>() as i32 * 2_i32)
}

pub fn input(typ: i32, bufp: &str, sasp: &mut sockaddr_storage) -> Result<i32, String> {
    match typ {
        1 => getsock(bufp, sasp),
        256 => resolve(bufp, sasp, 1),
        _ => resolve(bufp, sasp, 0),
    }
}

pub fn herror(ptr: &str) {
    eprintln!("{}", ptr);
}

pub fn getnetmask(
    _src: *mut libc::c_char,
    _mask: &sockaddr_storage,
    _name: *mut libc::c_char,
) -> i32 {
    0
}
