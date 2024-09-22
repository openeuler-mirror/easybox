//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use super::{gshadow::Sgrp, shadow::Spwd};
use crate::usermod_common::Config;
use nix::unistd::{Gid, Group, Uid, User};
use std::{
    ffi::CString,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};
use uucore::error::{UResult, USimpleError};

/// invalid argument to option
const E_BAD_ARG: i32 = 3;

/// similar to C code process_prefix_flag
pub fn process_prefix_flag(config: &mut Config) -> UResult<Option<String>> {
    if let Some(ref mut prefix) = config.prefix {
        if nix::unistd::setgid(nix::unistd::getgid()).is_err()
            || nix::unistd::setuid(nix::unistd::getuid()).is_err()
        {
            return Err(USimpleError::new(E_BAD_ARG, "failed to drop privileges").into());
        }

        if prefix.is_empty() || prefix == "/" {
            return Ok(None);
        }

        if !prefix.starts_with('/') {
            return Err(USimpleError::new(E_BAD_ARG, "prefix must be an absolute path").into());
        }

        if prefix.ends_with('/') {
            prefix.pop();
        }

        let path = Path::new(prefix);
        if !path.exists() {
            return Err(USimpleError::new(
                1,
                format!("Prefix directory '{}' does not exist", prefix),
            )
            .into());
        } else if !path.is_dir() {
            return Err(
                USimpleError::new(1, format!("Prefix '{}' is not a directory", prefix)).into(),
            );
        }
        return Ok(Some(prefix.clone()));
    }

    Ok(None)
}

///
pub fn prefix_getgrnam(
    name: &str,
    prefix: Option<String>,
    group_db_file_path: String,
) -> Option<Group> {
    if prefix.is_some() {
        let file = File::open(group_db_file_path).ok()?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line.ok()?;
            if let Some(group) = parse_group_line_by_name(&line, name) {
                return Some(group);
            }
        }
    }

    match Group::from_name(name) {
        Ok(Some(group)) => Some(group),
        Ok(None) => None,
        Err(_) => None,
    }
}

///
pub fn prefix_getgrgid(
    gid: u32,
    prefix: Option<String>,
    group_db_file_path: String,
) -> Option<Group> {
    if prefix.is_some() {
        let file = File::open(group_db_file_path).ok()?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line.ok()?;
            if let Some(group) = parse_group_line_by_gid(&line, Gid::from_raw(gid)) {
                return Some(group);
            }
        }
    }

    match Group::from_gid(Gid::from_raw(gid)) {
        Ok(Some(group)) => Some(group),
        Ok(None) => None,
        Err(_) => None,
    }
}

///
pub fn prefix_getgr_nam_gid(
    grname: &str,
    prefix: Option<String>,
    group_db_file_path: String,
) -> Option<Group> {
    if let Ok(gid) = grname.parse::<u32>() {
        return prefix_getgrgid(gid, prefix, group_db_file_path);
    }
    prefix_getgrnam(grname, prefix, group_db_file_path)
}

///
fn parse_group_line_by_gid(line: &str, gid: Gid) -> Option<Group> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 4 {
        if let Ok(parsed_gid) = parts[2].parse::<u32>() {
            if parsed_gid == gid.as_raw() {
                let name = parts[0].to_string();
                let passwd = CString::new(parts[1]).ok()?;
                let mem = parts[3].split(',').map(|s| s.to_string()).collect();
                return Some(Group {
                    name,
                    passwd,
                    gid,
                    mem,
                });
            }
        }
    }
    None
}

/// Parse a line of the /etc/group file, match the group name and return the corresponding Group structure
fn parse_group_line_by_name(line: &str, name: &str) -> Option<Group> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 4 {
        if parts[0] == name {
            if let Ok(parsed_gid) = parts[2].parse::<u32>() {
                let gid = Gid::from_raw(parsed_gid);
                let passwd = CString::new(parts[1]).ok()?;
                let mem = parts[3].split(',').map(|s| s.to_string()).collect();
                return Some(Group {
                    name: name.to_string(),
                    passwd,
                    gid,
                    mem,
                });
            }
        }
    }
    None
}

///
pub fn prefix_getpwuid(
    uid: u32,
    prefix: Option<String>,
    passwd_db_file_path: String,
) -> Option<User> {
    if let Some(_) = prefix {
        let file = File::open(passwd_db_file_path).ok()?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line.ok()?;
            if let Some(user) = parse_user_line_by_uid(&line, Uid::from_raw(uid)) {
                return Some(user);
            }
        }
    } else {
        return User::from_uid(Uid::from_raw(uid)).ok().flatten();
    }
    None
}

///
pub fn prefix_getpwnam(
    name: &str,
    prefix: Option<String>,
    passwd_db_file_path: String,
) -> Option<User> {
    if let Some(_) = prefix {
        let file = File::open(passwd_db_file_path).ok()?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line.ok()?;
            if let Some(user) = parse_user_line_by_name(&line, name) {
                return Some(user);
            }
        }
    } else {
        return User::from_name(name).ok().flatten();
    }
    None
}

