//! This file is part of the easybox package.
//
// (c) Maxwell Xu <eureka0xff@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::{BrokenTime, ClockOperations, HwclockConfig, HWCLOCK_ERROR};
use nix::errno::Errno;
use nix::libc::O_RDONLY;
use nix::sys::select::{select, FdSet};
use nix::sys::time::TimeVal;
use nix::{ioctl_none, ioctl_read, ioctl_write_ptr};
use once_cell::sync::OnceCell;
use std::fs::read_link;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, BorrowedFd};
use std::path::PathBuf;
use std::time::SystemTime;
use uucore::error::{UResult, USimpleError};
use uucore::msg_log::warnx;

/// From linux/rtc.h
const RTC_IOC_MAGIC: u8 = b'p';

const RTC_NR_UIE_ON: u8 = 0x03;
const RTC_NR_UIE_OFF: u8 = 0x04;
const RTC_NR_RD_TIME: u8 = 0x09;
const RTC_NR_SET_TIME: u8 = 0x0a;
const RTC_NR_VL_READ: u8 = 0x13;
const RTC_NR_VL_CLR: u8 = 0x14;
const RTC_NR_PARAM_GET: u8 = 0x13;
const RTC_NR_PARAM_SET: u8 = 0x14;

ioctl_read!(
    rtc_ioctl_read_time,
    RTC_IOC_MAGIC,
    RTC_NR_RD_TIME,
    BrokenTime
);
ioctl_write_ptr!(
    rtc_ioctl_set_time,
    RTC_IOC_MAGIC,
    RTC_NR_SET_TIME,
    BrokenTime
);
ioctl_write_ptr!(
    rtc_ioctl_param_get,
    RTC_IOC_MAGIC,
    RTC_NR_PARAM_GET,
    RtcParam
);
ioctl_write_ptr!(
    rtc_ioctl_param_set,
    RTC_IOC_MAGIC,
    RTC_NR_PARAM_SET,
    RtcParam
);
ioctl_none!(rtc_ioctl_uie_on, RTC_IOC_MAGIC, RTC_NR_UIE_ON);
ioctl_none!(rtc_ioctl_uie_off, RTC_IOC_MAGIC, RTC_NR_UIE_OFF);
ioctl_read!(rtc_ioctl_vl_read, RTC_IOC_MAGIC, RTC_NR_VL_READ, u32);
ioctl_none!(rtc_ioctl_vl_clear, RTC_IOC_MAGIC, RTC_NR_VL_CLR);

/// RTC device
static RTC_DEVICE: OnceCell<File> = OnceCell::new();

/// union member in RTC parameter struct
#[allow(dead_code)]
pub union RtcParamUnion {
    uvalue: u64,
    svalue: i64,
    ptr: u64,
}

/// RTC parameter struct
pub struct RtcParam {
    pub param: u64,
    pub u: RtcParamUnion,
    pub index: u32,
    pub pad: u32,
}

///
pub fn get_permissions_rtc(_config: &HwclockConfig) -> UResult<()> {
    Ok(())
}

/// in intel IA-64 architecture, there's also /dev/efirtc and /dev/misc/efirtc
const RTC_FILE_PATHS: &[&str] = &["/dev/rtc0", "/dev/rtc", "/dev/misc/rtc"];

///
fn get_file_path_from_fd(fd: i32) -> Option<PathBuf> {
    let fd_in_proc = PathBuf::from(format!("/proc/self/fd/{}", fd));
    read_link(fd_in_proc).ok()
}

/// The original get_file_path returns a global variable which stores file path
/// we get the path from fd here
pub fn get_file_path() -> Option<PathBuf> {
    match RTC_DEVICE.get() {
        Some(f) => get_file_path_from_fd(f.as_raw_fd()),
        None => None,
    }
}

