//! This file is part of the easybox package.
//
// (c)  Maxwell Xu <eureka0xff@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use chrono::{Datelike, NaiveDate, NaiveTime};
use chrono::{Local, NaiveDateTime, TimeZone, Timelike, Utc};
use clap::{crate_version, Arg, ArgAction, ArgGroup, Command};
use nix::libc::{clock_settime, syscall, SYS_settimeofday, CLOCK_REALTIME};
use nix::libc::{timespec, timeval};
use std::fs::File;
use std::io::{read_to_string, Read, Write};
use std::os::raw::c_int;
use std::path::PathBuf;
use std::process::exit;
use std::ptr::null;
use uucore::error::{UResult, USimpleError};
use uucore::libc::EXIT_FAILURE;
use uucore::msg_log::warnx;
use uucore::{format_usage, help_section, help_usage};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
///
mod directisa;

#[cfg(target_os = "linux")]
///
mod rtc_device;

///
pub const ABOUT: &str = help_section!("about", "hwclock.md");
///
pub const USAGE: &str = help_usage!("hwclock.md");

///
pub const EPOCH: i32 = 1900;
///
pub const MICROS_PER_SECOND: i64 = 1_000_000;
///
pub const NANOS_PER_SECOND: i64 = 1_000_000_000;

/// delay seconds for unknown devices
const RTC_DELAY_HARDWARE_UNKNOWN: f64 = 0.5;

/// default adjustment file path
const ADJTIME_PATH: &str = "/etc/adjtime";

///
pub const HWCLOCK_ERROR: i32 = -1;

/// functions arguments
pub mod arg_functions {
    ///
    pub const SHOW: &str = "show";
    ///
    pub const GET: &str = "get";
    ///
    pub const SET: &str = "set";
    ///
    pub const HCTOSYS: &str = "hctosys";
    ///
    pub const SYSTOHC: &str = "systohc";
    ///
    pub const SYSTZ: &str = "systz";
    ///
    pub const ADJUST: &str = "adjust";
    ///
    pub const PARAM_GET: &str = "param-get";
    ///
    pub const PARAM_SET: &str = "param-set";
    ///
    pub const PREDICT: &str = "predict";
    ///
    pub const VL_READ: &str = "vl-read";
    ///
    pub const VL_CLEAR: &str = "vl-clear";
}

/// options arguments
pub mod arg_options {
    ///
    pub const UTC: &str = "utc";
    ///
    pub const LOCALTIME: &str = "localtime";
    ///
    pub const RTC: &str = "rtc";
    ///
    pub const DIRECTISA: &str = "directisa";
    ///
    pub const DATE: &str = "date";
    ///
    pub const DELAY: &str = "delay";
    ///
    pub const UPDATE_DRIFT: &str = "update-drift";
    ///
    pub const NOADJFILE: &str = "noadjfile";
    ///
    pub const ADJFILE: &str = "adjfile";
    ///
    pub const TEST: &str = "test";
    ///
    pub const VERBOSE: &str = "verbose";
    ///
    pub const DEBUG: &str = "debug";
}

/// app configuration
pub struct HwclockConfig {
    ///
    pub adjfile_opt: Option<String>,
    ///
    pub date_opt: Option<i64>,
    ///
    pub delay_opt: Option<f64>,
    ///
    pub directisa_opt: bool,
    ///
    pub rtc_opt: Option<String>,
    ///
    pub localtime_opt: bool,
    ///
    pub noadjfile_opt: bool,
    ///
    pub test_opt: bool,
    ///
    pub utc_opt: bool,
    ///
    pub update_drift_opt: bool,
    ///
    pub verbose_opt: bool,
    ///
    pub rtc_param_get_opt_param: Option<String>,
    ///
    pub rtc_param_set_opt_params: Option<Vec<String>>,
    /// if true, hardware clock input and output will be considered as UTC
    pub clock_is_utc: bool,
    ///
    pub show: bool,
    ///
    pub get: bool,
    ///
    pub set: bool,
    ///
    pub hctosys: bool,
    ///
    pub systohc: bool,
    ///
    pub systz: bool,
    ///
    pub adjust: bool,
    ///
    pub param_get: bool,
    ///
    pub param_set: bool,
    ///
    pub predict: bool,
    ///
    pub vl_read: bool,
    ///
    pub vl_clear: bool,
}

/// broken tm struct in time.h
#[derive(Default, Clone, Copy)]
pub struct BrokenTime {
    ///
    pub tm_sec: c_int,
    ///
    pub tm_min: c_int,
    ///
    pub tm_hour: c_int,
    ///
    pub tm_mday: c_int,
    ///
    pub tm_mon: c_int,
    ///
    pub tm_year: c_int,
    ///
    pub tm_wday: c_int,
    ///
    pub tm_yday: c_int,
    ///
    pub tm_isdst: c_int,
}

impl From<NaiveDateTime> for BrokenTime {
    ///
    fn from(time: NaiveDateTime) -> BrokenTime {
        BrokenTime {
            tm_sec: time.second() as c_int,
            tm_min: time.minute() as c_int,
            tm_hour: time.hour() as c_int,
            tm_mday: time.day() as c_int,
            tm_mon: time.month0() as c_int,
            tm_year: time.year() - EPOCH as c_int,
            ..Default::default()
        }
    }
}

