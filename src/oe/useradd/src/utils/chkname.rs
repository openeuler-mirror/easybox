//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use nix::unistd::{sysconf, SysconfVar};

const LOGIN_NAME_MAX: usize = 256;
const GROUP_NAME_MAX_LENGTH: usize = 32;

///
pub fn is_valid_name(name: &str, is_badname: bool) -> bool {
    if is_badname {
        return true;
    }

    if name.is_empty() || name == "." || name == ".." {
        return false;
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphanumeric() && first_char != '_' && first_char != '.' {
        return false;
    }

    let mut numeric = first_char.is_digit(10);
    for (i, c) in name.chars().enumerate().skip(1) {
        if !c.is_alphanumeric()
            && c != '_'
            && c != '.'
            && c != '-'
            && (c != '$' || i != name.len() - 1)
        {
            return false;
        }
        numeric &= c.is_digit(10);
    }

    !numeric
}

/// Check if the username is valid
pub fn is_valid_user_name(name: &str, is_badname: bool) -> bool {
    let maxsize = match sysconf(SysconfVar::LOGIN_NAME_MAX) {
        Ok(Some(size)) => size as usize,
        Ok(None) | Err(_) => LOGIN_NAME_MAX,
    };

    if name.len() >= maxsize as usize {
        return false;
    }

    is_valid_name(name, is_badname)
}

///
pub fn is_valid_group_name(name: &str, is_badname: bool) -> bool {
    if GROUP_NAME_MAX_LENGTH > 0 && name.len() > GROUP_NAME_MAX_LENGTH {
        return false;
    }

    is_valid_name(name, is_badname)
}
