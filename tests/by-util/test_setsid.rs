//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use nix::unistd::{getpgid, getsid, Pid, Uid};
use std::process;
use std::time::Duration;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

const C_SETSID_PATH: &str = "/usr/bin/setsid";

fn expected_result_brief(
    bin_path: &str,
    test_scenario: &TestScenario,
    test_args: &[&str],
) -> std::result::Result<CmdResult, String> {
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

// In cargo test, `setsid`'s pgrp is not equal to its pid, so fork is disabled by default.
#[test]
fn test_setsid_sid_no_fork() {
    let test_scenario = TestScenario::new(util_name!());
    let child = test_scenario.ucmd().args(&["sleep", "10"]).run_no_wait();

    // Sleep for the child calling setsid().
    std::thread::sleep(Duration::from_millis(500));

    let parent_pid = process::id();
    let child_pid = child.id();

    let parent_pgid = getpgid(Pid::from_raw(parent_pid as i32).into()).unwrap();
    let child_pgid = getpgid(Pid::from_raw(child_pid as i32).into()).unwrap();
    println!("parent pid: {}", parent_pid);
    println!("child pid: {}", child_pid);
    println!("parent pgid: {}", parent_pgid);
    println!("child pgid: {}", child_pgid);

    // Checking sid from getsid().
    let parent_sid_getsid = getsid(Pid::from_raw(parent_pid as i32).into()).unwrap();
    let child_sid_getsid = getsid(Pid::from_raw(child_pid as i32).into()).unwrap();

    println!("Checking sid from getsid()");
    println!("parent sid: {}", parent_sid_getsid);
    println!("child sid: {}", child_sid_getsid);

    assert_ne!(parent_sid_getsid.as_raw(), 0);
    assert_ne!(child_sid_getsid.as_raw(), 0);
    assert_ne!(parent_sid_getsid, child_sid_getsid);
    assert_eq!(child_pid as i32, child_sid_getsid.as_raw());

    // Checking sid from sys.processes().
    let mut parent_sid = None;
    let mut child_sid = None;
    let mut sys = System::new_all();
    sys.refresh_all();
    for (pid, process) in sys.processes() {
        if pid.as_u32() == parent_pid || pid.as_u32() == child_pid {
            println!(
                "[PID]{pid} [SID]{} {:?}",
                process.session_id().unwrap().as_u32(),
                process.name()
            );
            match pid.as_u32() {
                _pid if _pid == parent_pid => {
                    parent_sid = process.session_id();
                }
                _pid if _pid == child_pid => {
                    child_sid = process.session_id();
                }
                _ => {}
            }
        }
    }

    println!("Checking sid from sys.processes()");
    println!("parent sid: {}", parent_sid.unwrap().as_u32());
    println!("child sid: {}", child_sid.unwrap().as_u32());

    assert_ne!(parent_sid, None);
    assert_ne!(child_sid, None);
    assert_ne!(child_sid, parent_sid);
    assert_eq!(child_pid, child_sid.unwrap().as_u32());
}

#[test]
fn test_setsid_sid_fork() {
    let test_scenario = TestScenario::new(util_name!());
    let child = test_scenario
        .ucmd()
        // The father of `sleep` is set to `/init` after `setsid` process exits.
        // So we use magic argument number to find `sleep` process by sys.process().
        .args(&["--fork", "sleep", "10.03270904"])
        .run_no_wait();

    // Sleep for the child calling setsid()
    std::thread::sleep(Duration::from_millis(500));

    let cargo_pid = process::id();
    let setsid_pid = child.id();
    let cargo_pgid = getpgid(Pid::from_raw(cargo_pid as i32).into()).unwrap();
    let setsid_pgid = getpgid(Pid::from_raw(setsid_pid as i32).into()).unwrap();
    let cargo_sid = getsid(Pid::from_raw(cargo_pid as i32).into()).unwrap();
    let setsid_sid = getsid(Pid::from_raw(setsid_pid as i32).into()).unwrap();
    println!("cargo pid: {}", cargo_pid);
    println!("setsid pid: {}", setsid_pid);
    println!("cargo pgid: {}", cargo_pgid);
    println!("setsid pgid: {}", setsid_pgid);
    println!("cargo sid: {}", cargo_sid);
    println!("setsid sid: {}", setsid_sid);

    // Finding `sleep`'s pid and sid from sys.processes().
    let mut sleep_pid = sysinfo::Pid::from_u32(0);
    let mut sleep_sid = None;
    let mut sys = System::new_all();
    sys.refresh_all();
    for (pid, process) in sys.processes() {
        if process.exe().ends_with("sleep")
            && process.cmd().len() >= 2
            && process.cmd()[1] == "10.03270904"
        {
            println!(
                "[PID]{pid} [SID]{} {:?}",
                process.session_id().unwrap().as_u32(),
                process.name()
            );
            sleep_pid = process.pid();
            sleep_sid = process.session_id();
        }
    }

    assert_ne!(sleep_pid.as_u32(), 0);
    println!("sleep sid: {}", sleep_pid.as_u32());
    assert_ne!(sleep_sid, None);
    println!("sleep sid: {}", sleep_sid.unwrap().as_u32());
    assert_eq!(sleep_pid.as_u32(), sleep_sid.unwrap().as_u32());
    assert_ne!(sleep_sid.unwrap().as_u32() as i32, setsid_sid.as_raw());
}

#[test]
fn test_setsid_exit_code() {
    let test_scenario = TestScenario::new(util_name!());

    let mut execvp_argvs = vec![
        // Test for no argument.
        vec![],
        // Test for execvp success and program exit success.
        vec!["echo", "success"],
        // Test for execvp success and program exit failure.
        vec!["sleep", "wrong-argument"],
        // Test for execvp failure for EX_EXEC_ENOENT.
        vec!["not-exist-exec-file-03270904"],
    ];

    if !Uid::effective().is_root() {
        // Test for execvp failure for EX_EXEC_FAILED.
        execvp_argvs.push(vec!["/dev/null"])
    } else {
        println!("Test skipped: a non-root uid needed.");
    }

    for fork_option in ["", "-f", "--fork"] {
        for wait_option in ["", "-w", "--wait"] {
            for ctty_option in ["", "-c", "--ctty"] {
                for execvp_argv in &execvp_argvs {
                    let mut test_args = vec![];
                    if !fork_option.is_empty() {
                        test_args.push(fork_option);
                    }
                    if !wait_option.is_empty() {
                        test_args.push(wait_option);
                    }
                    if !ctty_option.is_empty() {
                        test_args.push(ctty_option);
                    }
                    test_args.extend(execvp_argv);

                    let expcted_result = unwrap_or_return!(expected_result_brief(
                        C_SETSID_PATH,
                        &test_scenario,
                        &test_args
                    ));

                    test_scenario
                        .ucmd()
                        .args(&test_args)
                        .run()
                        .stdout_is(expcted_result.stdout_str())
                        .stderr_is(expcted_result.stderr_str())
                        .code_is(expcted_result.code());
                }
            }
        }
    }
}

#[test]
fn test_setsid_ctty() {
    let test_scenario = TestScenario::new(util_name!());
    let test_args = ["--ctty", "sleep", "10.03270904"];

    // Cargo test does not run in a tty.
    test_scenario
        .cmd("bash")
        .args(&[
            "-c",
            "if [ -t 0 ]; then echo \"stdin is a tty\"; else echo \"stdin is not a tty\"; fi;",
        ])
        .run()
        .stdout_is("stdin is not a tty\n");

    // Setting the controlling terminal fails because cargo test does not run in a tty.
    // The test for --ctty can be done manually.
    if let Ok(result) = run_ucmd_as_root(&test_scenario, &test_args) {
        let expected_result = test_scenario
            .cmd_keepenv("sudo")
            .env("LC_ALL", "C")
            .arg("-E")
            .arg("--non-interactive")
            .arg(&test_scenario.util_name)
            .args(&test_args)
            .run();
        result
            .stdout_is(expected_result.stdout_str())
            .stderr_is(expected_result.stderr_str())
            .code_is(expected_result.code());
    } else {
        println!("TEST SKIPPED");
    }
}
