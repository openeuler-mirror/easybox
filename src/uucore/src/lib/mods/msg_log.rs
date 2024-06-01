//! This file provides some common methods for logging.
//
// (c)  huangduirong <huangduirong@huawei.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::util_name;
use errno::errno;
use std::{io::Write, process};

#[allow(dead_code)]
pub fn errmsg(doexit: bool, excode: i32, adderr: bool, fmt: &str) {
    println!("{}: ", std::env::args().nth(0).unwrap());
    if fmt != "" {
        writeln!(std::io::stderr(), "{}", fmt).unwrap();
        if adderr {
            write!(std::io::stderr(), ": ").unwrap();
            eprintln!("{}", std::io::Error::last_os_error());
        }
    } else if adderr {
        eprintln!("{}", std::io::Error::last_os_error());
    }
    if doexit {
        std::process::exit(excode);
    }
}

pub fn errmsg_custom(doexit: bool, excode: i32, adderr: bool, fmt: &str) {
    eprint!("{}: ", util_name());
    if fmt != "" {
        writeln!(std::io::stderr(), "{}", fmt).unwrap();
        if adderr {
            write!(std::io::stderr(), ": ").unwrap();
            eprintln!("{}", std::io::Error::last_os_error());
        }
    } else if adderr {
        eprintln!("{}", std::io::Error::last_os_error());
    }
    if doexit {
        std::process::exit(excode);
    }
}

/// Equals to `errmsg()` in `c.h`.
#[allow(dead_code)]
pub fn errmsg_c(doexit: bool, excode: i32, adderr: bool, fmt: &str) {
    eprint!("{}: ", util_name());
    if fmt != "" {
        eprint!("{}", fmt);
        if adderr {
            eprint!(": ");
        }
    }
    if adderr {
        eprint!("{}", errno());
    }
    eprintln!("");
    if doexit {
        process::exit(excode);
    }
}

#[allow(dead_code)]
pub fn err(e: i32, fmt: &str) {
    errmsg(true, e, true, fmt);
}

#[allow(dead_code)]
pub fn errx(e: i32, fmt: &str) {
    errmsg(true, e, false, fmt);
}

#[allow(dead_code)]
pub fn warn(fmt: &str) {
    errmsg(false, 0, true, fmt);
}

#[allow(dead_code)]
pub fn warnx(fmt: &str) {
    errmsg(false, 0, false, fmt);
}

#[allow(dead_code)]
/// Print brief warn message without util name and errno.
pub fn warnb(fmt: &str) {
    eprintln!("{}", fmt);
}

/// Equals to `err()` in `c.h`.
#[allow(dead_code)]
pub fn err_c(e: i32, fmt: &str) {
    errmsg_c(true, e, true, fmt);
}

/// Equals to `errx()` in `c.h`.
#[allow(dead_code)]
pub fn errx_c(e: i32, fmt: &str) {
    errmsg_c(true, e, false, fmt);
}

/// Equals to `warn()` in `c.h`.
#[allow(dead_code)]
pub fn warn_c(fmt: &str) {
    errmsg_c(false, 0, true, fmt);
}

/// Equals to `warnx()` in `c.h`.
#[allow(dead_code)]
pub fn warnx_c(fmt: &str) {
    errmsg_c(false, 0, false, fmt);
}

/// Equals to `errtryhelp()` in `c.h`.
#[allow(dead_code)]
pub fn errtryhelp_c(eval: i32) {
    eprintln!("Try '{} --help' for more information.", util_name());
    process::exit(eval);
}
