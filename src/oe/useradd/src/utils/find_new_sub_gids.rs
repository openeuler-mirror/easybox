//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use super::{getdef::getdef_ulong, subordinateio::sub_gid_find_free_range};

///
pub fn find_new_sub_gids(range_start: &mut u32, range_count: &mut u32) -> i32 {
    let min = getdef_ulong("SUB_UID_MIN", 100_000);
    let max = getdef_ulong("SUB_UID_MAX", 600_100_000);
    let count = getdef_ulong("SUB_UID_COUNT", 65_536);

    if min > max || count >= max || (min + count - 1) > max {
        eprintln!(
            "Invalid configuration: SUB_UID_MIN ({}), SUB_UID_MAX ({}), SUB_UID_COUNT ({})",
            min, max, count
        );
        return -1;
    }

    let start = sub_gid_find_free_range(min, max, count);
    if start == -1 {
        eprintln!("Can't get unique subordinate UID range");
        return -1;
    }

    *range_start = start as u32;
    *range_count = count;
    0
}