impl From<BrokenTime> for NaiveDateTime {
    ///
    fn from(time: BrokenTime) -> NaiveDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                time.tm_year + EPOCH,
                time.tm_mon as u32 + 1,
                time.tm_mday as u32,
            )
            .expect("Broken tm date is invalid"),
            match time.tm_sec {
                // leap second
                60 => NaiveTime::from_hms_milli_opt(
                    time.tm_hour as u32,
                    time.tm_min as u32,
                    59,
                    NANOS_PER_SECOND as u32,
                )
                .expect("Broken tm time is invalid"),
                _ => NaiveTime::from_hms_opt(
                    time.tm_hour as u32,
                    time.tm_min as u32,
                    time.tm_sec as u32,
                )
                .expect("Broken tm time is invalid"),
            },
        )
    }
}

///
#[allow(non_camel_case_types, dead_code)]
#[derive(Default)]
struct timezone {
    /// Minutes west of GMT
    pub tz_minutewest: c_int,
    /// Nonzero if DST is ever in effect
    pub tz_dsttime: c_int,
}

///
pub struct ClockOperations {
    ///
    interface_name: &'static str,
    /// set the hardware clock permission
    get_permissions: fn(config: &HwclockConfig) -> UResult<()>,
    /// read time from hardware clock
    read_hardware_clock: fn(config: &HwclockConfig) -> UResult<BrokenTime>,
    /// set the hardware clock time
    set_hardware_clock: fn(config: &HwclockConfig, time: &BrokenTime) -> UResult<()>,
    /// synchronize to seconds value jump edge
    synchronize_to_clock_tick: fn(config: &HwclockConfig) -> UResult<()>,
}

impl ClockOperations {
    ///
    pub fn get_permissions(&self, config: &HwclockConfig) -> UResult<()> {
        let func = self.get_permissions;
        func(config)
    }
    ///
    pub fn read_hardware_clock(&self, config: &HwclockConfig) -> UResult<BrokenTime> {
        let func = self.read_hardware_clock;
        func(config)
    }
    ///
    pub fn set_hardware_clock(&self, config: &HwclockConfig, time: &BrokenTime) -> UResult<()> {
        let func = self.set_hardware_clock;
        func(config, time)
    }
    ///
    pub fn synchronize_to_clock_tick(&self, config: &HwclockConfig) -> UResult<()> {
        let func = self.synchronize_to_clock_tick;
        func(config)
    }
}

/// convert a timestamp to timeval
pub fn timestamp_to_timeval(time: i64) -> timeval {
    timeval {
        tv_sec: time,
        tv_usec: 0,
    }
}

/// convert a microsecond timestamp to timeval
pub fn timestamp_micros_to_timeval(time: i64) -> timeval {
    timeval {
        tv_sec: time / MICROS_PER_SECOND,
        tv_usec: time % MICROS_PER_SECOND,
    }
}

/// timeval increment
pub fn time_inc(addend: &timeval, increment: f64) -> timeval {
    let mut new_time = timeval {
        tv_sec: addend.tv_sec,
        tv_usec: addend.tv_usec,
    };
    new_time.tv_sec += increment as i64;
    new_time.tv_usec += ((increment - increment as i64 as f64) * (MICROS_PER_SECOND as f64)) as i64;
    if new_time.tv_usec < 0 {
        new_time.tv_sec -= 1;
        new_time.tv_usec += MICROS_PER_SECOND;
    } else if new_time.tv_usec >= MICROS_PER_SECOND {
        new_time.tv_sec += 1;
        new_time.tv_usec -= MICROS_PER_SECOND;
    }
    new_time
}

/// convert a timeval duration into seconds since EPOCH, microseconds are included.
pub fn time_diff(subtrahend: &timeval, subtractor: &timeval) -> f64 {
    (subtrahend.tv_sec - subtractor.tv_sec) as f64
        + (subtrahend.tv_usec - subtractor.tv_usec) as f64 / MICROS_PER_SECOND as f64
}

/// timeval add
pub fn timeval_add(a: &timeval, b: &timeval) -> timeval {
    let mut result = timeval {
        tv_sec: a.tv_sec,
        tv_usec: a.tv_usec,
    };
    result.tv_sec += b.tv_sec;
    result.tv_usec += b.tv_usec;
    if result.tv_usec >= MICROS_PER_SECOND {
        result.tv_sec += 1;
        result.tv_usec -= MICROS_PER_SECOND;
    }
    result
}

///
fn determine_clock_access_method(config: &HwclockConfig) -> &'static ClockOperations {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // default use cmos to get ur
        let directisa_clock = directisa::probe_for_directisa(config);
        if config.directisa_opt && directisa_clock.is_some() {
            if config.verbose_opt {
                println!("{}", directisa_clock.unwrap().interface_name);
            }
            return directisa_clock.unwrap();
        }
    }
    #[cfg(target_os = "linux")]
    // only linux machine can use RTC device
    {
        let rtc_device_clock = rtc_device::probe_for_rtc_device(config);
        if rtc_device_clock.is_some() {
            if config.verbose_opt {
                println!("{}", rtc_device_clock.unwrap().interface_name);
            }
            return rtc_device_clock.unwrap();
        }
    }
    if config.verbose_opt {
        println!("No usable clock interface found");
    }
    warnx("Cannot access the Hardware Clock via any known method");
    if !config.verbose_opt {
        warnx("Use the --verbose option to see the details of our search for an access method");
    }
    exit(1);
}

/// hardware clock is UTC in adjustment file
const UTC: i32 = 0;
/// hardware clock is UTC in adjustment file
const LOCAL: i32 = 1;
/// hardware clock is local in adjustment file
const UNKNOWN: i32 = 2;

