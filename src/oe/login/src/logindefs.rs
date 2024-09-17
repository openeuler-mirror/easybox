//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::{
    cell::RefCell,
    cmp::min,
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::exit,
    str::from_utf8,
};

use libc::{EXIT_FAILURE, LOG_ALERT};
use nix::{
    fcntl::{self, OFlag},
    sys::stat::{stat, Mode},
    unistd::{self, getegid, getuid, setuid, Uid, User},
    NixPath,
};

use crate::{
    login_unsafe::{setregid_wrapper, setreuid_wrapper, syslog_wrapper},
    utils::strtoul_auto,
};
const _PATH_HUSHLOGINS: &str = "/etc/hushlogins";
const _PATH_HUSHLOGIN: &str = ".hushlogin";
const _PATH_LOGINDEFS: &str = "/etc/login.defs";

/// login.defs item struct
#[derive(Debug, Clone)]
struct Item {
    /* name of the option.  */
    pub name: String,
    /* value of the option.  */
    pub value: String,
}

thread_local! {
    #[allow(non_upper_case_globals)]
    static list: RefCell<Vec<Item>> = RefCell::new(Vec::new());
}

/// Read number from logindefs
pub fn getlogindefs_num(name: &str, dflt: u32) -> u32 {
    let item = search(name, false);
    if item.is_none() || item.as_ref().unwrap().value.is_empty() {
        return dflt;
    }
    let item = item.unwrap();
    let retval = strtoul_auto(&item.value);
    return retval.unwrap_or_else(|_| {
        syslog_wrapper(
            libc::LOG_NOTICE,
            &format!(
                "{}: {} contains invalid numerical value: {}",
                _PATH_LOGINDEFS, name, item.value
            ),
        );
        dflt
    });
}

/// Read bool from logindefs
pub fn getlogindefs_bool(name: &str, dflt: bool) -> bool {
    let item = search(name, false);
    if let Some(itv) = item {
        if itv.value.is_empty() == false {
            return itv.value.to_lowercase() == "yes";
        }
    }
    dflt
}

/// Support hushlogin related function
pub fn get_hushlogin_status(pwd: &User) -> bool {
    const HUSHLOGIN_FILE: &str = "HUSHLOGIN_FILE";
    let conf = getlogindefs_str(HUSHLOGIN_FILE, "");
    let mut files: Vec<&str> = vec![&conf, _PATH_HUSHLOGIN];
    if search(HUSHLOGIN_FILE, false).is_some() {
        if files[0].is_empty() {
            return false; /* empty HUSHLOGIN_FILE defined */
        }
        files.pop();
    } else {
        files[0] = _PATH_HUSHLOGINS;
    }
    let pwd_shell = String::from(pwd.shell.to_string_lossy());

    for file in files {
        let mut ok = false;
        /* global hush-file */
        if file.starts_with('/') {
            if let Ok(st) = stat(file) {
                if st.st_size == 0 {
                    return true; /* for all accounts */
                }
                if let Ok(f) = File::options().read(true).open(file) {
                    let mut reader = BufReader::new(f);
                    let mut one_line = String::new();
                    while reader.read_line(&mut one_line).is_ok() {
                        one_line.pop();
                        if one_line
                            == *match one_line.starts_with('/') {
                                true => &pwd_shell,
                                false => &pwd.name,
                            }
                        {
                            return true; /* found username/shell */
                        }
                    }
                    return false; /* ignore per-account files */
                } else {
                    continue; /* ignore errors... */
                }
            } else {
                continue; /* file does not exist */
            }
        }

        /* per-account setting */
        let home_dir_file = pwd.dir.join(file);

        let ruid = getuid();
        let egid = getegid();

        if setregid_wrapper(u32::MAX, pwd.gid.as_raw()).is_ok()
            && setreuid_wrapper(0, pwd.uid.as_raw()).is_ok()
        {
            ok = effective_access(&home_dir_file, OFlag::O_RDONLY);
        }

        if setuid(Uid::from_raw(0)).is_err()
            || setreuid_wrapper(ruid.into(), 0).is_err()
            || setregid_wrapper(u32::MAX, egid.into()).is_err()
        {
            syslog_wrapper(LOG_ALERT, "hush login status: restore original IDs failed");
            exit(EXIT_FAILURE);
        }
        if ok {
            return true;
        }
    }
    false
}

