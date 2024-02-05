//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::io::Write;

use libc::EXIT_FAILURE;
use nix::{errno::Errno, sched::CpuSet};
use uucore::error::{UResult, USimpleError};

/// Parse mask into CpuSet
pub fn cpumask_parse(mask: &String) -> Result<CpuSet, Errno> {
    let mut set = CpuSet::new();
    let mut end = mask.len();
    let mut cpu = 0;
    if mask.starts_with("0x") {
        end -= 2;
    }
    for (idx, c) in mask.chars().rev().enumerate() {
        if idx >= end {
            break;
        }
        if c == ',' {
            continue;
        }
        let c_ascii: i32 = c as i32;
        let val = match c {
            '0'..='9' => c_ascii - '0' as i32,
            'a'..='f' => c_ascii + 10 - 'a' as i32,
            'A'..='F' => c_ascii + 10 - 'A' as i32,
            _ => -1,
        };
        if val < 0 {
            return Err(Errno::EINVAL);
        }
        if val & 1 > 0 {
            set.set(cpu)?;
        }
        if val & 2 > 0 {
            set.set(cpu + 1)?;
        }
        if val & 4 > 0 {
            set.set(cpu + 2)?;
        }
        if val & 8 > 0 {
            set.set(cpu + 3)?;
        }
        cpu += 4;
    }
    Ok(set)
}

/// Parse list into CpuSet
pub fn cpulist_parse(list: &String) -> Result<CpuSet, Errno> {
    let mut set = CpuSet::new();
    for p in list.split(',') {
        let range_loc = match p.find('-') {
            Some(i) => i,
            None => p.len(),
        };
        let step_loc = match p.find(':') {
            Some(i) => i,
            None => p.len(),
        };
        if range_loc > step_loc {
            return Err(Errno::EINVAL);
        }
        let a_res = p[0..range_loc].parse::<usize>();
        let b_res = match range_loc == p.len() {
            true => a_res.clone(),
            false => p[range_loc + 1..step_loc].parse::<usize>(),
        };
        let s_res = match step_loc == p.len() {
            true => Ok(1),
            false => p[step_loc + 1..].parse::<usize>(),
        };
        if a_res.is_err() || b_res.is_err() || s_res.is_err() {
            return Err(Errno::EINVAL);
        }
        let (mut a, b, s) = (a_res.unwrap(), b_res.unwrap(), s_res.unwrap());
        if a > b {
            return Err(Errno::EINVAL);
        }
        while a <= b {
            set.set(a)?;
            a += s;
        }
    }
    Ok(set)
}

/// Create string describe cpuset in mask form
pub fn cpumask_create(set: CpuSet) -> UResult<String> {
    let mut cpu = CpuSet::count() - 4;
    let mut res = Vec::new();
    let mut begin = false;
    loop {
        let mut val: u8 = 0;
        // We ensure that 'cpu < CpuSet::count()' is true
        // So call unwrap() is ok
        if set.is_set(cpu).unwrap() {
            val |= 1;
        }
        if set.is_set(cpu + 1).unwrap() {
            val |= 2;
        }
        if set.is_set(cpu + 2).unwrap() {
            val |= 4;
        }
        if set.is_set(cpu + 3).unwrap() {
            val |= 8;
        }
        if !begin && val > 0 {
            begin = true;
        }
        if begin || cpu == 0 {
            write!(&mut res, "{:x}", val)?;
            if cpu == 0 {
                break;
            }
        }
        cpu -= 4;
    }
    match String::from_utf8(res) {
        Ok(v) => Ok(v),
        _ => Err(USimpleError::new(
            EXIT_FAILURE,
            "internal error: conversion from cpuset to string failed",
        )),
    }
}

/// Create string describe cpuset in list form
pub fn cpulist_create(set: CpuSet) -> UResult<String> {
    let cpu_max: usize = CpuSet::count();
    let mut res = Vec::new();
    let mut i = 0;
    while i < cpu_max {
        // We ensure that 'cpu < CpuSet::count()' is true
        // So call unwrap() is ok
        if set.is_set(i).unwrap() {
            let mut run: usize = 0;
            for j in i + 1..cpu_max {
                if set.is_set(j).unwrap() {
                    run += 1;
                } else {
                    break;
                }
            }
            match run {
                0 => write!(&mut res, "{},", i),
                1 => write!(&mut res, "{},{},", i, i + 1),
                _ => write!(&mut res, "{}-{},", i, i + run),
            }?;
            i += run;
        }
        i += 1;
    }
    if !res.is_empty() {
        res.pop();
    }
    match String::from_utf8(res) {
        Ok(v) => Ok(v),
        _ => Err(USimpleError::new(
            EXIT_FAILURE,
            "internal error: conversion from cpuset to string failed",
        )),
    }
}
