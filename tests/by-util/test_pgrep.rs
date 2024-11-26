//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use rand::Rng;
use regex::Regex;
use std::{fs, os::unix::fs::symlink, path::Path, process, sync::Mutex, time::Duration};

const C_PGREP_PATH: &str = "/usr/bin/pgrep";
const C_SLEEP_PATH: &str = "/usr/bin/sleep";

const EXIT_USAGE: i32 = 2;
const MULTIPLE_PIDS: &str = "(?m)^[1-9][0-9]*$";

static MUX: Mutex<()> = Mutex::new(());

/// pgrep in rust is implemented based on version 4.0.4.189-21f6.
/// Some tests are not applicable to lower versions.
fn get_pgrep_version(bin_path: &str, test_scenario: &TestScenario) -> (u32, u32, u32) {
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
fn test_pgrep_with_no_arguments() {
    let test_scenario = TestScenario::new(util_name!());
    test_scenario
        .ucmd()
        .fails()
        .code_is(EXIT_USAGE)
        .no_stdout()
        .stderr_contains("pgrep: no matching criteria specified");
}

#[test]
fn test_pgrep() {
    let _lock = MUX.lock();
    let test_scenario = TestScenario::new(util_name!());

    let my_pid = &process::id().to_string();
    let not_ppid = &(process::id() + 1).to_string();
    let multiple_ppids = &format!("{},{}", my_pid, not_ppid);

    let uid_cmd = test_scenario.cmd("id").arg("-u").run();
    let uid = uid_cmd.stdout_str().trim();
    let not_uid = &(uid.parse::<u32>().unwrap() + 1).to_string();
    let multiple_uids = &format!("{},{}", uid, not_uid);

    let gid_cmd = test_scenario.cmd("id").arg("-g").run();
    let gid = gid_cmd.stdout_str().trim();
    let not_gid = &(gid.parse::<u32>().unwrap() + 1).to_string();
    let multiple_gids = &format!("{},{}", gid, not_gid);

    let test_proc_comm: &str = &format!("tp_{}", rand::thread_rng().gen_range(100000..=999999));
    let test_proc_path = &format!("/tmp/{}", test_proc_comm);
    if symlink(C_SLEEP_PATH, test_proc_path).is_err() {
        panic!("symlink failed.");
    }

    let test_proc_trim = &test_proc_comm[..7];
    let test_proc_upper = &test_proc_comm.to_uppercase();
    let test_proc_arg_1 = "10";
    let test_proc_arg_2 = "11";
    let test_proc_full = &format!("{} {}", test_proc_path, test_proc_arg_1);
    let mut test_proc_1 = test_scenario
        .cmd(test_proc_path)
        .arg(test_proc_arg_1)
        .run_no_wait();
    let mut test_proc_2 = test_scenario
        .cmd(test_proc_path)
        .arg(test_proc_arg_2)
        .run_no_wait();
    let _ = fs::remove_file(test_proc_path);
    std::thread::sleep(Duration::from_millis(50));

    let test_proc_1_sid_cmd = test_scenario
        .cmd("ps")
        .args(&["--no-headers", "-o", "sid", &test_proc_1.id().to_string()])
        .run();
    let test_proc_1_sid: &str = test_proc_1_sid_cmd.stdout_str().trim();
    let not_test_proc_1_sid = &(test_proc_1_sid.parse::<u32>().unwrap() + 1).to_string();
    let multiple_sids = &format!("{},{}", test_proc_1_sid, not_test_proc_1_sid);

    let test_proc_1_pgid_cmd = test_scenario
        .cmd("ps")
        .args(&["--no-headers", "-o", "pgid", &test_proc_1.id().to_string()])
        .run();
    let test_proc_1_pgid: &str = test_proc_1_pgid_cmd.stdout_str().trim();
    let not_test_proc_1_pgid = &(test_proc_1_pgid.parse::<u32>().unwrap() + 1).to_string();
    let multiple_pgids = &format!("{},{}", test_proc_1_pgid, not_test_proc_1_pgid);

    // Tests in procps-ng.
    let mut test_args_vec = vec![
        // Test for finding both test pids.
        vec![test_proc_comm],
        // Test for counting 2 test pids.
        vec!["-c", test_proc_comm],
        // Test for pgrep with : delimiter.
        vec!["-d", ":", test_proc_comm],
        // Test for matching against full process name.
        vec!["-f", test_proc_full],
        // Test for pgrep with matching gid.
        vec!["-G", gid, test_proc_comm],
        // Test for pgrep with not matching gid.
        vec!["-G", not_gid, test_proc_comm],
        // Test for pgrep with process name.
        vec!["-l", test_proc_comm],
        // Test for pgrep with full command line.
        vec!["-af", test_proc_path],
        // Test for finding newest test pid.
        vec!["-n", test_proc_comm],
        // Test for finding oldest test pid.
        vec!["-o", test_proc_comm],
        // Test for pgrep matches with parent pid.
        vec!["-P", my_pid, test_proc_comm],
        // Test for pgrep doesn't match with bogus parent pid.
        vec!["-P", not_ppid, test_proc_comm],
        // Test for pgrep matches with its own sid.
        vec!["-s", test_proc_1_sid, test_proc_comm],
        // Test for pgrep doesn't match with bogus sid.
        vec!["-s", not_test_proc_1_sid, test_proc_comm],
        // Cargo test does not run in a tty, so this test is skipped.
        // vec!["-t", $tty, test_proc_comm],
        // Test for pgrep doesn't match with bogus tty.
        vec!["-t", "glass", test_proc_comm],
        // Test for pgrep with matching euid.
        vec!["-u", uid, test_proc_comm],
        // Test for pgrep with no matching euid.
        vec!["-u", not_uid, test_proc_comm],
        // Test for pgrep with matching uid.
        vec!["-U", uid, test_proc_comm],
        // Test for pgrep with no matching uid.
        vec!["-U", not_uid, test_proc_comm],
        // Test for pgrep matches on substring.
        vec![test_proc_trim],
        // Test for pgrep matches full string with exact.
        vec!["-x", test_proc_comm],
        // Test for pgrep does not match substring with exact.
        vec!["-x", test_proc_trim],
    ];

    let pgrep_version = get_pgrep_version(C_PGREP_PATH, &test_scenario);
    println!(
        "C pgrep version is {}.{}.{}",
        pgrep_version.0, pgrep_version.1, pgrep_version.2
    );
    if pgrep_version >= (4, 0, 0) {
        // Test for pgrep with long non-matching pattern gives warning.
        test_args_vec.push(vec!["gnome-session-bi"]);
    }

    // New tests.
    let mut new_test_args_vec = vec![
        // Test for counting but no matching.
        vec!["-c", "no_matching"],
        // Test for multiple delimiters.
        vec!["-d", ":", "-d", "_", test_proc_comm],
        vec!["-d", "_", "-d", ":", test_proc_comm],
        // Test for pgrep with matching pgid.
        vec!["-g", test_proc_1_pgid, test_proc_comm],
        // Test for pgrep with not matching pgid.
        vec!["-g", not_test_proc_1_pgid, test_proc_comm],
        // Test for multiple pgids.
        vec!["-g", multiple_pgids, test_proc_comm],
        // Test for multiple gids.
        vec!["-G", multiple_gids, test_proc_comm],
        // Test for case insensitively.
        vec!["-i", test_proc_upper],
        // Test for finding newest but no matching.
        vec!["-n", "no_matching"],
        // Test for finding oldest but no matching.
        vec!["-o", "no_matching"],
        // Test for older with pattern.
        vec!["-O", "0", test_proc_comm],
        // Test for older with pattern but no matching.
        vec!["-O", "0", "no_matching"],
        // Test for multiple parent pids.
        vec!["-P", multiple_ppids, test_proc_comm],
        // Test for runstate.
        vec!["-r", "D", test_proc_comm],
        vec!["-r", "S", test_proc_comm],
        // Test for multiple runstates.
        vec!["-r", "DS", test_proc_comm],
        vec!["-r", "SD", test_proc_comm],
        // Test for multiple sids.
        vec!["-s", multiple_sids, test_proc_comm],
        // Test for multiple euids.
        vec!["-u", multiple_uids, test_proc_comm],
        // Test for multiple uids.
        vec!["-U", multiple_uids, test_proc_comm],
        // Test for too many patterns.
        vec!["pattern1", "pattern2"],
    ];

    if pgrep_version >= (4, 0, 3) {
        // Test for ignore ancestors.
        new_test_args_vec.push(vec!["-A", "init"]);
    }

    test_args_vec.extend(new_test_args_vec);

    for test_args in test_args_vec.iter() {
        let expected_result = unwrap_or_return!(expected_result_brief(
            C_PGREP_PATH,
            &test_scenario,
            test_args
        ));
        println!("expected stdout: {}", expected_result.stdout_str());
        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    }

    // The results of these tests are uncontrollable and may change over time.
    // So they are not tested by comparison with pgrep in C.
    let mut test_args_vec_uncontrollable = vec![
        // Test for older with no pattern.
        vec!["-O", "0"],
        // Test for controlling terminal.
        vec!["-t", "pts/10", "-v"],
        vec!["-t", "tty10", "-v"],
        vec!["-t", "ttyS10", "-v"],
        // Test for multiple controlling terminal.
        vec!["-t", "pts/10,pts/11", "-v"],
        // Test for unknown controlling terminal.
        vec!["-t", "?", "-v"],
        // Test for inverse.
        vec!["-v", "-P", my_pid, test_proc_comm],
    ];

    if pgrep_version >= (4, 0, 0) {
        // Test for cgroup.
        test_args_vec_uncontrollable.push(vec!["--cgroup", "/", "-v"]);
        test_args_vec_uncontrollable.push(vec!["--cgroup", "/init.scope", "-v"]);
        // Test for env.
        test_args_vec_uncontrollable.push(vec!["--env", "SHELL=/bin/bash", "-v"]);
    }

    for test_args in test_args_vec_uncontrollable.iter() {
        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_matches(&Regex::new(MULTIPLE_PIDS).unwrap());
    }

    let _ = test_proc_1.kill();
    let _ = test_proc_2.kill();
}

#[test]
fn test_pgrep_invalid_args() {
    let _lock = MUX.lock();
    let test_scenario = TestScenario::new(util_name!());

    // Test for invalid pattern.
    test_scenario.ucmd().arg("{(*").fails().code_is(EXIT_USAGE);

    let test_args_vec = vec![
        // Test for invalid pgroup.
        vec!["-g", "invalid_pgid"],
        // Test for invalid group.
        vec!["-G", "invalid_gid"],
        // Test for invalid parent.
        vec!["-P", "invalid_ppid"],
        // Test for invalid runstates.
        vec!["-r", "invalid_state"],
        // Test for invalid session.
        vec!["-s", "invalid_sid"],
        // Test for invalid terminal.
        vec!["-t", "invalid_terminal"],
        vec!["-t", "/dev/pts/1"],
        // Test for invalid euid.
        vec!["-u", "invalid_id"],
        // Test for invalid uid.
        vec!["-U", "invalid_id"],
    ];

    for test_args in test_args_vec.iter() {
        let expected_result = unwrap_or_return!(expected_result_brief(
            C_PGREP_PATH,
            &test_scenario,
            test_args
        ));
        println!("expected stderr: {}", expected_result.stderr_str());
        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    }

    let test_args_vec_uncontrollable = [
        // Test for invalid seconds.
        vec!["-O", "invalid_seconds"],
    ];

    for test_args in test_args_vec_uncontrollable.iter() {
        test_scenario
            .ucmd()
            .args(test_args)
            .run()
            .stdout_matches(&Regex::new(MULTIPLE_PIDS).unwrap());
    }
}

#[test]
fn test_pgrep_pidfile_from_stdin() {
    let _lock = MUX.lock();
    let test_scenario = TestScenario::new(util_name!());

    if !Path::new(C_PGREP_PATH).exists() {
        println!("Executable file {} not exist.", C_PGREP_PATH);
        return;
    }

    let pgrep_version = get_pgrep_version(C_PGREP_PATH, &test_scenario);
    println!(
        "C pgrep version is {}.{}.{}",
        pgrep_version.0, pgrep_version.1, pgrep_version.2
    );
    if pgrep_version <= (4, 0, 4) {
        println!("Test for reading pidfile from stdin skipped.");
        return;
    }

    let test_proc_comm: &str = &format!("tp_{}", rand::thread_rng().gen_range(100000..=999999));
    let test_proc_path = &format!("/tmp/{}", test_proc_comm);
    if symlink(C_SLEEP_PATH, test_proc_path).is_err() {
        panic!("symlink failed.");
    }
    let mut test_proc_1 = test_scenario.cmd(test_proc_path).arg("10").run_no_wait();
    let mut test_proc_2 = test_scenario.cmd(test_proc_path).arg("11").run_no_wait();
    let _ = fs::remove_file(test_proc_path);
    std::thread::sleep(Duration::from_millis(50));

    let pidfile_contents = [
        // Valid and matched pid.
        test_proc_1.id().to_string(),
        // Valid but not matched pid.
        (test_proc_1.id() + 10).to_string(),
        // Not Valid pid.
        format!("{},{}", test_proc_1.id(), test_proc_1.id() + 10),
    ];
    let test_args = vec!["-F", "-", test_proc_comm];

    for pidfile_content in pidfile_contents.iter() {
        let expected_result = test_scenario
            .cmd(C_PGREP_PATH)
            .args(&test_args)
            .pipe_in(pidfile_content.as_str())
            .run();
        println!("expected stdout: {}", expected_result.stdout_str());
        println!("expected stderr: {}", expected_result.stderr_str());
        test_scenario
            .ucmd()
            .args(&test_args)
            .pipe_in(pidfile_content.as_str())
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    }

    let _ = test_proc_1.kill();
    let _ = test_proc_2.kill();
}

#[test]
fn test_pgrep_pidfile_from_file() {
    let _lock = MUX.lock();
    let test_scenario = TestScenario::new(util_name!());

    if !Path::new(C_PGREP_PATH).exists() {
        println!("Executable file {} not exist.", C_PGREP_PATH);
        return;
    }

    let test_proc_comm: &str = &format!("tp_{}", rand::thread_rng().gen_range(100000..=999999));
    let test_proc_path = &format!("/tmp/{}", test_proc_comm);
    if symlink(C_SLEEP_PATH, test_proc_path).is_err() {
        panic!("symlink failed.");
    }
    let mut test_proc_1 = test_scenario.cmd(test_proc_path).arg("10").run_no_wait();
    let mut test_proc_2 = test_scenario.cmd(test_proc_path).arg("11").run_no_wait();
    let _ = fs::remove_file(test_proc_path);
    std::thread::sleep(Duration::from_millis(50));

    let pidfile_name = "pgrep_pidfile";
    let pidfile_contents = [
        // Valid and matched pid.
        test_proc_1.id().to_string(),
        // Valid but not matched pid.
        (test_proc_1.id() + 10).to_string(),
        // Not Valid pid.
        format!("{},{}", test_proc_1.id(), test_proc_1.id() + 10),
    ];
    let test_args = vec!["-F", pidfile_name, test_proc_comm];

    for pidfile_content in pidfile_contents.iter() {
        test_scenario
            .cmd("tee")
            .arg(pidfile_name)
            .pipe_in(pidfile_content.as_str())
            .run();

        let expected_result = test_scenario.cmd(C_PGREP_PATH).args(&test_args).run();
        println!("expected stdout: {}", expected_result.stdout_str());
        println!("expected stderr: {}", expected_result.stderr_str());
        test_scenario
            .ucmd()
            .args(&test_args)
            .run()
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());

        test_scenario.cmd("rm").args(&["-rf", pidfile_name]).run();
    }

    let _ = test_proc_1.kill();
    let _ = test_proc_2.kill();
}
