//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};

///
pub fn have_range(uid_file: &mut File, owner: &str, start: u32, count: u32) -> io::Result<bool> {
    let reader = BufReader::new(uid_file);
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 3 {
            let file_owner = parts[0];
            let file_start: u32 = parts[1].parse().unwrap_or(0);
            let file_count: u32 = parts[2].parse().unwrap_or(0);
            let file_end = file_start + file_count - 1;
            let query_end = start + count - 1;

            if owner == file_owner && start == file_start && query_end == file_end {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
///
pub fn remove_range(
    uid_file: &mut File,
    owner: String,
    start: u32,
    count: u32,
) -> io::Result<bool> {
    if !have_range(uid_file, &owner, start, count)? {
        return Ok(false);
    }

    uid_file.seek(SeekFrom::Start(0))?;
    let mut reader = BufReader::new(uid_file);
    let mut lines: Vec<String> = Vec::new();

    for line in reader.by_ref().lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 3 {
            let file_owner = parts[0];
            let file_start: u32 = parts[1].parse().unwrap_or(0);
            let file_count: u32 = parts[2].parse().unwrap_or(0);

            let file_end = file_start + file_count - 1;
            let query_end = start + count - 1;

            if !(owner == file_owner && start == file_start && query_end == file_end) {
                lines.push(line);
            }
        }
    }

    reader.get_mut().set_len(0)?;
    reader.get_mut().seek(SeekFrom::Start(0))?;

    for line in lines {
        writeln!(reader.get_mut(), "{}", line)?;
    }

    Ok(true)
}

///
pub fn add_range(uid_file: &mut File, owner: String, start: u32, count: u32) -> io::Result<bool> {
    if have_range(uid_file, &owner, start, count)? {
        return Ok(false);
    }

    writeln!(uid_file, "{}:{}:{}", owner, start, count)?;
    Ok(true)
}

///
pub fn sub_uid_add(uid_file: &mut File, owner: String, start: u32, count: u32) -> io::Result<bool> {
    add_range(uid_file, owner, start, count)
}

///
pub fn sub_uid_remove(
    uid_file: &mut File,
    owner: String,
    start: u32,
    count: u32,
) -> io::Result<bool> {
    remove_range(uid_file, owner, start, count)
}

///
pub fn sub_gid_add(gid_file: &mut File, owner: String, start: u32, count: u32) -> io::Result<bool> {
    add_range(gid_file, owner, start, count)
}

///
pub fn sub_gid_remove(
    gid_file: &mut File,
    owner: String,
    start: u32,
    count: u32,
) -> io::Result<bool> {
    remove_range(gid_file, owner, start, count)
}
