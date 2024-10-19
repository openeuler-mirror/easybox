//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use regex::Regex;
use std::path::Path;

const C_SYSCTL_PATH: &str = "/usr/sbin/sysctl";

/// Sysctl in rust is implemented based on version 4.0.4.189-21f6.
/// Some tests are not applicable to lower versions.
fn get_sysctl_version(bin_path: &str, test_scenario: &TestScenario) -> (u32, u32, u32) {
    if Path::new(bin_path).exists() {
        let result = test_scenario.cmd(bin_path).arg("-V").run();
        let version_pattern = Regex::new(r"(\d+\.\d+\.\d+)").unwrap();

        if let Some(captures) = version_pattern.captures(result.stdout_str()) {
            if let Some(version) = captures.get(1) {
                let version_vec = version
                    .as_str()
                    .split_whitespace()
                    .last()
                    .unwrap()
                    .split('.')
                    .take(3)
                    .collect::<Vec<_>>();
                return (
                    version_vec[0].parse::<u32>().unwrap_or_default(),
                    version_vec[1].parse::<u32>().unwrap_or_default(),
                    version_vec[2].parse::<u32>().unwrap_or_default(),
                );
            };
        }
    }

    (0, 0, 0)
}

fn expected_result_brief(
    bin_path: &str,
    test_scenario: &TestScenario,
    test_args: &[&str],
) -> std::result::Result<CmdResult, String> {
    if !Path::new(bin_path).exists() {
        return Err(format!("Executable file {} not exist.", bin_path));
    }

    let result = test_scenario.cmd(bin_path).args(test_args).run();

    Ok(CmdResult::new(
        bin_path.to_string(),
        Some(test_scenario.util_name.clone()),
        Some(result.tmpd()),
        Some(result.code()),
        result.succeeded(),
        result.stdout(),
        result.stderr(),
    ))
}

#[test]
fn test_sysctl_with_no_arguments() {
    let test_scenario = TestScenario::new(util_name!());
    test_scenario
        .ucmd()
        .fails()
        .stderr_contains("sysctl [options] [variable[=value] ...]");
}

#[test]
fn test_sysctl_read() {
    let test_scenario = TestScenario::new(util_name!());

    let mut test_args_vec = vec![
        // Test for reading a variable using slash delimiter.
        vec!["kernel/hostname"],
        // Test for reading a variable using dot delimiter.
        vec!["kernel.hostname"],
        // Test for reading a variable suppress key.
        vec!["-n", "kernel.hostname"],
        vec!["--values", "kernel.hostname"],
        // Test for reading a variable suppress value.
        vec!["-N", "kernel.hostname"],
        vec!["--names", "kernel.hostname"],
        // Test for reading a variable and printing value without new line.
        vec!["-b", "kernel.hostname"],
        vec!["--binary", "kernel.hostname"],
        // Test for pattern option.
        vec!["-a", "--pattern", "forward"],
        vec!["-a", "--pattern", "forward$"],
        vec!["-a", "--pattern", "'net.ipv4.conf.(eth|wlan)0.arp'"],
    ];

    let sysctl_version = get_sysctl_version(C_SYSCTL_PATH, &test_scenario);

    if sysctl_version >= (4, 0, 2) {
        // Test for reading values from all system directories.
        test_args_vec.push(vec!["--system"]);
        test_args_vec.push(vec!["--pattern", "'^net.ipv6'", "--system"]);
    }

    if sysctl_version >= (4, 0, 4) {
        // Test for reading using a path traversal.
        test_args_vec.push(vec!["/../../etc/passwd"]);
    }

    // Latest features, not released.
    // Skip stat_refresh with --all.
    if sysctl_version > (4, 0, 4) {
        test_args_vec.extend(vec![
            // Test for reading all variables.
            // Some values ​​are dynamically changing so use the -N option.
            vec!["-a", "-N"],
            vec!["--all", "-N"],
            vec!["-A", "-N"],
            vec!["-X", "-N"],
            // Test for include deprecated parameters to listing.
            vec!["-a", "-N", "--deprecated"],
        ]);
    }

    for test_args in test_args_vec.iter() {
        let expected_result = unwrap_or_return!(expected_result_brief(
            C_SYSCTL_PATH,
            &test_scenario,
            test_args
        ));

        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    }
}

#[test]
fn test_sysctl_write() {
    let test_scenario = TestScenario::new(util_name!());
    let mut test_args_vec: Vec<Vec<&str>> = Vec::new();

    let sysctl_version = get_sysctl_version(C_SYSCTL_PATH, &test_scenario);

    if sysctl_version < (4, 0, 2) {
        println!("Test skipped: C sysctl version is too low.");
        return;
    }

    if sysctl_version >= (4, 0, 2) {
        test_args_vec.extend(vec![
            // Test for writing from command line.
            vec!["--dry-run", "kernel.hostname=procps-test"],
            vec!["--dry-run", "-w", "kernel.hostname=procps-test"],
            // Test for writing from command line using slash.
            vec!["--dry-run", "kernel/hostname=procps-test"],
            vec!["--dry-run", "--write", "kernel/hostname=procps-test"],
            // Test for writing from file with slashes.
            vec!["--dry-run", "-psysctl_slash_test.conf"],
            // Test for writing unwritable file.
            vec!["-q", "kernel.hostname=procpstest"],
            // Test for writing unwritable file ignored.
            vec!["--quiet", "--", "-kernel.hostname=procpstest"],
            // Test for writing above /proc.
            vec!["/../../../etc=1"],
        ]);
    }

    // Latest features, not released.
    if sysctl_version > (4, 0, 4) {
        test_args_vec.extend(vec![
            // Test for writing from configuration file.
            vec!["--dry-run", "-f", "sysctl_glob_test.conf"],
            vec!["--dry-run", "--load", "sysctl_glob_test.conf"],
        ]);
    }

    for test_args in test_args_vec.iter() {
        let expected_result = unwrap_or_return!(expected_result_brief(
            C_SYSCTL_PATH,
            &test_scenario,
            test_args
        ));

        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    }
}

#[test]
fn test_sysctl_others() {
    let test_scenario = TestScenario::new(util_name!());

    let test_args_vec = [
        // Test for does nothing options.
        vec!["-o", "-x"],
        // Test for options -N and -q cannot coexist.
        vec!["-N", "-q"],
    ];

    for test_args in test_args_vec.iter() {
        let expected_result = unwrap_or_return!(expected_result_brief(
            C_SYSCTL_PATH,
            &test_scenario,
            test_args
        ));

        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    }
}
