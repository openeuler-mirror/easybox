//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, Error, ErrorKind};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::{fs, thread::available_parallelism};
///
pub const DISKSTATS: &str = "/proc/diskstats";
///
pub const UPTIME: &str = "/proc/uptime";
///
pub const STAT: &str = "/proc/stat";
///
pub const SYSFS_BLOCK: &str = "/sys/block";
///
pub const DEV_DISK_BY: &str = "/dev/disk/by";
///
pub const DEVMAP_DIR: &str = "/dev/mapper";
///
pub const C_BOLD_RED: &str = "\x1b[31;1m";
///
pub const C_LIGHT_GREEN: &str = "\x1b[32;22m";
///
pub const C_BOLD_MAGENTA: &str = "\x1b[35;1m";
///
pub const C_BOLD_BLUE: &str = "\x1b[34;1m";
///
pub const C_LIGHT_BLUE: &str = "\x1b[34;22m";
///
pub const C_NORMAL: &str = "\x1b[0m";

///
pub const SC_PERCENT_WARN: &str = C_BOLD_MAGENTA;
///
pub const SC_PERCENT_XTREME: &str = C_BOLD_RED;
///
pub const SC_ZERO_INT_STAT: &str = C_LIGHT_BLUE;
///
pub const SC_INT_STAT: &str = C_BOLD_BLUE;
///
pub const SC_ITEM_NAME: &str = C_LIGHT_GREEN;
///
pub const SC_NORMAL: &str = C_NORMAL;

///
pub const PERCENT_LIMIT_XHIGH: f64 = 90.0;
///
pub const PERCENT_LIMIT_HIGH: f64 = 75.0;
///
pub const PERCENT_LIMIT_XLOW: f64 = 25.0;
///
pub const PERCENT_LIMIT_LOW: f64 = 10.0;

///
pub const LEFT_BRACE: &str = "{";
///
pub const RIGHT_BRACE: &str = "}";

const MINORBITS: u64 = 20;
const S_MAXMINOR: u64 = (1 << MINORBITS) - 1;

pub fn read_uptime() -> Result<u64, io::Error> {
    let path = Path::new(UPTIME);

    let file = File::open(&path)?;
    let mut line = String::new();
    io::BufReader::new(file).read_line(&mut line)?;

    let mut parts = line.split('.');
    let up_sec: u64 = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid format"))
        .and_then(|s| s.parse().map_err(|e| Error::new(ErrorKind::InvalidData, e)))?;
    let up_cent: u64 = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid format"))
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .unwrap_or("0")
                .parse()
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        })?;

    Ok(up_sec * 100 + up_cent)
}

///
pub fn file_exists(file_path: &str) -> bool {
    if let Ok(_) = fs::metadata(file_path) {
        true
    } else {
        false
    }
}

///
pub fn get_cpu_nr() -> usize {
    return available_parallelism().unwrap().get();
}

///
pub fn sp_value(m: u64, n: u64, p: u64) -> f64 {
    ((n as f64 - m as f64) / p as f64) * 100.0
}

///
pub fn ll_sp_value(value1: u64, value2: u64, itv: u64) -> f64 {
    // println!("{}, {}, {}", value1, value2, itv);
    if value2 < value1 {
        0.0
    } else {
        sp_value(value1, value2, itv)
    }
}

///
pub fn cprintf_xpc(
    human: bool,
    xtrem: Option<&str>,
    num: usize,
    wi: usize,
    wd: i32,
    values: &[f64],
) {
    let mut lim = 0.005;
    let mut width = wi;
    let mut decimal_places = wd as usize;

    if human {
        if width < 4 {
            width = 4;
        }
        width -= 1;
        if decimal_places > 1 {
            decimal_places -= 1;
        }
    }

    if decimal_places == 1 {
        lim = 0.05;
    }

    for &val in values.iter().take(num) {
        let color = if let Some(xtrem) = xtrem {
            match xtrem {
                "XHIGH" if val >= PERCENT_LIMIT_XHIGH => SC_PERCENT_XTREME,
                "XHIGH" if val >= PERCENT_LIMIT_HIGH => SC_PERCENT_WARN,
                "XLOW" if val <= PERCENT_LIMIT_XLOW => SC_PERCENT_XTREME,
                "XLOW0" if val <= PERCENT_LIMIT_XLOW && val >= lim => SC_PERCENT_XTREME,
                "XLOW" if val <= PERCENT_LIMIT_LOW => SC_PERCENT_WARN,
                "XLOW0" if val <= PERCENT_LIMIT_LOW && val >= lim => SC_PERCENT_WARN,
                _ => {
                    if (decimal_places > 0 && val < lim) || (decimal_places == 0 && val <= 0.5) {
                        SC_ZERO_INT_STAT
                    } else {
                        SC_INT_STAT
                    }
                }
            }
        } else {
            if (decimal_places > 0 && val < lim) || (decimal_places == 0 && val <= 0.5) {
                SC_ZERO_INT_STAT
            } else {
                SC_INT_STAT
            }
        };

        let mut output = String::new();
        write!(
            &mut output,
            " {:width$.decimal$}",
            val,
            width = width,
            decimal = decimal_places
        )
        .unwrap();
        // let colored_output = colour_str(color, &output);
        print!("{}{}", color, output);
        print!("{}", SC_NORMAL);
        if human {
            print!("%");
        }
    }
}

