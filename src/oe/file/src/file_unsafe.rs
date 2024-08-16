//! This file is part of the easybox package.
//
// (c) Zhihua Zhao <YuukaC@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

//! Encapsulation of unsafe functions

pub fn setlocale() {
    unsafe {
        libc::setlocale(libc::LC_CTYPE, "\0".as_ptr() as *const libc::c_char);
    }
}

#[cfg(feature = "wide")]
pub fn wide_char_width(c: char) -> i32 {
    extern "C" {
        fn wcwidth(c: libc::wchar_t) -> libc::c_int;
    }
    unsafe { wcwidth(c as libc::wchar_t) as i32 }
}

#[cfg(feature = "wide")]
pub fn is_wide_print(c: char) -> bool {
    extern "C" {
        fn iswprint(c: libc::wchar_t) -> libc::c_int;
    }
    unsafe { iswprint(c as libc::wchar_t) != 0 }
}

#[cfg(feature = "sandbox")]
pub fn prctl() -> bool {
    // prevent child processes from getting more priv e.g. via setuid, capabilities, ...
    unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != -1
            && libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != -1
    }
}
