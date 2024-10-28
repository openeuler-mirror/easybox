//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::{
    cmp::min,
    ffi::c_char,
    net::IpAddr,
    num::ParseIntError,
    os::unix::{
        ffi::OsStrExt,
        io::{AsFd, AsRawFd},
    },
    path::PathBuf,
};

use libc::{c_uint, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use nix::{
    errno::Errno,
    sys::{
        sendfile::sendfile,
        stat::{fstat, SFlag},
    },
    unistd::{isatty, read, ttyname, write},
};

use crate::login_unsafe::usleep_wrapper;

/* --- ttyutils --- */
/// Get terminal name, return (tty_path, tty_name, tty_number)
pub fn get_terminal_name() -> Result<(PathBuf, String, String), Errno> {
    let fd = get_terminal_stdfd()?;
    let tty = ttyname(fd)?;
    let tty_path = tty.clone();
    let tty_name = match tty.strip_prefix("/dev/") {
        Ok(v) => v,
        Err(_) => &tty,
    };
    let mut tty_number_vec = vec![];
    let mut found_digit = false;
    for i in tty_name.as_os_str().as_bytes() {
        if i.is_ascii_digit() {
            found_digit = true;
        }
        if found_digit {
            tty_number_vec.push(i.clone());
        }
    }
    Ok((
        tty_path,
        tty_name.to_string_lossy().to_string(),
        String::from_utf8_lossy(&tty_number_vec).to_string(),
    ))
}

/// Get terminal std in/out/err fd
fn get_terminal_stdfd() -> Result<i32, Errno> {
    if let Ok(true) = isatty(STDIN_FILENO) {
        return Ok(STDIN_FILENO);
    }
    if let Ok(true) = isatty(STDOUT_FILENO) {
        return Ok(STDOUT_FILENO);
    }
    if let Ok(true) = isatty(STDERR_FILENO) {
        return Ok(STDERR_FILENO);
    }
    Err(Errno::EINVAL)
}
/* --- ttyutils end --- */

/* --- strutils --- */
/// Copy string into byte array
#[inline]
pub fn str2memcpy(dest: &mut [c_char], src: &str) {
    let bytes_num = min(src.len() + 1, dest.len());
    let src_bytes = src.as_bytes();
    for i in 0..bytes_num {
        dest[i] = *src_bytes.get(i).unwrap_or(&0) as c_char;
    }
}
/* --- ttyutils end --- */

/// Convert an `IpAddr` to the format expected by `Utmpx`.
/// https://docs.rs/crate/utmpx/latest/source/src/lib.rs#116
pub fn cast_addr(addr: &IpAddr) -> [c_uint; 4usize] {
    match addr {
        IpAddr::V4(ipv4addr) => [u32::from(*ipv4addr), 0, 0, 0],
        IpAddr::V6(ipv6addr) => {
            let octets = ipv6addr.octets();
            let mut ret = [0u32; 4];
            for i in 0..4 {
                ret[i] = u32::from(octets[i * 4]) << 24
                    | u32::from(octets[i * 4 + 1]) << 16
                    | u32::from(octets[i * 4 + 2]) << 8
                    | u32::from(octets[i * 4 + 3]);
            }
            ret
        }
    }
}

/// Convert val to an `unsigned long int' in auto base.
/// The base is determined by the presence of a leading
/// zero, indicating octal or a leading "0x" or "0X", indicating hexadecimal.
pub fn strtoul_auto(val: &str) -> Result<u32, ParseIntError> {
    let mut base = 10;
    let mut v = val;
    if val.starts_with('0') {
        let a = val.chars().nth(2);
        if a == Some('x') || a == Some('X') {
            base = 16;
            v = &val[2..];
        } else if a.is_some() {
            base = 8;
            v = &val[1..];
        }
    }
    u32::from_str_radix(v, base)
}

/// Read data in 'from' stream and write them in 'to' stream
pub fn ul_copy_file<F1: AsFd, F2: AsFd>(from: F1, to: F2) {
    if let Ok(st) = fstat(from.as_fd().as_raw_fd()) {
        if !SFlag::from_bits_truncate(st.st_mode).contains(SFlag::S_IFREG) {
            copy_file_simple(from, to);
            return;
        }
        if sendfile_all(&to, &from, st.st_size as usize).is_err() {
            copy_file_simple(from, to);
        }
    }
}

fn sendfile_all<F1: AsFd, F2: AsFd>(out: &F1, infd: &F2, mut count: usize) -> Result<(), ()> {
    let mut tries = 0;
    while count > 0 {
        let ret = sendfile(out, infd, None, count);
        if let Err(err) = ret {
            if (err == Errno::EAGAIN || err == Errno::EINTR) && tries < 5 {
                tries += 1;
                usleep_wrapper(250000);
                continue;
            }
            return Err(());
        }
        let ret_size = ret.unwrap();
        if ret_size == 0 {
            return Ok(());
        }
        tries = 0;
        count -= ret_size;
    }
    Ok(())
}

fn copy_file_simple<F1: AsFd, F2: AsFd>(from: F1, to: F2) {
    let mut buf = [0 as u8; 8192];
    loop {
        let nr = read_all(&from, &mut buf);
        if nr.is_err() || nr == Ok(0) {
            return;
        }
        if write_all(&to, &buf[..nr.unwrap()]).is_err() {
            return;
        }
    }
}

fn read_all<F: AsFd>(fd: &F, buf: &mut [u8]) -> Result<usize, ()> {
    let mut tries = 0;
    let mut has_read = 0;
    while has_read < buf.len() {
        let ret = read(fd.as_fd().as_raw_fd(), &mut buf[has_read..]);
        if let Err(err) = ret {
            if (err == Errno::EAGAIN || err == Errno::EINTR) && tries < 5 {
                tries += 1;
                usleep_wrapper(250000);
                continue;
            }
            return Err(());
        }
        let ret = ret.unwrap();
        if ret == 0 {
            return Ok(has_read);
        }
        tries = 0;
        has_read += ret;
    }
    Ok(has_read)
}

fn write_all<F: AsFd>(fd: &F, buf: &[u8]) -> Result<(), ()> {
    let mut has_write = 0;
    while has_write < buf.len() {
        let tmp = write(fd.as_fd().as_raw_fd(), &buf[has_write..]);
        if let Ok(ret) = tmp {
            has_write += ret;
        } else if tmp != Err(Errno::EINTR) && tmp != Err(Errno::EAGAIN) {
            return Err(());
        }
        if tmp == Err(Errno::EAGAIN) {
            /* Try later, *sigh* */
            usleep_wrapper(250000);
        }
    }
    return Ok(());
}
