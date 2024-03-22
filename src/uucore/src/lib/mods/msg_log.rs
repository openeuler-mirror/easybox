//! This file provides some common methods for logging.
//
// (c)  huangduirong <huangduirong@huawei.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::io::Write;

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
