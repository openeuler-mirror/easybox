//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::mem;

use libc::{sockaddr_storage, AF_IPX};

use crate::net_tools::AFType;

pub const IPX_AFTYPE: AFType = AFType {
    name: "ipx",
    title: "Novell IPX",
    af: libc::AF_IPX,
    alen: 0,
    print: Some(print),
    sprint: Some(sprint),
    input: Some(input),
    herror: None,
    rprint: None,
    rinput: None,
    getmask: None,
    fd: -1,
    flag_file: Some("/proc/net/ipx"),
};

pub struct SockaddrIpx {
    pub sipx_family: u16,
    pub sipx_port: u16,
    pub sipx_network: u32,
    pub sipx_node: [u8; 6],
    pub sipx_socket: u16,
}

impl SockaddrIpx {
    pub fn new() -> Self {
        SockaddrIpx {
            sipx_family: AF_IPX as u16,
            sipx_port: 0,
            sipx_network: 0,
            sipx_node: [0; 6],
            sipx_socket: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < std::mem::size_of::<SockaddrIpx>() {
            return None;
        }

        let mut ipx = SockaddrIpx::new();
        ipx.sipx_family = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        ipx.sipx_port = u16::from_be_bytes(bytes[2..4].try_into().unwrap());
        ipx.sipx_network = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        ipx.sipx_node.copy_from_slice(&bytes[8..14]);
        ipx.sipx_socket = u16::from_be_bytes(bytes[14..16].try_into().unwrap());

        Some(ipx)
    }
}

impl From<&[u8]> for SockaddrIpx {
    fn from(bytes: &[u8]) -> Self {
        let mut ipx = SockaddrIpx::new();
        if bytes.len() == std::mem::size_of::<SockaddrIpx>() {
            ipx.sipx_family = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
            ipx.sipx_port = u16::from_be_bytes(bytes[2..4].try_into().unwrap());
            ipx.sipx_network = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
            ipx.sipx_node.copy_from_slice(&bytes[8..14]);
            ipx.sipx_socket = u16::from_be_bytes(bytes[14..16].try_into().unwrap());
        }
        ipx
    }
}

impl From<&sockaddr_storage> for SockaddrIpx {
    fn from(sasp: &sockaddr_storage) -> Self {
        let mut ipx = SockaddrIpx::new();
        ipx.sipx_family = sasp.ss_family;
        ipx
    }
}

pub fn print(ptr: Vec<i8>) -> String {
    let ptr: Vec<u8> = ptr.into_iter().map(|v| v as u8).collect();
    let v: [u8; 16] = ptr.try_into().unwrap();
    let sipx = SockaddrIpx::from_bytes(&v).unwrap();

    let index = sipx.sipx_node.iter().rposition(|&v| v != 0);
    if index.is_some() {
        if u32::from_be(sipx.sipx_network) != 0 {
            format!(
                "{:08X}:{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                sipx.sipx_network,
                sipx.sipx_node[0],
                sipx.sipx_node[1],
                sipx.sipx_node[2],
                sipx.sipx_node[3],
                sipx.sipx_node[4],
                sipx.sipx_node[5],
            )
        } else {
            format!("{:08X}", sipx.sipx_network)
        }
    } else if u32::from_be(sipx.sipx_network) != 0 {
        format!(
            "{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            sipx.sipx_node[0],
            sipx.sipx_node[1],
            sipx.sipx_node[2],
            sipx.sipx_node[3],
            sipx.sipx_node[4],
            sipx.sipx_node[5],
        )
    } else {
        String::new()
    }
}

pub fn sprint(_sasp: &sockaddr_storage, _numeric: i32) -> Option<String> {
    None
}

pub fn input(_typ: i32, _bufp: &str, _sasp: &mut sockaddr_storage) -> Result<i32, String> {
    Ok(0)
}

#[allow(dead_code)]
pub fn getsock(_bufp: Vec<u8>, sasp: &mut sockaddr_storage) -> i32 {
    let sipx: &mut SockaddrIpx = unsafe { mem::transmute(sasp) };
    sipx.sipx_port = 0;

    let tmp_node: Vec<u8> = sipx
        .sipx_node
        .into_iter()
        .map(|v| char::from(v).to_uppercase().next().unwrap() as u8)
        .collect();
    sipx.sipx_node = tmp_node.try_into().unwrap();

    0
}

pub fn _input(typ: i32, bufp: Vec<u8>, sasp: &mut sockaddr_storage) -> i32 {
    let sai: &mut SockaddrIpx = unsafe { mem::transmute(sasp) };
    sai.sipx_family = AF_IPX as u16;
    sai.sipx_port = 0;
    sai.sipx_network = 0;
    sai.sipx_node = [0; 6];
    sai.sipx_socket = 0;
    let mut _bufp: Vec<u8> = bufp.clone();
    let typ = typ & 3;
    if typ <= 1 {
        let input = String::from_utf8(bufp).unwrap();
        match input.parse::<u32>() {
            Ok(netnum) => {
                if netnum == 0xffffffff || netnum == 0 {
                    return -1;
                }
                sai.sipx_network = u32::from_be(netnum);
            }
            Err(_) => {
                let pos = input.find(|c: char| !c.is_ascii_digit()).unwrap();
                if typ == 1 {
                    if &input[pos..pos + 1] != "\0" {
                        return -2;
                    }
                    return 0;
                }
                if typ == 0 {
                    if &input[pos..pos + 1] != ":" {
                        return -3;
                    }
                    _bufp = _bufp[pos..].to_vec();
                }
            }
        }
    }
    let sasp: &mut sockaddr_storage = unsafe { mem::transmute(sai) };
    getsock(_bufp, sasp)
}
