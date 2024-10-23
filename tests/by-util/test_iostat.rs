// This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;
use std::path::Path;
const C_IOSTAT_PATH: &str = "/usr/bin/iostat";
const THRESHOLD_PERCENTAGE: f32 = 0.2;
use regex::Regex;
use serde_json::Value;

fn remove_ansi_codes(text: &str) -> String {
    let re = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(text, "").to_string()
}

fn is_integer(value: f32) -> bool {
    value.fract() == 0.0
}

fn extract_floats(input: &str) -> (Vec<f32>, Vec<Vec<f32>>) {
    let avg_cpu_line = input
        .lines()
        .skip_while(|line| !line.starts_with("avg-cpu:"))
        .nth(1)
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect::<Vec<f32>>();

    let device_lines = input
        .lines()
        .skip_while(|line| !line.starts_with("Device"))
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    (avg_cpu_line, device_lines)
}

fn extract_floats_second(input: &str) -> Vec<Vec<f32>> {
    let mut device_lines_iter = input.lines().skip_while(|line| !line.starts_with("Device"));

    let remain_lines = device_lines_iter.next().unwrap_or("");

    let device_lines = remain_lines
        .lines()
        .skip_while(|line| !line.starts_with("Device"))
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    device_lines
}

fn parse_value_with_unit(value: &str) -> Option<f32> {
    let len = value.len();
    if len == 0 {
        return None;
    }

    let (number_str, multiplier) = match &value[len - 1..] {
        "k" => (&value[..len - 1], 1_000.0),
        "M" => (&value[..len - 1], 1_000_000.0),
        "G" => (&value[..len - 1], 1_000_000_000.0),
        "%" => (&value[..len - 1], 0.01),
        _ => (value, 1.0),
    };

    number_str.parse::<f32>().ok().map(|num| num * multiplier)
}

fn extract_floats_with_unit(input: &str) -> (Vec<f32>, Vec<Vec<f32>>) {
    let mut cpu_data = Vec::new();
    let mut device_data = Vec::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("avg-cpu") {
            if let Some(cpu_line) = lines.next() {
                cpu_data.extend(
                    cpu_line
                        .split_whitespace()
                        .filter_map(parse_value_with_unit),
                );
            }
            break;
        }
    }

    while let Some(line) = lines.next() {
        if line.contains("Device") {
            break;
        }
    }

    while let Some(line) = lines.next() {
        let values = line
            .split_whitespace()
            .take_while(|s| s.parse::<f32>().is_ok())
            .filter_map(parse_value_with_unit)
            .collect::<Vec<f32>>();
        if !values.is_empty() {
            device_data.push(values);
        }
    }

    (cpu_data, device_data)
}

fn compare_f32_with_threshold(a: f32, b: f32) -> bool {
    let threshold = (a.max(b)) * THRESHOLD_PERCENTAGE;
    (a - b).abs() <= threshold
}

fn compare_f32_vector(a: &Vec<f32>, b: &Vec<f32>) {
    assert_eq!(a.len(), b.len());
    for (i, j) in a.iter().zip(b.iter()) {
        assert!(compare_f32_with_threshold(*i, *j), "{} and {}", *i, *j);
    }
}

fn compare_output(refe: &str, resu: &str, unit: bool) {
    let raw_ref = remove_ansi_codes(refe);
    let raw_res = remove_ansi_codes(resu);
    let (cpu_num_ref, device_num_ref);
    let (cpu_num_res, device_num_res);
    if unit {
        (cpu_num_ref, device_num_ref) = extract_floats_with_unit(&raw_ref);
        (cpu_num_res, device_num_res) = extract_floats_with_unit(&raw_res);
    } else {
        (cpu_num_ref, device_num_ref) = extract_floats(&raw_ref);
        (cpu_num_res, device_num_res) = extract_floats(&raw_res);
    }
    compare_f32_vector(&cpu_num_ref, &cpu_num_res);
    assert_eq!(
        device_num_ref.len(),
        device_num_res.len(),
        "{}\n{}",
        raw_res,
        raw_ref
    );
    for i in 0..device_num_ref.len() {
        assert_eq!(
            device_num_ref[i].len(),
            device_num_res[i].len(),
            "{:?}",
            device_num_res
        );
        compare_f32_vector(&device_num_ref[i], &device_num_res[i]);
    }
}

