//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use nix::{ioctl_readwrite_bad, ioctl_write_ptr_bad};
use std::{mem, ptr};
use uucore::libc::{arpreq, c_int, ifreq, sockaddr, socket};

/// Get a socket used in ioctl.
pub fn socket_wrapper(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    unsafe { socket(domain, ty, protocol) }
}

/// Initialize some C structures.
pub fn zeroed_wrapper<T>() -> T {
    unsafe { mem::zeroed() }
}

/// Copy memory between some C structures.
pub fn memcpy_wrapper(src: *const u8, dst: *mut u8, count: usize) {
    unsafe { ptr::copy_nonoverlapping(src, dst, count) }
}

// SIOCGIFHWADDR => 0x8927
ioctl_readwrite_bad!(
    /// Use ioctl to get hardware address.
    ioctl_get_hardware_address,
    0x8927,
    ifreq
);
// SIOCDARP => 0x8953
ioctl_write_ptr_bad!(
    /// Use ioctl to delete an arp entry.
    ioctl_delete_arp,
    0x8953,
    arpreq
);
// SIOCSARP => 0x8955
ioctl_write_ptr_bad!(
    /// Use ioctl to set an arp entry.
    ioctl_set_arp,
    0x8955,
    arpreq
);

/// Get hardware address by ioctl.
pub fn ioctl_get_hardware_address_wrapper(fd: c_int, ifr: *mut ifreq) -> nix::Result<c_int> {
    unsafe { ioctl_get_hardware_address(fd, ifr) }
}

/// Delete arp entry by ioctl.
pub fn ioctl_delete_arp_wrapper(fd: c_int, req: arpreq) -> nix::Result<c_int> {
    unsafe { ioctl_delete_arp(fd, &req) }
}

/// Set arp entry by ioctl.
pub fn ioctl_set_arp_wrapper(fd: c_int, req: arpreq) -> nix::Result<c_int> {
    unsafe { ioctl_set_arp(fd, &req) }
}

/// Get ifru_hwaddr in union.
pub fn ifru_hwaddr_wrapper(ifr: ifreq) -> sockaddr {
    unsafe { ifr.ifr_ifru.ifru_hwaddr }
}