///
/// util-linux/sys-utils/hwclock.c
#[derive(Default, Clone, Copy)]
struct AdjTime {
    /// non-zero if we need to save the adjtime file
    pub dirty: i32,
    /// how much time actually runs per day
    pub drift_factor: f64,
    /// last adjustment time
    pub last_adj_time: i64,
    /// time not adjusted
    pub not_adjusted: f64,
    /// last calibration time
    pub last_calib_time: i64,
    /// UTC: 0, LOCAL: 1, UNKNOWN: 2
    pub local_utc: i32,
}

///
fn hw_clock_is_utc(config: &HwclockConfig, adjtime: &AdjTime) -> bool {
    let ret = match config.utc_opt {
        true => true,
        _ => match config.localtime_opt {
            true => false,
            _ => adjtime.local_utc != LOCAL,
        },
    };
    if config.verbose_opt {
        println!(
            "Assuming hardware clock is kept in {} time",
            match ret {
                true => "UTC",
                false => "local",
            }
        )
    }
    ret
}

// There are three lines in /etc/adjtime
// First line: drift factor, last adjust time and adjustment status
// Second line: last calibration time
// Third line: clock mode (UTC, LOCAL or UNKNOWN)
fn read_adjtime(config: &HwclockConfig) -> UResult<AdjTime> {
    let adjfile_path = PathBuf::from(match &config.adjfile_opt {
        Some(s) => s,
        None => return Err(USimpleError::new(1, "Expected --adjfile")),
    });
    let mut adjfile = match File::open(&adjfile_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(USimpleError::new(
                EXIT_FAILURE,
                format!("Cannot open {}, {}", adjfile_path.display(), e.to_string()),
            ))
        }
    };
    let content_raw = match read_to_string(&mut adjfile) {
        Ok(s) => s,
        Err(_) => return Err(USimpleError::new(1, "Adjustment file is broken")),
    };
    let lines: Vec<&str> = content_raw.split('\n').collect();
    let line_1_vec: Vec<&str> = lines[0].split(' ').collect();
    if lines.len() < 3 || line_1_vec.len() != 3 {
        return Err(USimpleError::new(1, "Adjustment file is broken"));
    }
    let adjtime = AdjTime {
        drift_factor: line_1_vec[0]
            .parse::<f64>()
            .expect("unrecognized drift factor"),
        last_adj_time: line_1_vec[1]
            .parse::<i64>()
            .expect("unrecognized last adjust time"),
        not_adjusted: line_1_vec[2]
            .parse::<f64>()
            .expect("unrecognized not adjusted time"),
        last_calib_time: lines[1]
            .parse::<i64>()
            .expect("unrecognized last calib time"),
        local_utc: match lines[2] {
            "UTC" => UTC,
            "LOCAL" => LOCAL,
            _ => UNKNOWN,
        },
        ..Default::default()
    };
    if config.verbose_opt {
        println!(
            "Last drift adjustment done at {} seconds after 1969.",
            adjtime.last_adj_time
        );
        println!(
            "Last calibration done at {} seconds after 1969",
            adjtime.last_calib_time
        );
        println!(
            "Hardware clock is on {} time",
            match adjtime.local_utc {
                UTC => "UTC",
                LOCAL => "Local",
                _ => "unknown",
            }
        )
    }
    Ok(adjtime)
}

/// directly set hardware clock
fn set_hardware_clock(
    config: &HwclockConfig,
    clock: &ClockOperations,
    newtime: i64,
) -> UResult<()> {
    let naivedatetime = match config.clock_is_utc {
        true => {
            let gmttime = Utc.timestamp_opt(newtime, 0).unwrap();
            gmttime.naive_utc()
        }
        false => {
            let localtime = Local.timestamp_opt(newtime, 0).unwrap();
            localtime.naive_local()
        }
    };
    if config.verbose_opt {
        println!(
            "Setting Hardware Clock to {} = {} seconds since 1969",
            naivedatetime, newtime
        );
    }
    if !config.test_opt {
        return clock.set_hardware_clock(config, &BrokenTime::from(naivedatetime));
    }
    Ok(())
}

/// get RTC time from hardware clock
fn read_hardware_clock(config: &HwclockConfig, clock: &ClockOperations) -> UResult<i64> {
    let clock_broken_tm = match clock.read_hardware_clock(config) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };
    if config.verbose_opt {
        println!(
            "Time read from Hardware Clock: {:4}-{:02}-{:02} {:02}:{:02}:{:02}",
            clock_broken_tm.tm_year + EPOCH,
            clock_broken_tm.tm_mon + 1,
            clock_broken_tm.tm_mday,
            clock_broken_tm.tm_hour,
            clock_broken_tm.tm_min,
            clock_broken_tm.tm_sec
        );
    }

    let clock_naive_time = NaiveDateTime::from(clock_broken_tm);
    let localed_clock_time = match config.clock_is_utc {
        true => Local.from_utc_datetime(&clock_naive_time),
        false => Local.from_local_datetime(&clock_naive_time).unwrap(),
    };

    if config.verbose_opt {
        println!(
            "Hw clock time: {} = {} seconds since 1969",
            localed_clock_time.format("%Y/%m/%d %H:%M:%S"),
            localed_clock_time.timestamp()
        );
    }
    Ok(localed_clock_time.timestamp())
}