fn extract_device_names(input: &str) -> Vec<String> {
    let input = remove_ansi_codes(input);
    input
        .lines()
        .skip_while(|line| !line.contains("Device"))
        .skip(1)
        .map(|line| line.split_whitespace().last().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn test_iostat_values(test_args: &Vec<&str>, unit: bool) {
    let iostat_path = Path::new(C_IOSTAT_PATH);
    if iostat_path.exists() {
        let task = TestScenario::new(util_name!());
        let refe = task.cmd(C_IOSTAT_PATH).args(test_args).run();
        let resu = task.ucmd().args(test_args).run();
        let ref_str = refe.stdout_str();
        let res_str = resu.stdout_str();
        compare_output(ref_str, res_str, unit);
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_iostat_no_arg() {
    let test_args = Vec::new();
    test_iostat_values(&test_args, false);
}

#[test]
fn test_iostat_cpu() {
    let test_args = Vec::from(["-c"]);
    test_iostat_values(&test_args, false);
}

#[test]
fn test_iostat_device() {
    let test_args = Vec::from(["-d"]);
    test_iostat_values(&test_args, false);
}

#[test]
fn test_iostat_interval_count() {
    let iostat_path = Path::new(C_IOSTAT_PATH);
    let test_args = &["1", "3"];
    if iostat_path.exists() {
        let task = TestScenario::new(util_name!());
        let refe = task.cmd(C_IOSTAT_PATH).args(test_args).run();
        let resu = task.ucmd().args(test_args).run();
        let reflen = refe
            .stdout_str()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();

        let reslen = resu
            .stdout_str()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        assert_eq!(reflen, reslen);
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_iostat_directory() {
    let test_args = Vec::from(["-f", "/tmp"]);
    test_iostat_values(&test_args, false);
    let test_args2 = Vec::from(["+f", "/tmp"]);
    test_iostat_values(&test_args2, false);
}

#[test]
fn test_iostat_json() {
    let test_args = Vec::from(["-o", "JSON"]);
    let task = TestScenario::new(util_name!());
    let resu = task.ucmd().args(&test_args).succeeds();
    let res_str = resu.stdout_str();
    let _: Value = serde_json::from_str(res_str).unwrap();
}

#[test]
fn test_iostat_human() {
    let test_args = Vec::from(["--human"]);
    test_iostat_values(&test_args, true);
}

#[test]
fn test_iostat_pretty() {
    let test_args = Vec::from(["--pretty"]);
    test_iostat_values(&test_args, true);
}

#[test]
fn test_iostat_h() {
    let test_args = Vec::from(["-h"]);
    test_iostat_values(&test_args, true);
}

#[test]
fn test_iostat_mapper() {
    let iostat_path = Path::new(C_IOSTAT_PATH);
    let test_args = &Vec::from(["-N"]);
    if iostat_path.exists() {
        let task = TestScenario::new(util_name!());
        let refe = task.cmd(C_IOSTAT_PATH).args(test_args).run();
        let resu = task.ucmd().args(test_args).run();
        let ref_str = refe.stdout_str();
        let res_str = resu.stdout_str();
        let ref_devices = extract_device_names(&ref_str);
        let res_devices = extract_device_names(&res_str);
        assert_eq!(ref_devices.len(), res_devices.len());
        for (i, _) in ref_devices.iter().enumerate() {
            assert_eq!(ref_devices[i], res_devices[i]);
        }
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_iostat_dec() {
    let test_args = &Vec::from(["--dec=0"]);
    let task = TestScenario::new(util_name!());
    let resu = task.ucmd().args(test_args).run();
    let res_str = resu.stdout_str();
    let raw_str = remove_ansi_codes(&res_str);
    let (cpu_num_res, _) = extract_floats(&raw_str);
    for i in 0..cpu_num_res.len() {
        assert!(is_integer(cpu_num_res[i]));
    }
}

#[test]
fn test_iostat_p() {
    let test_args = Vec::from(["-p", "ALL"]);
    test_iostat_values(&test_args, false);
}

#[test]
fn test_iostat_ext() {
    let test_args = Vec::from(["-x"]);
    test_iostat_values(&test_args, false);
}

#[test]
fn test_iostat_short() {
    let test_args = Vec::from(["-s"]);
    test_iostat_values(&test_args, false);
}

#[test]
fn test_iostat_y() {
    let test_args = Vec::from(["-y", "1", "3"]);
    let task = TestScenario::new(util_name!());
    let resu = task.ucmd().args(&test_args).succeeds();
    let res_str = resu.stdout_str();
    let raw_str = remove_ansi_codes(&res_str);
    let print_count = raw_str.matches("avg-cpu").count();
    assert_eq!(print_count, 2);
}

#[test]
fn test_iostat_z() {
    let test_args = Vec::from(["-z", "1", "3"]);
    let task = TestScenario::new(util_name!());
    let resu = task.ucmd().args(&test_args).succeeds();
    let res_str = resu.stdout_str();
    let raw_str = remove_ansi_codes(&res_str);
    let device_num_res = extract_floats_second(&raw_str);
    for i in 0..device_num_res.len() {
        let mut sum = 0.0;
        for j in 0..device_num_res[i].len() {
            sum += device_num_res[i][j];
        }
        assert_ne!(sum, 0.0);
    }
}

#[test]
fn test_iostat_time() {
    let iostat_path = Path::new(C_IOSTAT_PATH);
    let test_args = &["-t"];
    if iostat_path.exists() {
        let task = TestScenario::new(util_name!());
        let refe = task.cmd(C_IOSTAT_PATH).args(test_args).run();
        let resu = task.ucmd().args(test_args).run();
        let reflen = refe
            .stdout_str()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();

        let reslen = resu
            .stdout_str()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        assert_eq!(reflen, reslen);
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}
