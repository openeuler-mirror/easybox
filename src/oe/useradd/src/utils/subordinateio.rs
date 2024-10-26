//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Clone)]
struct SubordinateRange {
    ///
    owner: String,
    ///
    start: u32,
    ///
    count: u32,
}

///
impl SubordinateRange {
    ///
    fn new(owner: &str, start: u32, count: u32) -> Self {
        SubordinateRange {
            owner: owner.to_string(),
            start,
            count,
        }
    }
}

///
fn read_subordinate_ranges(file_path: &str) -> io::Result<Vec<SubordinateRange>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut ranges = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() == 3 {
            if let (Ok(start), Ok(count)) = (fields[1].parse::<u32>(), fields[2].parse::<u32>()) {
                ranges.push(SubordinateRange::new(fields[0], start, count));
            }
        }
    }

    Ok(ranges)
}

///
fn find_free_range(
    ranges: &mut Vec<SubordinateRange>,
    min: u32,
    max: u32,
    count: u32,
) -> Option<u32> {
    ranges.sort_by(|a, b| a.start.cmp(&b.start));

    let mut allocated_ranges = BTreeSet::new();
    for range in ranges.iter() {
        allocated_ranges.insert((range.start, range.start + range.count - 1));
    }
    let mut low = min;
    for &(start, end) in allocated_ranges.iter() {
        if low + count - 1 < start {
            return Some(low);
        }

        low = end + 1;
        if low > max {
            break;
        }
    }

    if low + count - 1 <= max {
        return Some(low);
    }

    None
}

///
pub fn sub_uid_find_free_range(min: u32, max: u32, count: u32) -> i64 {
    let file_path = "/etc/subuid";
    let mut ranges = match read_subordinate_ranges(file_path) {
        Ok(ranges) => ranges,
        Err(_) => return -1,
    };

    match find_free_range(&mut ranges, min, max, count) {
        Some(start) => start as i64,
        None => -1,
    }
}

///
pub fn sub_gid_find_free_range(min: u32, max: u32, count: u32) -> i64 {
    let file_path = "/etc/subgid";
    let mut ranges = match read_subordinate_ranges(file_path) {
        Ok(ranges) => ranges,
        Err(_) => return -1,
    };

    match find_free_range(&mut ranges, min, max, count) {
        Some(start) => start as i64,
        None => -1,
    }
}

///
pub fn local_sub_uid_assigned(owner: &str, subuid_db_file: String) -> bool {
    let ranges = match read_subordinate_ranges(subuid_db_file.as_str()) {
        Ok(ranges) => ranges,
        Err(_) => return false,
    };

    for range in ranges.iter() {
        if range.owner == owner {
            return true;
        }
    }

    false
}

///
pub fn local_sub_gid_assigned(owner: &str, subgid_db_file: String) -> bool {
    let ranges = match read_subordinate_ranges(subgid_db_file.as_str()) {
        Ok(ranges) => ranges,
        Err(_) => return false,
    };

    for range in ranges.iter() {
        if range.owner == owner {
            return true;
        }
    }
    false
}