/// accurately set hardware clock
fn set_hardware_clock_exact(
    config: &HwclockConfig,
    clock: &ClockOperations,
    set_hw_time: i64,
    sys_ref_time: &timeval,
) -> UResult<()> {
    let mut target_i64olerance_secs = 0.001;
    let mut tolerance_incr_secs = 0.001;
    let delay = match config.delay_opt {
        Some(delay) => {
            if delay != -1.0 {
                config.delay_opt.unwrap()
            } else {
                get_hardware_delay(config)
            }
        }
        None => get_hardware_delay(config),
    };
    if config.verbose_opt {
        println!("Using delay: {:.6} seconds", delay);
    }
    let rtc_set_delay_tv = timeval {
        tv_sec: 0,
        tv_usec: (delay * 1e6) as i64,
    };

    let mut target_sys_time = timeval_add(sys_ref_time, &rtc_set_delay_tv);
    let mut prev_sys_time = *sys_ref_time;
    let mut now_sys_time: timeval;
    let mut delta_vs_target: f64;
    let mut ticksize: f64;
    loop {
        now_sys_time = timestamp_micros_to_timeval(Utc::now().timestamp_micros());
        delta_vs_target = time_diff(&now_sys_time, &target_sys_time);
        ticksize = time_diff(&now_sys_time, &prev_sys_time);
        prev_sys_time = now_sys_time;
        if ticksize < 0.0 {
            if config.verbose_opt {
                println!(
                    "time jumped backward {:.6} seconds to {}.{:06} - retargeting",
                    ticksize, now_sys_time.tv_sec, now_sys_time.tv_usec
                );
            }
        } else if delta_vs_target < 0.0 {
            continue;
        } else if delta_vs_target <= target_i64olerance_secs {
            break;
        } else {
            if config.verbose_opt {
                println!(
                    "missed it - {}.{:06} is too far past {}.{:06} ({:.6} > {:.6})",
                    now_sys_time.tv_sec,
                    now_sys_time.tv_usec,
                    target_sys_time.tv_sec,
                    target_sys_time.tv_usec,
                    delta_vs_target,
                    target_i64olerance_secs
                );
            }
            target_i64olerance_secs += tolerance_incr_secs;
            tolerance_incr_secs *= 2.0;
        }
        if now_sys_time.tv_usec < target_sys_time.tv_usec {
            target_sys_time.tv_sec = now_sys_time.tv_sec;
        } else {
            target_sys_time.tv_sec = now_sys_time.tv_sec + 1;
        }
    }
    let new_hw_time = set_hw_time + {
        let diff = time_diff(&now_sys_time, sys_ref_time) - delay;
        if diff - diff as i64 as f64 > 0.0 {
            diff as i64 as f64 + 1.0
        } else {
            diff as i64 as f64
        }
    } as i64;
    if config.verbose_opt {
        println!(
            "{}.{} is close enough to {}.{:06} ({:.6} < {:.6})",
            now_sys_time.tv_sec,
            now_sys_time.tv_usec,
            target_sys_time.tv_sec,
            target_sys_time.tv_usec,
            delta_vs_target,
            target_i64olerance_secs
        );
        println!(
            "Set RTC to {} ({} + {}; refsystime = {}.{:06})",
            new_hw_time,
            set_hw_time,
            new_hw_time - set_hw_time,
            sys_ref_time.tv_sec,
            sys_ref_time.tv_usec
        );
    }
    set_hardware_clock(config, clock, new_hw_time)
}

/// get RTC delay seconds from hardware
fn get_hardware_delay(config: &HwclockConfig) -> f64 {
    let path = match rtc_device::get_file_path() {
        Some(path) => path,
        None => return RTC_DELAY_HARDWARE_UNKNOWN,
    };
    let rtc_name = path.file_name();
    if rtc_name.is_none() {
        return RTC_DELAY_HARDWARE_UNKNOWN;
    }
    let mut rtc_name_file = match File::open(format!(
        "/sys/class/rtc/{}/name",
        rtc_name.unwrap().to_string_lossy()
    )) {
        Ok(f) => f,
        Err(_) => {
            return RTC_DELAY_HARDWARE_UNKNOWN;
        }
    };
    // if we can access the file in /sys/class/rtc
    let mut file_content = String::new();
    let _ = rtc_name_file.read_to_string(&mut file_content);
    // In my machine, /sys/class/rtc/rtc0/name is `rtc_cmos rtc_cmos`
    let names: Vec<&str> = file_content.split_whitespace().collect();
    if config.verbose_opt {
        println!("RTC type: {}", names[0]);
    }
    // x86 cmos RTC
    if names.contains(&"rtc_cmos") {
        return 0.5;
    }
    // Another hardware
    return 0.0;
}

/// synchronize to RTC clock tick, wait until RTC value updated
fn synchronize_to_clock_tick(config: &HwclockConfig, clock: &ClockOperations) -> UResult<()> {
    if config.verbose_opt {
        println!("Waiting for clock tick...");
    }
    match clock.synchronize_to_clock_tick(config) {
        Ok(_) => {
            if config.verbose_opt {
                println!("...got clock tick");
            }
            Ok(())
        }
        Err(e) => {
            if config.verbose_opt {
                println!("...synchronization failed");
            }
            Err(e)
        }
    }
}

/// calculate seconds drift
fn calculate_adjustment(
    config: &HwclockConfig,
    factor: f64,
    last_time: i64,
    not_adjusted: f64,
    sys_time: i64,
) -> timeval {
    let exact_adjustment = (sys_time - last_time) as f64 * factor / 86400.0 + not_adjusted;
    let drift = time_inc(
        &timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        exact_adjustment,
    );

    if config.verbose_opt {
        println!(
            "Time since last adjustment is {} seconds",
            sys_time - last_time
        );
        println!(
            "Calculated Hardware Clock drift is {}.{:06}",
            drift.tv_sec, drift.tv_usec
        )
    }
    drift
}

/// max drift factor
const MAX_DRIFT: f64 = 2145.0;