/// open the RTC file, return its file descriptor
/// if we have already opened, return its file descriptor
pub fn open_rtc(config: &HwclockConfig, path: Option<&String>) -> i32 {
    let fd = RTC_DEVICE.get().map(|file| file.as_raw_fd());
    if fd.is_some() {
        return fd.unwrap();
    }
    let mut open_options = OpenOptions::new();
    let open_options = open_options.read(true).custom_flags(O_RDONLY);
    match path {
        Some(path_ref) => match open_options.open(path_ref) {
            Ok(f) => {
                let fd = f.as_raw_fd();
                let _ = RTC_DEVICE.set(f);
                fd
            }
            Err(e) => {
                eprintln!("Unable to open {}, {}", path_ref, e.to_string());
                HWCLOCK_ERROR
            }
        },
        None => {
            for &path in RTC_FILE_PATHS {
                if config.verbose_opt {
                    println!("Trying to open {}", path);
                }
                match open_options.open(path) {
                    Ok(f) => {
                        let fd = f.as_raw_fd();
                        let _ = RTC_DEVICE.set(f);
                        return fd;
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::NotFound {
                            continue;
                        }
                        eprintln!("Unable to open {}, {}", path, e.to_string());
                        break;
                    }
                }
            }
            HWCLOCK_ERROR
        }
    }
}

///
pub fn read_hardware_clock_rtc(config: &HwclockConfig) -> UResult<BrokenTime> {
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    let mut rtc_time = BrokenTime::default();
    match unsafe { rtc_ioctl_read_time(fd, &mut rtc_time) } {
        Err(e) => Err(USimpleError::new(
            1,
            format!(
                "ioctl(RTC_RD_TIME) to {} to read the time failed, {}",
                get_file_path_from_fd(fd).unwrap().display(),
                e.desc()
            ),
        )),
        Ok(_) => Ok(rtc_time),
    }
}

///
pub fn set_hardware_clock_rtc(config: &HwclockConfig, time: &BrokenTime) -> UResult<()> {
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    match unsafe { rtc_ioctl_set_time(fd, time) } {
        Ok(_) => {
            if config.verbose_opt {
                println!("ioctl(RTC_SET_TIME) was successful");
            };
            Ok(())
        }
        Err(e) => {
            return Err(USimpleError::new(
                1,
                format!(
                    "ioctl(RTC_SET_TIME) to {} to set the time failed, {}",
                    get_file_path_from_fd(fd).unwrap().display(),
                    e.to_string()
                ),
            ))
        }
    }
}

///
pub fn synchronize_to_clock_tick_rtc(config: &HwclockConfig) -> UResult<()> {
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    match unsafe { rtc_ioctl_uie_on(fd) } {
        Ok(_) => {
            let mut rtc_fdset = FdSet::new();
            let mut tv = TimeVal::new(10, 0);
            // fd will not be -1 here, and we guarantee that fd refers to an open file
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
            rtc_fdset.insert(&borrowed_fd);
            let select_rc = select(
                Some(fd + 1),
                Some(&mut rtc_fdset),
                None,
                None,
                Some(&mut tv),
            );
            let uie_off_rc = unsafe { rtc_ioctl_uie_off(fd) };
            if select_rc.is_ok() && uie_off_rc.is_ok() {
                return Ok(());
            };
            if select_rc.is_err() {
                warnx(&format!(
                    "select() to {} to wait for clock tick failed",
                    get_file_path_from_fd(fd).unwrap().display()
                ));
            }
            if uie_off_rc.is_err() {
                warnx(&format!(
                    "ioctl() to {} to turn off RTC update interrupts failed",
                    get_file_path_from_fd(fd).unwrap().display()
                ));
            }
            Err(USimpleError::new(1, "Synchronize to clock tick failed"))
        }
        Err(e) => {
            if e == Errno::ENOTTY || e == Errno::EINVAL {
                if config.verbose_opt {
                    println!(
                        "ioctl({}, RTC_UIE_ON, 0) to {} failed, {}",
                        fd,
                        get_file_path_from_fd(fd).unwrap().display(),
                        e.desc()
                    );
                    println!("Waiting in loop for time from RTC to change");
                }
                match busywait_for_rtc_clock_tick(fd) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(USimpleError::new(
                        1,
                        format!("Busywait for RTC clock tick failed, {}", e.to_string()),
                    )),
                }
            } else {
                Err(USimpleError::new(
                    1,
                    format!(
                        "ioctl({}, RTC_UIE_ON, 0) to {} failed, {}",
                        fd,
                        get_file_path_from_fd(fd).unwrap().display(),
                        e.to_string()
                    ),
                ))
            }
        }
    }
}

