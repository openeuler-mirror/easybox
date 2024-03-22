//! This file is part of the easybox package.
//
// (c) Maxwell Xu <eureka0xff@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

// DIRECTISA ACCESS IS FOR X86 BASED ARCH ONLY
use crate::{BrokenTime, ClockOperations, HwclockConfig};
use nix::libc::{syscall, SYS_iopl};
use std::arch::asm;
use uucore::error::{UResult, USimpleError};

macro_rules! bcd2bin {
    ($val:expr) => {
        $val = ($val & 15) + (($val >> 4) * 10)
    };
}

macro_rules! bin2bcd {
    ($val:expr) => {
        $val = (($val / 10) << 4) + ($val % 10)
    };
}

///
const IOPL_NOT_IMPLEMENTED: i32 = -2;
///
const CLOCK_CTL_ADDR: u16 = 0x70;
///
const CLOCK_DATA_ADDR: u16 = 0x71;

// Read data from the specified register in CMOS (Complementary Metal-Oxide-Semiconductor)
fn cmos_read(reg: u8) -> u8 {
    let mut ret: u8;
    unsafe {
        // Write the register to the control address
        asm!(
            "out dx, al",
            in("al") reg,
            in("dx") CLOCK_CTL_ADDR
        );
        // Read data from the data address
        asm!(
            "in al, dx",
            out("al") ret,
            in("dx") CLOCK_DATA_ADDR
        );
    }
    ret
}

///
fn cmos_write(reg: u8, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("al") reg,
            in("dx") CLOCK_CTL_ADDR
        );
        asm!(
            "out dx, al",
            in("al") val,
            in("dx") CLOCK_DATA_ADDR
        )
    }
}

///
fn cmos_clock_busy() -> bool {
    // The clock is undefined (being updated) if bit 7 of byte 10 is set.
    cmos_read(10) & 0x80 != 0
}

///
pub fn read_hardware_clock_cmos(_config: &HwclockConfig) -> UResult<BrokenTime> {
    let mut pmbit = 0;
    let mut time = BrokenTime::default();
    let mut status;
    loop {
        if !cmos_clock_busy() {
            time.tm_sec = cmos_read(0) as i32;
            time.tm_min = cmos_read(2) as i32;
            time.tm_hour = cmos_read(4) as i32;
            time.tm_wday = cmos_read(6) as i32;
            time.tm_mday = cmos_read(7) as i32;
            time.tm_mon = cmos_read(8) as i32;
            time.tm_year = cmos_read(9) as i32;
            status = cmos_read(11);
            if time.tm_sec == cmos_read(0) as i32 {
                break;
            }
        }
    }

    if status != 0x04 {
        bcd2bin!(time.tm_sec);
        bcd2bin!(time.tm_min);
        pmbit = time.tm_hour & 0x80;
        time.tm_hour &= 0x7f;
        bcd2bin!(time.tm_hour);
        bcd2bin!(time.tm_wday);
        bcd2bin!(time.tm_mday);
        bcd2bin!(time.tm_mon);
        bcd2bin!(time.tm_year);
    }

    // util-linux/sys-utils/hwclock_cmos.c
    // if time.tm_year is in the range [69-99], it refers to 1969-1999
    // if time.tm_year is in the range [0-68], it refers to 2000-2068
    time.tm_wday -= 1;
    time.tm_mon -= 1;
    if time.tm_year < 69 {
        time.tm_year += 100;
    }
    if pmbit != 0 {
        time.tm_hour += 12;
        if time.tm_hour == 24 {
            time.tm_hour = 0;
        }
    }
    time.tm_isdst = -1; // don't know whether it's daylight
    Ok(time)
}

///
pub fn set_hardware_clock_cmos(_config: &HwclockConfig, time: &BrokenTime) -> UResult<()> {
    let mut pmbit = 0;
    let mut tm = time.to_owned();
    let save_control = cmos_read(11);
    cmos_write(11, save_control | 0x80);
    let save_freq_select = cmos_read(10);
    cmos_write(10, save_freq_select | 0x70);
    tm.tm_year %= 100;
    tm.tm_mon += 1;
    tm.tm_wday += 1;

    // 12hr mode
    if save_control != 0x02 {
        if tm.tm_hour == 0 {
            tm.tm_hour = 24;
        }
        if tm.tm_hour > 12 {
            tm.tm_hour -= 12;
            pmbit = 0x80;
        }
    }

    // BCD mode
    if save_control != 0x04 {
        bin2bcd!(tm.tm_sec);
        bin2bcd!(tm.tm_min);
        bin2bcd!(tm.tm_hour);
        bin2bcd!(tm.tm_wday);
        bin2bcd!(tm.tm_mday);
        bin2bcd!(tm.tm_mon);
        bin2bcd!(tm.tm_year);
    }

    cmos_write(0, tm.tm_sec as u8);
    cmos_write(2, tm.tm_min as u8);
    cmos_write(4, (tm.tm_hour | pmbit) as u8);
    cmos_write(6, tm.tm_wday as u8);
    cmos_write(7, tm.tm_mday as u8);
    cmos_write(8, tm.tm_mon as u8);
    cmos_write(9, tm.tm_year as u8);

    cmos_write(11, save_control);
    cmos_write(10, save_freq_select);
    Ok(())
}

/// assembly RTC access needs to set IO permissions
pub fn get_permissions_cmos(_config: &HwclockConfig) -> UResult<()> {
    let rc = unsafe { syscall(SYS_iopl, 3) } as i32;
    match rc {
        0 => Ok(()),
        IOPL_NOT_IMPLEMENTED => Err(USimpleError::new(rc, "ISA port access is not implemented")),
        _ => Err(USimpleError::new(rc, "iopl() port access failed")),
    }
}

///
pub fn synchronize_to_clock_tick_cmos(_config: &HwclockConfig) -> UResult<()> {
    const WAIT_FOR_RISE_MAX_COUNT: i32 = 10_000_000;
    const WAIT_FOR_FALL_MAX_COUNT: i32 = 2_000_000;
    let mut count = 0;
    while !cmos_clock_busy() {
        count += 1;
        if count > WAIT_FOR_RISE_MAX_COUNT {
            return Err(USimpleError::new(
                1,
                "Waiting for cmos clock rise timed out",
            ));
        }
    }
    count = 0;
    while !cmos_clock_busy() {
        count += 1;
        if count > WAIT_FOR_FALL_MAX_COUNT {
            return Err(USimpleError::new(
                1,
                "Waiting for cmos clock fall timed out",
            ));
        }
    }
    Ok(())
}

///
const CMOS_CLOCK_OPERATIONS: &ClockOperations = &ClockOperations {
    interface_name: "Using direct ISA access to the clock",
    get_permissions: get_permissions_cmos,
    read_hardware_clock: read_hardware_clock_cmos,
    set_hardware_clock: set_hardware_clock_cmos,
    synchronize_to_clock_tick: synchronize_to_clock_tick_cmos,
};

///
pub fn probe_for_directisa(_config: &HwclockConfig) -> Option<&'static ClockOperations> {
    Some(CMOS_CLOCK_OPERATIONS)
}