/*
 * Returns:
 *	@dflt		if @name not found
 *	String::new()		(empty string) if found, but value not defined
 *	"string"	if found
 */
/// Read string from logindefs
pub fn getlogindefs_str(name: &str, dflt: &str) -> String {
    let item = search(name, false);
    if let Some(iv) = item {
        return iv.value;
    }
    return dflt.to_string();
}

/// Read string from logindefs then set environment variable
pub fn logindefs_setenv(name: &str, conf: &str, dflt: Option<&str>) -> bool {
    let mut val = String::new();
    if let Some(dfv) = dflt {
        val = getlogindefs_str(conf, dfv);
    } else if search(conf, false).is_none() {
        // dflt is NULL and no conf exist, failed
        return false;
    }
    if let Some((_, v)) = val.split_once('=') {
        val = v.to_string();
    }
    env::set_var(name, val);
    true
}

/// Load logindefs from specified path
pub fn load_defaults<P: AsRef<Path>>(path: P) {
    if let Ok(f) = File::options().read(true).open(path) {
        let mut reader = BufReader::new(f);
        let mut one_line = String::new();
        while {
            one_line.clear();
            reader.read_line(&mut one_line).unwrap_or(0)
        } > 0
        {
            let buf = one_line.as_bytes();
            if buf.is_empty() || buf[0] == b'#' || buf[0] == b'\n' {
                continue; /* only comment or empty line */
            }

            let buf = buf.split(|x| *x == b'#' || *x == b'\n').next().unwrap();
            if buf.is_empty() {
                continue; /* empty line */
            }

            /* ignore space at begin of the line */
            let mut name_start = 0;
            for i in 0..buf.len() {
                if buf[i].is_ascii_whitespace() == false {
                    name_start = i;
                    break;
                }
            }

            /* go to the end of the name */
            let mut name_end = name_start;
            for i in name_start..buf.len() {
                if buf[i].is_ascii_whitespace() || buf[i] == b'=' {
                    break;
                }
                name_end = i + 1;
            }
            if name_end == name_start {
                continue;
            }

            /* go to the begin of the value */
            let mut data_start = min(name_end + 1, buf.len());
            for i in data_start..buf.len() {
                if buf[i].is_ascii_whitespace() == false && buf[i] != b'=' && buf[i] != b'"' {
                    data_start = i;
                    break;
                }
            }

            /* remove space at the end of the value */
            let mut data_end = buf.len();
            for i in (data_start..buf.len()).rev() {
                if buf[i].is_ascii_whitespace() == false && buf[i] != b'"' {
                    data_end = i + 1;
                    break;
                }
            }

            store(
                from_utf8(&buf[name_start..name_end]).unwrap(),
                from_utf8(&buf[data_start..data_end]).unwrap(),
            );
        }
    }
}

/*
 * We need to check the effective UID/GID.
 */
fn effective_access<P: NixPath>(path: &P, mode: OFlag) -> bool {
    if let Ok(fd) = fcntl::open(path, mode, Mode::empty()) {
        unistd::close(fd).ok();
        return true;
    }
    return false;
}

fn store(name: &str, value: &str) {
    let new_item = Item {
        name: name.to_string(),
        value: value.to_string(),
    };
    list.with(|listv| listv.borrow_mut().push(new_item));
}

fn search(name: &str, is_second_call: bool) -> Option<Item> {
    let mut need_load = false;
    let mut ret = None;
    list.with(|listv| {
        let listvec = listv.borrow();
        if listvec.is_empty() {
            need_load = true;
            return;
        }
        for item in listvec.iter() {
            // item.name always in uppercase
            if item.name == name.to_uppercase() {
                ret = Some(item.to_owned());
                return;
            }
        }
    });
    if need_load {
        load_defaults(_PATH_LOGINDEFS);
        if !is_second_call {
            return search(name, true);
        }
    }
    ret
}

/// Used in test suite
pub fn dump_list() -> Vec<u8> {
    let mut res = Vec::new();
    list.with(|listv| {
        let listvec = listv.borrow();
        for i in listvec.iter() {
            let val = match i.value.is_empty() {
                true => "(null)",
                false => &i.value,
            };
            writeln!(&mut res, "${}: '{}'", i.name, val).unwrap();
        }
    });
    res
}