///
fn busywait_for_rtc_clock_tick(rtc_fd: i32) -> UResult<()> {
    let mut start_time = BrokenTime::default();
    let mut now_time = BrokenTime::default();
    if unsafe { rtc_ioctl_read_time(rtc_fd, &mut start_time) }.is_err() {
        return Err(USimpleError::new(
            1,
            format!(
                "ioctl(RTC_RD_TIME) to {} to read the time failed",
                get_file_path_from_fd(rtc_fd).unwrap().display()
            ),
        ));
    }
    let begin = SystemTime::now();
    loop {
        match unsafe { rtc_ioctl_read_time(rtc_fd, &mut now_time) } {
            Ok(_) => {
                if now_time.tm_sec != start_time.tm_sec {
                    return Ok(());
                }
            }
            Err(_) => {
                return Err(USimpleError::new(
                    1,
                    format!(
                        "ioctl(RTC_RD_TIME) to {} to read the time failed",
                        get_file_path_from_fd(rtc_fd).unwrap().display()
                    ),
                ))
            }
        }
        match begin.elapsed() {
            Ok(duration) => {
                if duration.as_secs_f64() > 1.5 {
                    return Err(USimpleError::new(
                        1,
                        "Timed out while busywaiting, maybe systime has been changed",
                    ));
                }
            }
            Err(_) => {
                return Err(USimpleError::new(
                    1,
                    "loop exited, maybe systime has been changed",
                ));
            }
        }
    }
}

///
const RTC_PARAM_FEATURES: u64 = 0;
///
const RTC_PARAM_CORRECTION: u64 = 1;
///
const RTC_PARAM_BACKUP_SWITCH: u64 = 2;

///
const RTC_PARAMS: &[(u64, &str, &str)] = &[
    (RTC_PARAM_FEATURES, "features", "supported features"),
    (RTC_PARAM_CORRECTION, "correction", "time correction"),
    (RTC_PARAM_BACKUP_SWITCH, "bsm", "backup switch"),
];

/// convert user input RTC parameter name to parameter id
fn resolve_rtc_param_alias(alias: &String) -> Option<u64> {
    for &(id, name, _help) in RTC_PARAMS {
        if name == alias {
            return Some(id);
        }
    }
    if alias.starts_with("0x") {
        u64::from_str_radix(alias.strip_prefix("0x").unwrap(), 16).ok()
    } else if alias.starts_with("0X") {
        u64::from_str_radix(alias.strip_prefix("0X").unwrap(), 16).ok()
    } else {
        u64::from_str_radix(alias, 10).ok()
    }
}

///
pub fn rtc_get_param(config: &HwclockConfig) -> UResult<(u64, u64)> {
    let name = config.rtc_param_get_opt_param.as_ref().unwrap();
    let mut param = RtcParam {
        param: match resolve_rtc_param_alias(name) {
            Some(n) => n,
            None => {
                return Err(USimpleError::new(
                    1,
                    "could not convert parameter name to number",
                ))
            }
        },
        u: RtcParamUnion { uvalue: 0 },
        index: 0,
        pad: 0,
    };
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    match unsafe { rtc_ioctl_param_get(fd, &mut param) } {
        Ok(_) => {
            if config.verbose_opt {
                println!(
                    "ioctl({}, RTC_PARAM_GET, param) to {} succeeded.",
                    fd,
                    get_file_path_from_fd(fd).unwrap().display()
                );
            };
            Ok((param.param, unsafe { param.u.uvalue }))
        }
        Err(e) => Err(USimpleError::new(
            1,
            format!(
                "ioctl({}, RTC_PARAM_GET, param) to {} failed, {}",
                fd,
                get_file_path_from_fd(fd).unwrap().display(),
                e.desc()
            ),
        )),
    }
}