///
fn adjust_drift_factor(
    config: &HwclockConfig,
    adjtime: &mut AdjTime,
    hwclock_time: &timeval,
    now_time: &timeval,
) {
    if !config.update_drift_opt {
        if config.verbose_opt {
            println!("Not adjusting drift factor because the --update-drift option was not used");
        }
    } else if adjtime.last_calib_time == 0 {
        if config.verbose_opt {
            println!("Not adjusting drift factor because last calibration time is zero,");
            println!("so history is bad and calibration startover is necessary");
        }
    } else if hwclock_time.tv_sec - adjtime.last_calib_time < 4 * 60 * 60 {
        if config.verbose_opt {
            println!("Not adjusting drift factor because it has been less than four hours since the last calibration");
        }
    } else {
        let last_calib = timestamp_to_timeval(adjtime.last_calib_time);
        let factor_adjust = time_diff(now_time, hwclock_time)
            / (time_diff(now_time, &last_calib) / (24.0 * 60.0 * 60.0));
        let mut drift_factor = adjtime.drift_factor + factor_adjust;
        if drift_factor.abs() > MAX_DRIFT {
            if config.verbose_opt {
                println!(
                    "Clock drift factor was calculated as {:.6} seconds/day.",
                    drift_factor
                );
                println!("It is far too much. Resetting to zero.");
            }
            drift_factor = 0.0;
        } else {
            if config.verbose_opt {
                println!(
                    "Clock drifted {:.6} seconds in the past {:.6} seconds",
                    time_diff(now_time, hwclock_time),
                    time_diff(now_time, &last_calib)
                );
                println!(
                    "in spite of a drift factor of {:.6} days/day.",
                    adjtime.drift_factor
                );
                println!("Adjusting drift factor by {:.6} seconds/day", factor_adjust);
            }
        }
        adjtime.drift_factor = drift_factor;
    }
    adjtime.last_calib_time = now_time.tv_sec;
    adjtime.last_adj_time = now_time.tv_sec;
    adjtime.not_adjusted = 0.0;
    adjtime.dirty = 1;
}

///
fn do_adjustment(
    config: &HwclockConfig,
    clock: &ClockOperations,
    adjtime: &mut AdjTime,
    hwclock_time: &timeval,
    sys_time: &timeval,
) -> UResult<()> {
    if adjtime.last_adj_time == 0 {
        if config.verbose_opt {
            println!("Not setting clock because last adjustment time is zero, so history is bad.");
        }
    } else if adjtime.drift_factor.abs() > MAX_DRIFT {
        if config.verbose_opt {
            println!(
                "Not setting clock because drift factor {} is far too high.",
                adjtime.drift_factor
            );
        }
    } else {
        let rc = set_hardware_clock_exact(
            config,
            clock,
            hwclock_time.tv_sec,
            &time_inc(
                &sys_time,
                -(hwclock_time.tv_usec / MICROS_PER_SECOND) as f64,
            ),
        );
        adjtime.last_adj_time = hwclock_time.tv_sec;
        adjtime.not_adjusted = 0.0;
        adjtime.dirty = 1;
        return rc;
    }
    Ok(())
}

/// save adjustment file to /etc/adjtime or --adjfile
fn save_adjtime(config: &HwclockConfig, adjtime: &AdjTime) -> UResult<()> {
    let content = String::from(format!(
        "{:.06} {} {:.06}\n{}\n{}\n",
        adjtime.drift_factor,
        adjtime.last_adj_time,
        adjtime.not_adjusted,
        adjtime.last_calib_time,
        match adjtime.local_utc {
            LOCAL => "LOCAL",
            _ => "UTC",
        }
    ));
    if config.verbose_opt {
        print!(
            "New {} data: \n{}",
            config.adjfile_opt.as_ref().unwrap(),
            content
        );
    }
    if !config.test_opt {
        let mut file = match File::create(config.adjfile_opt.as_ref().unwrap()) {
            Ok(f) => f,
            Err(e) => {
                return Err(USimpleError::new(
                    HWCLOCK_ERROR,
                    format!("cannot open adjfile, {}", e.to_string()),
                ))
            }
        };
        file.write(content.as_bytes()).unwrap();
    }
    Ok(())
}

/// set kernel's timezone
/// we can use settimeofday(NULL, timezone), but nix does not implement timezone
fn __set_timezone(tz: &timezone) -> i64 {
    unsafe {
        syscall(
            SYS_settimeofday,
            null::<i64>(),
            tz as *const timezone as i64,
        )
    }
}

/// set kernel's timezone and time
/// kernel's timezone is often not used by applications, these applications reads /etc/localtime
/// but kernel's time is used
fn set_system_clock(config: &HwclockConfig, newtime: &timeval) -> UResult<()> {
    let minutewest = Local::now().offset().utc_minus_local() / 60;
    if config.verbose_opt {
        if config.clock_is_utc {
            println!("Calling settimeofday(NULL, 0) to lock the warp_lock function");
            if minutewest != 0 {
                println!(
                    "Calling settimeofday(NULL, {}) to set the kernel timezone",
                    minutewest
                );
            }
        } else {
            println!("Calling settimeofday(NULL, {}) to warp system time, set PCIL and the kernel timezone", minutewest);
        }
        if config.hctosys {
            println!(
                "Calling settimeofday({}.{:06}, NULL) to set the system time",
                newtime.tv_sec, newtime.tv_usec
            );
        }
    }
    if !config.test_opt {
        let tz_utc = timezone::default();
        let tz = timezone {
            tz_minutewest: minutewest,
            tz_dsttime: 0,
        };
        let mut rc = 0;
        if config.clock_is_utc {
            let r = __set_timezone(&tz_utc);
            if r != 0 {
                eprintln!("syscall to set UTC timezone failed with exit code {}", r);
                rc = HWCLOCK_ERROR;
            }
        }
        if rc == 0 && !(config.clock_is_utc && minutewest == 0) {
            let r = __set_timezone(&tz);
            if r != 0 {
                eprintln!("syscall to set Local timezone failed with exit code {}", r);
                rc = HWCLOCK_ERROR;
            }
        }
        if rc == 0 && config.hctosys {
            rc = unsafe {
                clock_settime(
                    CLOCK_REALTIME,
                    &timespec {
                        tv_sec: newtime.tv_sec,
                        tv_nsec: newtime.tv_usec * 1_000,
                    },
                )
                .into()
            };
        }
        return match rc {
            0 => Ok(()),
            _ => {
                eprintln!("settimeofday() failed");
                Err(USimpleError::new(1, "settimeofday() failed"))
            }
        };
    }
    Ok(())
}