///
fn parse_user_line_by_uid(line: &str, uid: Uid) -> Option<User> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 7 {
        if let Ok(parsed_uid) = parts[2].parse::<u32>() {
            if parsed_uid == uid.as_raw() {
                return Some(parse_user_parts(parts));
            }
        }
    }
    None
}

///
fn parse_user_line_by_name(line: &str, name: &str) -> Option<User> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 7 {
        if parts[0] == name {
            return Some(parse_user_parts(parts));
        }
    }
    None
}

///
fn parse_user_parts(parts: Vec<&str>) -> User {
    let name = parts[0].to_string();
    let passwd = CString::new(parts[1]).unwrap();
    let uid = Uid::from_raw(parts[2].parse::<u32>().unwrap());
    let gid = Gid::from_raw(parts[3].parse::<u32>().unwrap());
    let gecos = CString::new(parts[4]).unwrap();
    let dir = PathBuf::from(parts[5]);
    let shell = PathBuf::from(parts[6]);

    User {
        name,
        passwd,
        uid,
        gid,
        gecos,
        dir,
        shell,
    }
}

///
pub fn prefix_getspnam(
    name: &str,
    _prefix: Option<String>,
    shadow_db_file_path: String,
) -> Option<Spwd> {
    let file = File::open(shadow_db_file_path).ok()?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.ok()?;
        if let Some(spwd) = parse_spwd_line_by_name(&line, name) {
            return Some(spwd);
        }
    }

    None
}

/// Parse a line of the shadow file, match the username and return the corresponding Spwd structure
fn parse_spwd_line_by_name(line: &str, name: &str) -> Option<Spwd> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 9 && parts[0] == name {
        Some(Spwd {
            sp_namp: parts[0].to_string(),
            sp_pwdp: parts[1].to_string(),
            sp_lstchg: parse_optional_i64(parts[2]),
            sp_min: parse_optional_i64(parts[3]),
            sp_max: parse_optional_i64(parts[4]),
            sp_warn: parse_optional_i64(parts[5]),
            sp_inact: parse_optional_i64(parts[6]),
            sp_expire: parse_optional_i64(parts[7]),
            sp_flag: parse_optional_i64(parts[8]),
        })
    } else {
        None
    }
}

/// Parse optional i64 string
fn parse_optional_i64(field: &str) -> Option<i64> {
    if field.is_empty() {
        None
    } else {
        field.parse::<i64>().ok()
    }
}

///
pub fn read_all_groups(group_file_path: &str) -> UResult<Vec<Group>> {
    let path = Path::new(group_file_path);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut groups = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(group) = parse_group_line(&line) {
                groups.push(group);
            }
        }
    }

    Ok(groups)
}

/// Parse a line from the /etc/group file and return a Group structure
pub fn parse_group_line(line: &str) -> Option<Group> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 4 {
        if let Ok(gid) = parts[2].parse::<u32>() {
            let name = parts[0].to_string();
            let passwd = CString::new(parts[1]).ok()?;
            let gid = nix::unistd::Gid::from_raw(gid);
            let mem = parts[3].split(',').map(|s| s.to_string()).collect();
            return Some(Group {
                name,
                passwd,
                gid,
                mem,
            });
        }
    }
    None
}

///
pub fn read_all_spwd(shadow_db_file_path: &str) -> UResult<Vec<Spwd>> {
    let path = Path::new(shadow_db_file_path);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut spwds = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(spwd) = parse_spwd_line(&line) {
                spwds.push(spwd);
            }
        }
    }

    Ok(spwds)
}

///
fn parse_spwd_line(line: &str) -> Option<Spwd> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 9 {
        Some(Spwd {
            sp_namp: parts[0].to_string(),
            sp_pwdp: parts[1].to_string(),
            sp_lstchg: parse_optional_i64(parts[2]),
            sp_min: parse_optional_i64(parts[3]),
            sp_max: parse_optional_i64(parts[4]),
            sp_warn: parse_optional_i64(parts[5]),
            sp_inact: parse_optional_i64(parts[6]),
            sp_expire: parse_optional_i64(parts[7]),
            sp_flag: parse_optional_i64(parts[8]),
        })
    } else {
        None
    }
}

///
pub fn read_all_sgrp(gshadow_db_file_path: &str) -> UResult<Vec<Sgrp>> {
    let path = Path::new(gshadow_db_file_path);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut sgrps = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(sgrp) = parse_sgrp_line(&line) {
                sgrps.push(sgrp);
            }
        }
    }
    Ok(sgrps)
}

///
pub fn parse_sgrp_line(s: &str) -> Option<Sgrp> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() >= 4 {
        Some(Sgrp {
            sg_name: parts[0].to_string(),
            sg_passwd: parts[1].to_string(),
            sg_adm: parts[2].split(',').map(|x| x.to_string()).collect(),
            sg_mem: parts[3].split(',').map(|x| x.to_string()).collect(),
        })
    } else {
        None
    }
}
