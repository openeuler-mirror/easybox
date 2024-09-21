//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use libc::uid_t;
use nix::unistd::{Uid, User};
use uucore::error::{UResult, USimpleError};

use crate::utils::getdef::*;

///
pub fn get_ranges(sys_user_flag: bool) -> UResult<(uid_t, uid_t)> {
    let mut uid_min = 1000;
    let mut uid_max = 60000;
    uid_min = getdef_ulong("UID_MIN", uid_min);
    uid_max = getdef_ulong("UID_MAX", uid_max);
    if uid_min > uid_max {
        return Err(USimpleError::new(
            1,
            format!(
                "Invalid configuration: UID_MIN ({}), UID_MAX ({})",
                uid_min, uid_max
            ),
        )
        .into());
    }
    if sys_user_flag {
        uid_min = getdef_ulong("SYS_UID_MIN", 101);
        uid_max = getdef_ulong("SYS_UID_MAX", 999);

        if uid_min > uid_max {
            return Err(USimpleError::new(
                1,
                format!(
                    "Invalid configuration: SYS_UID_MIN ({}), SYS_UID_MAX ({})",
                    uid_min, uid_max
                ),
            )
            .into());
        }
    }

    Ok((uid_min, uid_max))
}

///
pub fn find_next_available_uid(
    uid_min: uid_t,
    uid_max: uid_t,
    used_uids: &HashSet<uid_t>,
) -> UResult<uid_t> {
    if let Some(&max_uid) = used_uids.iter().max() {
        let next_uid = max_uid + 1;
        if next_uid <= uid_max && !used_uids.contains(&next_uid) {
            return Ok(next_uid);
        }
    }

    for uid in uid_min..=uid_max {
        if !used_uids.contains(&uid) {
            return Ok(uid);
        }
    }

    Err(USimpleError::new(1, "No unused UIDs available").into())
}

///
pub fn find_next_available_system_uid(
    uid_min: uid_t,
    uid_max: uid_t,
    used_uids: &HashSet<uid_t>,
) -> UResult<uid_t> {
    for uid in (uid_min..=uid_max).rev() {
        if !used_uids.contains(&uid) {
            return Ok(uid);
        }
    }

    Err(USimpleError::new(1, "No unused system UIDs available").into())
}

///
fn get_passwd_file_path(
    chroot_dir: Option<String>,
    prefix_dir: Option<String>,
    file_name: &str,
    after_chroot: bool,
) -> PathBuf {
    let mut path = if after_chroot {
        Path::new(file_name).to_path_buf()
    } else {
        let mut path = PathBuf::new();

        if let Some(chroot) = chroot_dir {
            path.push(chroot);
        }

        if let Some(prefix) = prefix_dir {
            path.push(prefix.strip_prefix("/").unwrap_or(&prefix));
        }

        path.push(file_name.strip_prefix("/").unwrap_or(file_name));
        path
    };

    if !path.is_absolute() {
        path = Path::new("/").join(path);
    }

    path
}

///
pub fn get_used_uids_and_uid_names(
    chroot_dir: Option<String>,
    prefix_dir: Option<String>,
    after_chroot: bool,
    uid_min: uid_t,
    uid_max: uid_t,
) -> UResult<(HashSet<uid_t>, HashSet<String>)> {
    let mut used_uids = HashSet::new();
    let mut uid_names = HashSet::new();
    let uid_file_path = get_passwd_file_path(chroot_dir, prefix_dir, "/etc/passwd", after_chroot);

    if !uid_file_path.exists() {
        return Err(USimpleError::new(
            1,
            format!("Group file path {:?} does not exist", uid_file_path),
        )
        .into());
    }

    match File::open(&uid_file_path) {
        Ok(uid_file) => {
            let reader = BufReader::new(uid_file);

            for line in reader.lines() {
                let line = line?;
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 2 {
                    if let Ok(uid) = parts[2].parse::<uid_t>() {
                        if uid >= uid_min && uid <= uid_max {
                            used_uids.insert(uid);
                            if let Some(name) = parts.get(0) {
                                uid_names.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(USimpleError::new(
                1,
                format!("Error opening uid file {:?}: {}", uid_file_path, e),
            )
            .into());
        }
    }

    Ok((used_uids, uid_names))
}

///
pub fn find_new_uid(
    chroot_dir: Option<String>,
    prefix_dir: Option<String>,
    sys_uid_flag: bool,
    preferred_uid: Option<uid_t>,
    after_chroot: bool,
) -> UResult<uid_t> {
    let (uid_min, uid_max) = get_ranges(sys_uid_flag)?;
    let (used_uids, _) =
        get_used_uids_and_uid_names(chroot_dir, prefix_dir, after_chroot, uid_min, uid_max)?;

    if let Some(preferred_uid) = preferred_uid {
        if User::from_uid(Uid::from_raw(preferred_uid))
            .unwrap()
            .is_some()
        {
            return Err(USimpleError::new(1, "Preferred UID is already in use").into());
        }
    }

    if !sys_uid_flag {
        find_next_available_uid(uid_min, uid_max, &used_uids)
    } else {
        find_next_available_system_uid(uid_min, uid_max, &used_uids)
    }
}
