//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use errno::{set_errno, Errno};
use libc::{sockaddr, sockaddr_storage, EINVAL};

use crate::net_tools::HWType;

pub const IB_HWTYPE: HWType = HWType {
    name: "infiniband",
    title: "InfiniBand",
    typ: libc::ARPHRD_INFINIBAND as i32,
    alen: 20, // INFINIBAND_ALEN
    print: Some(print),
    input: Some(hinput),
    activate: None,
    suppress_null_addr: 0,
};

pub fn print(ptr: Vec<i8>) -> String {
    let mut buff = String::new();
    for byte in ptr {
        buff.push_str(&format!("{:02X}:", byte));
    }
    buff.pop();

    eprintln!(
        "Infiniband hardware address can be incorrect! Please read BUGS section in ifconfig(8)."
    );
    buff
}

fn hex_char_to_val(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

pub fn hinput(bufp: &str, sasp: &mut sockaddr_storage) -> Result<(), i32> {
    let sap: &mut sockaddr = unsafe { &mut *(sasp as *mut sockaddr_storage as *mut sockaddr) };
    sap.sa_family = libc::ARPHRD_INFINIBAND;

    let mut i: usize = 0;
    let mut chars = bufp.chars().peekable();
    while let Some(c) = chars.next() {
        if i >= 20 {
            break;
        }

        let mut val = match hex_char_to_val(c) {
            Some(v) => v << 4,
            None => {
                set_errno(Errno(EINVAL));
                return Err(-1);
            }
        };

        match chars.peek() {
            Some(next) => {
                if *next == ':' || *next == '\0' {
                    val >>= 4;
                } else {
                    let val_2 = match hex_char_to_val(*next) {
                        Some(v) => v,
                        None => {
                            set_errno(Errno(EINVAL));
                            return Err(-1);
                        }
                    };
                    val |= val_2;
                }
                if *next != '\0' {
                    chars.next();
                }
            }
            None => {
                val >>= 4;
            }
        };

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            sap.sa_data[i] = val as i8;
        }
        #[cfg(target_arch = "aarch64")]
        {
            sap.sa_data[i] = val as u8;
        }
        i += 1;

        if chars.peek() == Some(&':') {
            chars.next();
        }
    }

    Ok(())
}