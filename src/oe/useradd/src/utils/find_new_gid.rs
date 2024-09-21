//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::useradd_common::Paths;

use super::getdef::getdef_ulong;
use libc::gid_t;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use uucore::error::{UResult, USimpleError};

///
pub fn get_ranges(sys_user_flag: bool) -> UResult<(gid_t, gid_t)> {
    let mut gid_min = 1000;
    let mut gid_max = 60000;
    gid_min = getdef_ulong("GID_MIN", gid_min);
    gid_max = getdef_ulong("GID_MAX", gid_max);
    if gid_min > gid_max {
        return Err(USimpleError::new(
            1,
            format!(
                "Invalid configuration: GID_MIN ({}), GID_MAX ({})",
                gid_min, gid_max
            ),
        )
        .into());
    }
    if sys_user_flag {
        gid_min = getdef_ulong("SYS_GID_MIN", 101);
        gid_max = getdef_ulong("SYS_GID_MAX", 999);

        if gid_min > gid_max {
            return Err(USimpleError::new(
                1,
                format!(
                    "Invalid configuration: SYS_GID_MIN ({}), SYS_GID_MAX ({})",
                    gid_min, gid_max
                ),
            )
            .into());
        }
    }

    Ok((gid_min, gid_max))
}

///
pub fn check_gid(
    gid: gid_t,
    min: gid_t,
    max: gid_t,
    used_gids: &HashSet<gid_t>,
    non_unique: bool,
) -> Result<(), i32> {
    if gid < min || gid > max {
        return Err(nix::errno::Errno::ERANGE as i32);
    }
    if !non_unique && used_gids.contains(&gid) {
        return Err(nix::errno::Errno::EEXIST as i32);
    }
    Ok(())
}

///
pub fn normalize_path(path: &str) -> String {
    let mut normalized = path.to_string();
    if normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

///
fn get_used_gids_and_group_names(
    file_path: String,
    sys_group_flag: bool,
) -> UResult<(HashSet<gid_t>, HashSet<String>)> {
    let (gid_min, gid_max) = get_ranges(sys_group_flag)?;

    let mut used_gids = HashSet::new();
    let mut group_names = HashSet::new();

    let group_file_path = Path::new(&file_path).to_path_buf();

    if !group_file_path.exists() {
        return Err(USimpleError::new(
            1,
            format!("Group file path {:?} does not exist", group_file_path),
        )
        .into());
    }

    match File::open(&group_file_path) {
        Ok(group_file) => {
            let reader = BufReader::new(group_file);

            for line in reader.lines() {
                let line = line?;
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 2 {
                    if let Ok(gid) = parts[2].parse::<gid_t>() {
                        if gid >= gid_min && gid <= gid_max {
                            used_gids.insert(gid);
                            if let Some(name) = parts.get(0) {
                                group_names.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(USimpleError::new(
                1,
                format!("Error opening group file {:?}: {}", group_file_path, e),
            )
            .into());
        }
    }

    Ok((used_gids, group_names))
}

///
pub fn find_next_available_gid(
    gid_min: gid_t,
    gid_max: gid_t,
    used_gids: &HashSet<gid_t>,
) -> UResult<gid_t> {
    if let Some(&max_gid) = used_gids.iter().max() {
        let next_gid = max_gid + 1;
        if next_gid <= gid_max && !used_gids.contains(&next_gid) {
            return Ok(next_gid);
        }
    }

    for gid in gid_min..=gid_max {
        if !used_gids.contains(&gid) {
            return Ok(gid);
        }
    }

    Err(USimpleError::new(1, "No unused GIDs available").into())
}

///
pub fn find_next_available_system_gid(
    gid_min: gid_t,
    gid_max: gid_t,
    used_gids: &HashSet<gid_t>,
) -> UResult<gid_t> {
    for gid in (gid_min..=gid_max).rev() {
        if !used_gids.contains(&gid) {
            return Ok(gid);
        }
    }

    Err(USimpleError::new(1, "No unused system GIDs available").into())
}

///
pub fn find_new_gid(
    sys_group_flag: bool,
    preferred_gid: Option<gid_t>,
    force: bool,
    non_unique: bool,
    path: &Paths,
) -> UResult<gid_t> {
    let (gid_min, gid_max) = get_ranges(sys_group_flag)?;
    let (used_gids, _) = get_used_gids_and_group_names(path.group_db_file.clone(), sys_group_flag)?;

    if let Some(preferred_gid) = preferred_gid {
        match check_gid(preferred_gid, gid_min, gid_max, &used_gids, non_unique) {
            Ok(()) => return Ok(preferred_gid),
            Err(_) => {
                if !force {
                    return Err(USimpleError::new(1, "Preferred GID is already in use").into());
                }
            }
        }
    }

    if used_gids.is_empty() {
        return Ok(gid_min);
    }

    if !sys_group_flag {
        find_next_available_gid(gid_min, gid_max, &used_gids)
    } else {
        find_next_available_system_gid(gid_min, gid_max, &used_gids)
    }
}
