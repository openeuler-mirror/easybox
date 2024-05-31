//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use libc::{c_char, snprintf};

/// Print a float with modifier "%g" in C by [`libc::snprintf`].
pub fn pretty_float(value: f64) -> String {
    const NUMSTR_SIZE: usize = 200;

    let format = ['%' as u8, 'g' as u8, 0u8];
    let numstr = [0u8; NUMSTR_SIZE];

    let nbchars = unsafe {
        snprintf(
            numstr.as_ptr() as *mut c_char,
            NUMSTR_SIZE,
            format.as_ptr() as *const c_char,
            value,
        )
    };

    if nbchars < 0 || nbchars >= NUMSTR_SIZE as i32 {
        return String::from("");
    }
    let numstr = &numstr[..nbchars as usize];

    unsafe { String::from_utf8_unchecked(numstr.to_vec()) }
}
