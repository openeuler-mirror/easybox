//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use std::{path::Path, time::SystemTime};

const C_USLEEP_PATH: &str = "/usr/bin/usleep";
const SLEEP_OVERHEAD_MICROSECONDS: u128 = 50000;

#[test]
fn test_usleep_1000000() {
    // 1000000 (dec) = 3641100 (oct) = f4240(hex)
    for sleep_duration in ["1000000", "03641100", "0xf4240", "0XF4240"] {
        let test_scenario = TestScenario::new(util_name!());
        let test_args = &[sleep_duration];

        let (expected_code, expected_stdout, expected_stderr) = if Path::new(C_USLEEP_PATH).exists()
        {
            let c_result = test_scenario.cmd(C_USLEEP_PATH).args(test_args).succeeds();
            (
                c_result.code(),
                c_result.stdout_str().to_owned(),
                c_result.stderr_str().to_owned(),
            )
        } else {
            (0, String::from(""), String::from("warning: usleep is deprecated, and will be removed in near future!\nwarning: use \"sleep 1\" instead...\n"))
        };

        let start_time = SystemTime::now();
        test_scenario
            .ucmd()
            .args(test_args)
            .succeeds()
            .code_is(expected_code)
            .stdout_is(expected_stdout)
            .stderr_is(expected_stderr);
        let end_time = SystemTime::now();
        if let Ok(time_diff) = end_time.duration_since(start_time) {
            let time_diff_micros = time_diff.as_micros();
            if time_diff_micros.abs_diff(1000000) > SLEEP_OVERHEAD_MICROSECONDS {
                panic!(
                    "argument {}, expect sleep for 1000000 microseconds, actually sleep for {} microseconds", sleep_duration,
                    time_diff_micros
                );
            }
        }
    }
}

#[test]
fn test_usleep_oot() {
    for oot_option in ["-o", "--oot"] {
        let test_scenario = TestScenario::new(util_name!());
        let test_args = &[oot_option];

        let (expected_code, expected_stdout, expected_stderr) = if Path::new(C_USLEEP_PATH).exists()
        {
            let c_result = test_scenario.cmd(C_USLEEP_PATH).args(test_args).succeeds();
            (
                c_result.code(),
                c_result.stdout_str().to_owned(),
                c_result.stderr_str().to_owned(),
            )
        } else {
            (0, String::from("oot says hey!\n"), String::from(""))
        };

        test_scenario
            .ucmd()
            .args(test_args)
            .succeeds()
            .code_is(expected_code)
            .stdout_is(expected_stdout)
            .stderr_is(expected_stderr);
    }
}

#[test]
fn test_usleep_no_arg() {
    let test_scenario = TestScenario::new(util_name!());

    let (expected_code, expected_stdout, expected_stderr) = if Path::new(C_USLEEP_PATH).exists() {
        let c_result = test_scenario.cmd(C_USLEEP_PATH).succeeds();
        (
            c_result.code(),
            c_result.stdout_str().to_owned(),
            c_result.stderr_str().to_owned(),
        )
    } else {
        (0, String::from(""), String::from("warning: usleep is deprecated, and will be removed in near future!\nwarning: use \"sleep 1e-06\" instead...\n"))
    };

    let start_time = SystemTime::now();
    test_scenario
        .ucmd()
        .succeeds()
        .code_is(expected_code)
        .stdout_is(expected_stdout)
        .stderr_is(expected_stderr);
    let end_time = SystemTime::now();
    if let Ok(time_diff) = end_time.duration_since(start_time) {
        let time_diff_micros = time_diff.as_micros();
        if time_diff_micros.abs_diff(1) > SLEEP_OVERHEAD_MICROSECONDS {
            panic!(
                "expect sleep for 1 microsecond, actually sleep for {} microseconds",
                time_diff_micros
            );
        }
    }
}

#[test]
fn test_usleep_extra_operand() {
    let test_scenario = TestScenario::new(util_name!());
    let test_args = &["1000000", "2000000"];

    let (expected_code, expected_usage_error) = if Path::new(C_USLEEP_PATH).exists() {
        let c_result = test_scenario.cmd(C_USLEEP_PATH).args(test_args).fails();
        let usage_error_msg: Vec<&str> = c_result.stderr_str().split(": ").collect();
        (
            c_result.code(),
            usage_error_msg
                .last()
                .unwrap()
                .strip_suffix("\n")
                .unwrap()
                .to_owned(),
        )
    } else {
        (
            2,
            String::from("exactly one argument (number of microseconds) must be used"),
        )
    };

    test_scenario
        .ucmd()
        .args(test_args)
        .fails()
        .code_is(expected_code)
        .usage_error(expected_usage_error);
}
