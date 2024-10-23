//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use nix::unistd::Uid;

///
pub fn get_uid(optarg: &str) -> Option<Uid> {
    let trimmed = optarg.trim();

    if trimmed.is_empty() {
        return None;
    }

    match trimmed.parse::<i64>() {
        Ok(val) if val >= 0 && val <= u32::MAX as i64 => Some(Uid::from_raw(val as u32)),
        _ => None,
    }
}