/// display a timeval in local timezone
fn display_time(time: &timeval) {
    let local = Local
        .timestamp_micros(time.tv_sec * MICROS_PER_SECOND + time.tv_usec)
        .unwrap();
    println!("{}", local);
}

/// handle all hardware clock functions
fn manipulate_clock(
    config: &HwclockConfig,
    set_time: i64,
    startup_time: &timeval,
    adjtime: &mut AdjTime,
    clock: &ClockOperations,
) -> UResult<()> {
    let mut read_time = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut hwclock_time = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut startup_hwclock_time = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut time_drift = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if (config.set || config.systohc || config.adjust)
        && (adjtime.local_utc == UTC) != config.clock_is_utc
    {
        adjtime.local_utc = match config.clock_is_utc {
            true => UTC,
            false => LOCAL,
        };
        adjtime.dirty = 1;
    }

    if config.predict {
        hwclock_time = timestamp_to_timeval(set_time);
        time_drift = calculate_adjustment(
            config,
            adjtime.drift_factor,
            adjtime.last_adj_time,
            adjtime.not_adjusted,
            hwclock_time.tv_sec,
        );
        hwclock_time = time_inc(
            &hwclock_time,
            -(time_drift.tv_sec as f64 + time_drift.tv_usec as f64 / 1e6),
        );

        if config.verbose_opt {
            println!("Target date:   {}", set_time);
            println!("Predicted RTC: {}", hwclock_time.tv_sec);
        }
        display_time(&hwclock_time);
        return Ok(());
    }

    if config.systz {
        return set_system_clock(config, startup_time);
    }
    let get_permissions = clock.get_permissions(config);
    if get_permissions.is_err() {
        return Err(get_permissions.unwrap_err());
    }

    if !((config.set || config.systohc) && !config.update_drift_opt) {
        match synchronize_to_clock_tick(config, clock) {
            Err(e) => return Err(e),
            Ok(_) => (),
        }
        // in original util-linux, there is an invalid check in read_hardware_clock
        // in our program, if broken time data is invalid, BrokenTime to NaiveDateTime transform will panic
        hwclock_time = timestamp_to_timeval(match read_hardware_clock(config, clock) {
            Ok(t) => t,
            Err(e) => return Err(e),
        });
        read_time = timestamp_micros_to_timeval(Utc::now().timestamp_micros());
        time_drift = calculate_adjustment(
            config,
            adjtime.drift_factor,
            adjtime.last_adj_time,
            adjtime.not_adjusted,
            hwclock_time.tv_sec,
        );
        if !config.show {
            hwclock_time = time_inc(&time_drift, hwclock_time.tv_sec as f64);
        }
        startup_hwclock_time = time_inc(&hwclock_time, time_diff(startup_time, &read_time));
    }
    if config.show || config.get {
        display_time(&startup_hwclock_time);
    }
    if config.set {
        let rc = set_hardware_clock_exact(config, clock, set_time, startup_time);
        if rc.is_err() {
            return rc;
        }
        if !config.noadjfile_opt {
            adjust_drift_factor(
                config,
                adjtime,
                &startup_hwclock_time,
                &timestamp_to_timeval(set_time),
            );
        }
    } else if config.adjust {
        if time_drift.tv_sec > 0 || time_drift.tv_sec < -1 {
            let rc = do_adjustment(config, clock, adjtime, &hwclock_time, &read_time);
            if rc.is_err() {
                return rc;
            }
        } else {
            println!("Needed adjustment is less than one second, so not setting clock");
        }
    } else if config.systohc {
        let now_time = timestamp_micros_to_timeval(Utc::now().timestamp_micros());
        let ref_time = timestamp_to_timeval(now_time.tv_sec);
        let rc = set_hardware_clock_exact(config, clock, ref_time.tv_sec, &ref_time);
        if rc.is_err() {
            return rc;
        }
        if !config.noadjfile_opt {
            adjust_drift_factor(config, adjtime, &hwclock_time, &now_time);
        }
    } else if config.hctosys {
        return set_system_clock(config, &hwclock_time);
    }
    if !config.noadjfile_opt && adjtime.dirty != 0 {
        return save_adjtime(config, adjtime);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
/// handle RTC parameter
fn manipulate_rtc_param(config: &HwclockConfig) -> UResult<()> {
    if config.rtc_param_get_opt_param.is_some() {
        match rtc_device::rtc_get_param(config) {
            Ok((id, value)) => {
                println!("The RTC parameter 0x{:x} is set to 0x{:x}", id, value);
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "unable to read the RTC parameter {}",
                    config.rtc_param_get_opt_param.as_ref().unwrap()
                );
                return Err(e);
            }
        }
    } else if config.rtc_param_set_opt_params.is_some() {
        if config.test_opt {
            return Ok(());
        }
        return rtc_device::rtc_set_param(config);
    }
    Err(USimpleError::new(
        1,
        "Invalid argument for --param-get or --param-set",
    ))
}

#[cfg(target_os = "linux")]
/// handle RTC voltage low information
fn manipulate_rtc_voltage_low(config: &HwclockConfig) -> UResult<()> {
    if config.vl_read {
        return rtc_device::rtc_vl_read(config);
    }
    if config.vl_clear {
        return rtc_device::rtc_vl_clear(config);
    }
    Ok(())
}

/// There is no parser for unknown date formats in chrono.
/// In util-linux, hwclock can parse time in any format.
/// Therefore, we have to introduce supplementary libraries to chrono such as dateparser.
/// Although the dateparser library is used,
/// the supported formats are far less than those in util-linux.
/// The file util-linux/sys-utils/hwclock_parse_date.y,
/// which has several thousand lines, is used to support various formats.
fn parse_date_from_str(date_raw: &String) -> Option<i64> {
    let parsed =
        dateparser::parse_with(date_raw, &Local, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    parsed.map(|time| time.timestamp()).ok()
}

///
#[uucore::main]
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let startup_time = timestamp_micros_to_timeval(Local::now().timestamp_micros());
    let mut set_time = 0;
    let arguments = oe_app().get_matches_from(args);

    let rtc_opt = arguments
        .get_one::<String>(arg_options::RTC)
        .map(|path| path.to_owned());

    let delay_opt = arguments
        .get_one::<String>(arg_options::DELAY)
        .map(|delay| delay.parse::<f64>().unwrap());

    let date_opt = match arguments.get_one::<String>(arg_options::DATE) {
        Some(date_raw) => match parse_date_from_str(&date_raw) {
            Some(timestamp) => {
                set_time = timestamp;
                Some(timestamp)
            }
            None => return Err(USimpleError::new(1, "Invalid argument for --date")),
        },
        None => None,
    };
    let adjfile_opt = match arguments.get_one::<String>(arg_options::ADJFILE) {
        Some(path) => Some(path.to_owned()),
        None => Some(String::from(ADJTIME_PATH)),
    };

    // RTC parameters
    let rtc_param_get_opt_param = arguments
        .get_one::<String>(arg_functions::PARAM_GET)
        .map(|param: &String| param.to_owned());

    let rtc_param_set_opt_params: Option<Vec<String>> =
        arguments.get_many::<String>(arg_functions::PARAM_SET).map(
            |params: clap::parser::ValuesRef<'_, String>| params.map(|s| String::from(s)).collect(),
        );

    let mut config = HwclockConfig {
        directisa_opt: arguments.contains_id(arg_options::DIRECTISA),
        localtime_opt: arguments.contains_id(arg_options::LOCALTIME),
        noadjfile_opt: arguments.contains_id(arg_options::NOADJFILE),
        test_opt: arguments.contains_id(arg_options::TEST),
        utc_opt: arguments.contains_id(arg_options::UTC),
        update_drift_opt: arguments.contains_id(arg_options::UPDATE_DRIFT),
        verbose_opt: arguments.contains_id(arg_options::VERBOSE),
        rtc_opt,
        delay_opt,
        date_opt,
        adjfile_opt,
        rtc_param_get_opt_param,
        rtc_param_set_opt_params,
        clock_is_utc: true,
        show: true,
        get: arguments.contains_id(arg_functions::GET),
        set: arguments.contains_id(arg_functions::SET),
        hctosys: arguments.contains_id(arg_functions::HCTOSYS),
        systohc: arguments.contains_id(arg_functions::SYSTOHC),
        systz: arguments.contains_id(arg_functions::SYSTZ),
        adjust: arguments.contains_id(arg_functions::ADJUST),
        param_get: arguments.contains_id(arg_functions::PARAM_GET),
        param_set: arguments.contains_id(arg_functions::PARAM_SET),
        predict: arguments.contains_id(arg_functions::PREDICT),
        vl_read: arguments.contains_id(arg_functions::VL_READ),
        vl_clear: arguments.contains_id(arg_functions::VL_CLEAR),
    };

    if arguments.contains_id(arg_options::DEBUG) {
        eprintln!("hwclock: use --verbose, --debug has been deprecated.");
    }

    if config.set
        || config.get
        || config.systohc
        || config.hctosys
        || config.systz
        || config.adjust
        || config.predict
        || config.param_get
        || config.param_set
        || config.vl_read
        || config.vl_clear
    {
        config.show = false;
    }

    if config.test_opt {
        config.verbose_opt = true;
    }

    if config.update_drift_opt && !config.set && !config.systohc {
        eprintln!("--update-drift requires --set or --systohc");
        exit(1);
    }

    if config.noadjfile_opt && !config.utc_opt && !config.localtime_opt {
        eprintln!("With --noadjfile, you must specify either --utc or --localtime");
        exit(1);
    }

    if (config.set || config.predict) && config.date_opt.is_none() {
        eprintln!("--date is required for --set or --predict");
        exit(1);
    }

    #[cfg(target_os = "linux")]
    {
        if config.param_get || config.param_set {
            return manipulate_rtc_param(&config);
        }

        if config.vl_read || config.vl_clear {
            return manipulate_rtc_voltage_low(&config);
        }
    }

    if config.verbose_opt {
        println!("hwclock from easybox {}", crate_version!());
        println!(
            "System Time: {}.{:06}",
            startup_time.tv_sec, startup_time.tv_usec
        );
    }

    // In util-linux, determine_access_method() is not called here for the --systz and --predict options.
    // But as long as the RTC can be accessed, an access method result can be returned,
    // even if the RTC will not be accessed subsequently. As long as the RTC cannot be accessed,
    // the program will exit directly in determine_access_method, and there will be no subsequent
    // processing of the --systz and --predict options.
    // Here, the advantage of still calling determine_access_method() for the two cases of --systz and --systohc
    // is to simplify the code.
    let access_method = determine_clock_access_method(&config);

    let mut adjtime = AdjTime::default();
    if !config.noadjfile_opt && !(config.systz && (config.utc_opt || config.localtime_opt)) {
        adjtime = read_adjtime(&config).unwrap();
    } else {
        // Avoid writing adjtime file if we don't need to
        adjtime.dirty = 0;
    }
    config.clock_is_utc = hw_clock_is_utc(&config, &adjtime);
    let rc = manipulate_clock(
        &config,
        set_time,
        &startup_time,
        &mut adjtime,
        access_method,
    );
    if config.test_opt {
        println!("Test mode: nothing was changed.");
    }
    rc
}

///
pub fn oe_app<'a>() -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .group(ArgGroup::new("functions").multiple(false))
        .next_help_heading("FUNCTIONS")
        .arg(
            Arg::new(arg_functions::SHOW)
                .long(arg_functions::SHOW)
                .short('r')
                .help("display the RTC time")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::GET)
                .long(arg_functions::GET)
                .help("display drift corrected RTC time")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::SET)
                .long(arg_functions::SET)
                .help("set the RTC according to --date")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::HCTOSYS)
                .long(arg_functions::HCTOSYS)
                .short('s')
                .help("set the system time from the RTC")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::SYSTOHC)
                .long(arg_functions::SYSTOHC)
                .short('w')
                .help("set the RTC from the system time")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::SYSTZ)
                .long(arg_functions::SYSTZ)
                .help("send timescale configurations to the kernel")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::ADJUST)
                .long(arg_functions::ADJUST)
                .short('a')
                .help("adjust the RTC to account for systematic drift")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::PARAM_GET)
                .long(arg_functions::PARAM_GET)
                .takes_value(true)
                .value_name("param")
                .help("display the RTC parameter")
                .action(ArgAction::Append)
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::PARAM_SET)
                .long(arg_functions::PARAM_SET)
                .takes_value(true)
                .value_names(&["param", "value"])
                .help("set the RTC parameter")
                .action(ArgAction::Append)
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::PREDICT)
                .long(arg_functions::PREDICT)
                .help("predict the drifted RTC time according to --date")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::VL_READ)
                .long(arg_functions::VL_READ)
                .help("read voltage low information")
                .group("functions"),
        )
        .arg(
            Arg::new(arg_functions::VL_CLEAR)
                .long(arg_functions::VL_CLEAR)
                .help("clear voltage low information")
                .group("functions"),
        )
        .next_help_heading("OPTIONS")
        .arg(
            Arg::new(arg_options::UTC)
                .long(arg_options::UTC)
                .short('u')
                .help("the RTC timescale is UTC")
                .conflicts_with(arg_options::LOCALTIME),
        )
        .arg(
            Arg::new(arg_options::LOCALTIME)
                .long(arg_options::LOCALTIME)
                .short('l')
                .help("the RTC timescale is Local")
                .conflicts_with(arg_options::UTC),
        )
        .arg(
            Arg::new(arg_options::RTC)
                .long(arg_options::RTC)
                .short('f')
                .help("use an alternate file to /dev/rtc0")
                .takes_value(true)
                .value_name("file")
                .action(ArgAction::Append)
                .conflicts_with(arg_options::DIRECTISA),
        )
        .arg(
            Arg::new(arg_options::DIRECTISA)
                .long(arg_options::DIRECTISA)
                .help("use the ISA bus instead of /dev/rtc0 access")
                .conflicts_with(arg_options::RTC),
        )
        .arg(
            Arg::new(arg_options::DATE)
                .long(arg_options::DATE)
                .help("date/time input for --set and --predict")
                .takes_value(true)
                .value_name("time")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(arg_options::DELAY)
                .long(arg_options::DELAY)
                .takes_value(true)
                .value_name("sec")
                .help("delay used when set new RTC time")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new(arg_options::UPDATE_DRIFT)
                .long(arg_options::UPDATE_DRIFT)
                .help("update the RTC drift factor"),
        )
        .arg(
            Arg::new(arg_options::NOADJFILE)
                .long(arg_options::NOADJFILE)
                .help("do not use /etc/adjtime")
                .conflicts_with(arg_options::ADJFILE),
        )
        .arg(
            Arg::new(arg_options::ADJFILE)
                .long(arg_options::ADJFILE)
                .takes_value(true)
                .value_name("file")
                .help("use an alternate file to /etc/adjtime")
                .action(ArgAction::Append)
                .conflicts_with(arg_options::NOADJFILE),
        )
        .arg(
            Arg::new(arg_options::TEST)
                .long(arg_options::TEST)
                .help("dry run; implies --verbose"),
        )
        .arg(
            Arg::new(arg_options::DEBUG)
                .long(arg_options::DEBUG)
                .short('D')
                .help("deprecated debug option, use verbose instead"),
        )
        .arg(
            Arg::new(arg_options::VERBOSE)
                .long(arg_options::VERBOSE)
                .short('v')
                .help("display more details"),
        )
        .arg(
            Arg::new("help")
                .long("help")
                .short('h')
                .help("display help information"),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .help("display version information"),
        )
}
