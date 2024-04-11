//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
#![allow(deprecated)] // Disable ENOATTR warning

use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::Path,
};

use errno::Errno;
use extattr::{
    getxattr, lgetxattr, listxattr, llistxattr, lremovexattr, lsetxattr, removexattr, setxattr,
    Flags,
};
use libc::{ENOATTR, ENOTSUP, EXIT_FAILURE};
use uucore::error::{UResult, USimpleError};

use crate::attr_common::Config;

const MAXNAMELEN: usize = 256;
const USER_NAME: &str = "user.";
const SECURE_NAME: &str = "security.";
const TRUSTED_NAME: &str = "trusted.";
const XFSROOT_NAME: &str = "xfsroot.";

/// Based on attr_set() in libattr.c
pub fn attr_set(config: &Config, attrvalue: &Vec<u8>) -> UResult<()> {
    let mut res: Result<(), Errno> = Err(Errno(0));
    let lflags = Flags::empty();
    for compat in 0..2 {
        let name = api_convert(config, compat)?;
        if config.follow {
            res = setxattr(
                <String as AsRef<Path>>::as_ref(&config.filename),
                name,
                attrvalue,
                lflags,
            );
        } else {
            res = lsetxattr(
                <String as AsRef<Path>>::as_ref(&config.filename),
                name,
                attrvalue,
                lflags,
            );
        }
        match res {
            Err(Errno(ENOATTR)) => continue,
            Err(Errno(ENOTSUP)) => continue,
            _ => break,
        };
    }
    match res {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!(
                "attr_set: {}\nCould not set \"{}\" for {}",
                err.to_string(),
                config.attrname,
                config.filename
            );
            Err(USimpleError::new(EXIT_FAILURE, ""))
        }
    }
}

/// Based on attr_get() in libattr.c
pub fn attr_get(config: &Config) -> UResult<Vec<u8>> {
    let mut res: Result<Vec<u8>, Errno> = Err(Errno(0));
    for compat in 0..2 {
        let name = api_convert(config, compat)?;
        if config.follow {
            res = getxattr(<String as AsRef<Path>>::as_ref(&config.filename), name);
        } else {
            res = lgetxattr(<String as AsRef<Path>>::as_ref(&config.filename), name);
        }
        match res {
            Err(Errno(ENOATTR)) => continue,
            Err(Errno(ENOTSUP)) => continue,
            _ => break,
        };
    }
    match res {
        Ok(attrvalue) => Ok(attrvalue),
        Err(err) => {
            eprintln!(
                "attr_get: {}\nCould not get \"{}\" for {}",
                err.to_string(),
                config.attrname,
                config.filename
            );
            Err(USimpleError::new(EXIT_FAILURE, ""))
        }
    }
}

/// Based on attr_remove() in libattr.c
pub fn attr_remove(config: &Config) -> UResult<()> {
    let mut res: Result<(), Errno> = Err(Errno(0));
    for compat in 0..2 {
        let name = api_convert(config, compat)?;
        if config.follow {
            res = removexattr(<String as AsRef<Path>>::as_ref(&config.filename), name);
        } else {
            res = lremovexattr(<String as AsRef<Path>>::as_ref(&config.filename), name);
        }
        match res {
            Err(Errno(ENOATTR)) => continue,
            Err(Errno(ENOTSUP)) => continue,
            _ => break,
        };
    }
    match res {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!(
                "attr_remove: {}\nCould not remove \"{}\" for {}",
                err.to_string(),
                config.attrname,
                config.filename
            );
            Err(USimpleError::new(EXIT_FAILURE, ""))
        }
    }
}

/// Based on attr_list() in libattr.c
pub fn attr_list(config: &Config) -> UResult<Vec<(OsString, usize)>> {
    let res: Result<Vec<OsString>, Errno>;
    if config.follow {
        res = listxattr(<String as AsRef<Path>>::as_ref(&config.filename));
    } else {
        res = llistxattr(<String as AsRef<Path>>::as_ref(&config.filename));
    }
    if let Err(err) = res {
        eprintln!(
            "attr_list: {}\nCould not list {}",
            err.to_string(),
            config.filename
        );
        return Err(USimpleError::new(EXIT_FAILURE, ""));
    }
    let mut alist: Vec<(OsString, usize)> = Vec::new();
    for attrname in res.unwrap() {
        if let Ok(name) = api_unconvert(config, attrname.as_os_str()) {
            let res_get: Result<Vec<u8>, Errno>;
            if config.follow {
                res_get = getxattr(<String as AsRef<Path>>::as_ref(&config.filename), attrname);
            } else {
                res_get = lgetxattr(<String as AsRef<Path>>::as_ref(&config.filename), attrname);
            }
            if let Ok(val) = res_get {
                alist.push((name, val.len()));
            }
        }
    }
    Ok(alist)
}

/*
 * Convert IRIX API components into Linux/XFS API components,
 * and vice-versa.
 */
fn api_convert(config: &Config, compat: i8) -> UResult<String> {
    if config.attrname.len() >= MAXNAMELEN {
        return Err(USimpleError::new(EXIT_FAILURE, "Todo"));
    }
    let mut name: String;
    if config.rootflag {
        if compat == 1 {
            name = XFSROOT_NAME.to_string();
        } else {
            name = TRUSTED_NAME.to_string();
        }
    } else if config.secureflag {
        name = SECURE_NAME.to_string();
    } else {
        name = USER_NAME.to_string();
    }
    name += &config.attrname;
    Ok(name)
}

fn api_unconvert(config: &Config, linuxname: &OsStr) -> Result<OsString, ()> {
    #[allow(non_camel_case_types)]
    #[derive(PartialEq)]
    enum ATTRTYPE {
        ATTR_USER,
        ATTR_SECURE,
        ATTR_ROOT,
    }
    let bytes_name = linuxname.as_bytes();
    let mut find_iter = bytes_name.splitn(2, |n| *n == b'.');
    if let Some(prefix) = find_iter.next() {
        let str_prefix = std::str::from_utf8(prefix).unwrap_or_default();
        let len_add_one = str_prefix.len() + 1; // Add the last '.'
        let attr_type: ATTRTYPE;
        if len_add_one == USER_NAME.len() && USER_NAME.starts_with(str_prefix) {
            attr_type = ATTRTYPE::ATTR_USER;
        } else if len_add_one == SECURE_NAME.len() && SECURE_NAME.starts_with(str_prefix) {
            attr_type = ATTRTYPE::ATTR_SECURE;
        } else if len_add_one == TRUSTED_NAME.len() && TRUSTED_NAME.starts_with(str_prefix) {
            attr_type = ATTRTYPE::ATTR_ROOT;
        } else if len_add_one == XFSROOT_NAME.len() && XFSROOT_NAME.starts_with(str_prefix) {
            attr_type = ATTRTYPE::ATTR_ROOT;
        } else {
            return Err(());
        }
        // Found:
        if config.secureflag && attr_type != ATTRTYPE::ATTR_SECURE {
            return Err(());
        }
        if config.rootflag && attr_type != ATTRTYPE::ATTR_ROOT {
            return Err(());
        }
        return Ok(OsString::from_vec(
            find_iter.next().unwrap_or_default().to_vec(),
        ));
    };
    Err(())
}