///
pub fn get_interval(prev_uptime: u64, curr_uptime: u64) -> u64 {
    let mut itv = curr_uptime - prev_uptime;
    // Paranoia checking
    if itv == 0 {
        itv = 1;
    }

    itv
}

///
pub fn toggle(value: usize) -> usize {
    match value {
        0 => 1,
        1 => 0,
        _ => value,
    }
}

///
pub fn s_value(m: u64, n: u64, p: u64) -> f64 {
    ((n as f64 - m as f64) / p as f64) * 100.0
}

///
pub fn cprintf_f(unit: bool, sign: bool, num: usize, wi: usize, wd: i32, values: &[f64]) {
    let mut lim = 0.005;
    if wd == 1 {
        lim = 0.05;
    }

    for &val in values.iter().take(num) {
        let color = if (wd > 0 && val < lim && val > -lim) || (wd == 0 && val <= 0.5 && val >= -0.5)
        {
            SC_ZERO_INT_STAT
        } else if sign && val <= -10.0 {
            SC_PERCENT_XTREME
        } else if sign && val <= -5.0 {
            SC_PERCENT_WARN
        } else {
            SC_INT_STAT
        };

        if unit {
            print!("{} ", color);
            cprintf_unit(2, wi, val);
        } else {
            let mut output = String::new();
            if sign {
                write!(
                    &mut output,
                    " {:+width$.decimal$}",
                    val,
                    width = wi,
                    decimal = wd as usize
                )
                .unwrap();
            } else {
                write!(
                    &mut output,
                    " {:width$.decimal$}",
                    val,
                    width = wi,
                    decimal = wd as usize
                )
                .unwrap();
            }
            print!("{}{}{}", color, output, SC_NORMAL);
        }
    }
}

///
pub fn cprintf_unit(mut unit: usize, mut wi: usize, mut dval: f64) {
    if wi < 4 {
        wi = 4;
    }

    while dval >= 1024.0 {
        dval /= 1024.0;
        unit += 1;
    }

    let dplaces_nr = 1;
    print!(
        "{:width$.precision$}",
        dval,
        width = wi - 1,
        precision = if dplaces_nr > 0 { 1 } else { 0 }
    );
    print!("{}", SC_NORMAL);
    let units = ['s', 'B', 'k', 'M', 'G', 'T', 'P', '?'];

    if unit >= units.len() {
        unit = units.len() - 1;
    }
    print!("{}", units[unit]);
}

///
pub fn cprintf_u64(unit: bool, num: usize, wi: usize, values: &[u64]) {
    for &val in values.iter().take(num) {
        let color = if val == 0 {
            SC_ZERO_INT_STAT
        } else {
            SC_INT_STAT
        };

        if unit {
            print!("{} ", color);
            cprintf_unit(2, wi, val as f64);
        } else {
            let mut output = String::new();
            write!(&mut output, " {:width$}", val, width = wi).unwrap();
            print!("{}{}{}", color, output, SC_NORMAL);
        }
    }
}

///
pub fn get_persistent_type_dir(dir_type: String) -> Result<String, io::Error> {
    let path = format!("{}-{}", DEV_DISK_BY, dir_type);
    if file_exists(&path) {
        Ok(path)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Not found"))
    }
}

pub fn get_major_minor_nr(filename: &str) -> Result<(u32, u32), io::Error> {
    let mut dfile = if filename.starts_with('/') {
        filename.to_string()
    } else {
        format!("/dev/{}", filename)
    };

    dfile = dfile.replace('!', "/");

    let metadata = fs::metadata(&dfile)?;

    let st_rdev = metadata.rdev();
    let major = (st_rdev >> 8) as u32;
    let minor = (st_rdev & 0xff) as u32;

    Ok((major, minor))
}

pub fn get_devmapname(device_name: &str) -> Option<String> {
    let entries = fs::read_dir(DEVMAP_DIR).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_symlink() {
            continue;
        }

        let target_path = fs::read_link(&path).ok()?;
        let target = target_path.file_name()?.to_str()?;

        if target == device_name {
            return path.file_name()?.to_str().map(String::from);
        }
    }

    None
}

pub fn transform_devmapname(major: u32, minor: u32) -> Option<String> {
    let dm_dir = match fs::read_dir(DEVMAP_DIR) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Cannot open {}: {}", DEVMAP_DIR, e);
            return None;
        }
    };

    for entry in dm_dir.flatten() {
        let path = entry.path();
        if let Ok(metadata) = entry.metadata() {
            let st_rdev = metadata.rdev();
            let dm_major = (st_rdev >> MINORBITS) as u32;
            let dm_minor = (st_rdev & S_MAXMINOR) as u32;

            if dm_major == major && dm_minor == minor {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        return Some(name_str.to_string());
                    }
                }
            }
        }
    }

    None
}

///
pub fn return_tab(tab: usize) -> String {
    let mut output = String::new();
    for _ in 0..tab {
        output += "\t";
    }
    return output;
}