///
pub fn rtc_set_param(config: &HwclockConfig) -> UResult<()> {
    let name = &config.rtc_param_set_opt_params.clone().unwrap()[0];
    let value = &config.rtc_param_set_opt_params.clone().unwrap()[1];
    let param = RtcParam {
        param: match resolve_rtc_param_alias(name) {
            Some(n) => n,
            None => {
                return Err(USimpleError::new(
                    1,
                    "could not convert parameter name to number",
                ))
            }
        },
        u: RtcParamUnion {
            uvalue: match value.parse::<u64>() {
                Ok(v) => v,
                Err(_) => {
                    return Err(USimpleError::new(
                        1,
                        "could not convert parameter value to number",
                    ))
                }
            },
        },
        index: 0,
        pad: 0,
    };
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    match unsafe { rtc_ioctl_param_set(fd, &param) } {
        Ok(_) => {
            if config.verbose_opt {
                println!(
                    "ioctl({}, RTC_PARAM_SET, param) to {} succeeded.",
                    fd,
                    get_file_path_from_fd(fd).unwrap().display()
                );
            }
            Ok(())
        }
        Err(e) => Err(USimpleError::new(
            1,
            format!(
                "ioctl({}, RTC_PARAM_SET, param) to {} failed, {}",
                fd,
                get_file_path_from_fd(fd).unwrap().display(),
                e.to_string()
            ),
        )),
    }
}

///
const RTC_VL_DATA_INVALID: u32 = 1 << 0;
///
const RTC_VL_BACKUP_LOW: u32 = 1 << 1;
///
const RTC_VL_BACKUP_EMPTY: u32 = 1 << 2;
///
const RTC_VL_ACCURACY_LOW: u32 = 1 << 3;
///
const RTC_VL_BACKUP_SWITCH: u32 = 1 << 4;

///
const VL_BITS: &[(u32, &str)] = &[
    (RTC_VL_DATA_INVALID, "Voltage too low, RTC data is invalid"),
    (RTC_VL_BACKUP_LOW, "Backup voltage is low"),
    (RTC_VL_BACKUP_EMPTY, "Backup empty or not present"),
    (
        RTC_VL_ACCURACY_LOW,
        "Voltage is low, RTC accuracy is reduced",
    ),
    (RTC_VL_BACKUP_SWITCH, "Backup switchover happened"),
];

/// read a voltage low information
pub fn rtc_vl_read(config: &HwclockConfig) -> UResult<()> {
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    let mut vl_value: u32 = 0;
    match unsafe { rtc_ioctl_vl_read(fd, &mut vl_value) } {
        Ok(r) => {
            if config.verbose_opt {
                println!("ioctl({}, RTC_VL_READ) returned {}", fd, r);
            }
        }
        Err(_) => {
            return Err(USimpleError::new(
                1,
                format!("ioctl({}, RTC_VL_READ) failed", fd),
            ))
        }
    }
    for &(bit, _desc) in VL_BITS {
        if vl_value == bit {
            return Ok(());
        }
    }
    Err(USimpleError::new(1, format!("{} - Unknown bit", vl_value)))
}

/// clear a voltage low information
pub fn rtc_vl_clear(config: &HwclockConfig) -> UResult<()> {
    let fd = open_rtc(config, config.rtc_opt.as_ref());
    if fd == HWCLOCK_ERROR {
        return Err(USimpleError::new(1, "Unable to open RTC"));
    }
    match unsafe { rtc_ioctl_vl_clear(fd) } {
        Ok(_) => {
            if config.verbose_opt {
                println!("ioctl({}, RTC_VL_CLEAR) succeeded", fd);
            };
            Ok(())
        }
        Err(_) => Err(USimpleError::new(
            1,
            format!("ioctl({}, RTC_VL_CLEAR) failed", fd),
        )),
    }
}

///
const RTC_CLOCK_OPERATIONS: &ClockOperations = &ClockOperations {
    interface_name: "Using the rtc interface to the clock.",
    get_permissions: get_permissions_rtc,
    read_hardware_clock: read_hardware_clock_rtc,
    set_hardware_clock: set_hardware_clock_rtc,
    synchronize_to_clock_tick: synchronize_to_clock_tick_rtc,
};

///
pub fn probe_for_rtc_device(config: &HwclockConfig) -> Option<&'static ClockOperations> {
    if open_rtc(config, config.rtc_opt.as_ref()) < 0 {
        return None;
    }
    Some(RTC_CLOCK_OPERATIONS)
}
