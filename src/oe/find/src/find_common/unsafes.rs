//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

///
pub fn get_sys_arg_max() -> Option<i64> {
    let arg_max = unsafe { libc::sysconf(libc::_SC_ARG_MAX) };

    if arg_max == -1 {
        None
    } else {
        Some(arg_max)
    }
}
