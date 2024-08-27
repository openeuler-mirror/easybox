// This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;
use serial_test::serial;
use std::env;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
const C_KILLALL_PATH: &str = "/usr/bin/killall";

#[test]
#[serial]
fn test_killall_single() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["sleep"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_verbose() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["-v", "sleep"];
    let task = TestScenario::new(util_name!());
    let binding = task.ucmd().args(test_args).succeeds();
    let stderr_str = binding.stderr_str();
    assert!(!stderr_str.is_empty());
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_exact() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["-e", "sleep 100"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).run();
    thread::sleep(Duration::from_millis(100));
    assert!(!check_child_exit(&mut child));
    let test_args2 = &["-e", "sleep 10000"];
    task.ucmd().args(test_args2).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_ignore() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["SLEep"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).run();
    thread::sleep(Duration::from_millis(100));
    assert!(!check_child_exit(&mut child));
    let test_args2 = &["-I", "SLEep"];
    task.ucmd().args(test_args2).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_younger() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    thread::sleep(Duration::from_secs(2));
    let test_args = &["-y", "1s", "sleep"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).run();
    thread::sleep(Duration::from_millis(100));
    assert!(!check_child_exit(&mut child));
    let test_args2 = &["-y", "1y", "sleep"];
    task.ucmd().args(test_args2).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_older() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    thread::sleep(Duration::from_secs(2));
    let test_args = &["-o", "1s", "sleep"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_interactive() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["-i", "sleep"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).pipe_in("n\n").run();
    thread::sleep(Duration::from_secs(1));
    assert!(!check_child_exit(&mut child));
    let test_args2 = &["-i", "sleep"];
    task.ucmd().args(test_args2).pipe_in("y\n").succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
fn test_killall_list() {
    let test_args = &["-l"];
    let task = TestScenario::new(util_name!());
    let binding = task.ucmd().args(test_args).succeeds();
    let stdout_str = binding.stdout_str();
    let killall_path = Path::new(C_KILLALL_PATH);
    if killall_path.exists() {
        TestScenario::new(util_name!())
            .cmd(C_KILLALL_PATH)
            .arg("-l")
            .succeeds()
            .stdout_only(stdout_str);
    } else {
        assert_eq!(
            stdout_str,
            "HUP INT QUIT ILL TRAP ABRT BUS FPE KILL \
        USR1 SEGV USR2 PIPE ALRM TERM STKFLT\nCHLD CONT STOP TSTP TTIN \
        TTOU URG XCPU XFSZ VTALRM PROF WINCH POLL PWR SYS\n"
        );
    }
}

#[test]
fn test_killall_quiet() {
    let test_args = &["-q", "non-existent-process-abcd121dadasefg"];
    let task = TestScenario::new(util_name!());
    let binding = task.ucmd().args(test_args).run();
    let stderr_str = binding.code_is(1).stderr_str();
    assert!(stderr_str.is_empty());
}

#[test]
#[serial]
fn test_killall_regexp() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["-r", "sle*p"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_signal() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["-v", "-s", "SIGINT", "sleep"];
    let task = TestScenario::new(util_name!());
    let binding = task.ucmd().args(test_args).succeeds();
    let stderr_str = binding.stderr_str();
    assert!(stderr_str.contains("2"));
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_user() {
    let user = get_username().unwrap_or("".to_string());
    if user.is_empty() {
        return;
    }
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["-u", "non-existent-user-adsd121312", "sleep 100"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).run();
    thread::sleep(Duration::from_millis(100));
    assert!(!check_child_exit(&mut child));
    let test_args2 = &["-u", &user, "sleep"];
    task.ucmd().args(test_args2).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_wait() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child = Command::new("sleep")
        .arg("10000")
        .spawn()
        .expect("Failed to start process");
    let test_args = &["sleep", "-w"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).run();
    assert!(check_child_exit(&mut child));
}

#[test]
#[serial]
fn test_killall_namespace() {
    TestScenario::new(util_name!()).ucmd().arg("sleep").run();
    let mut child1 = Command::new("sleep")
        .arg("100")
        .spawn()
        .expect("Failed to start process");
    let mut child2 = Command::new("sleep")
        .arg("200")
        .spawn()
        .expect("Failed to start process");
    let pid1 = &child1.id().to_string();
    let test_args = &["-e", "-n", pid1, "sleep 200"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child2));
    let test_args2 = &["sleep"];
    task.ucmd().args(test_args2).succeeds();
    thread::sleep(Duration::from_millis(100));
    assert!(check_child_exit(&mut child1));
}

fn check_child_exit(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(Some(_status)) => return true,
        Ok(None) => return false,
        Err(_e) => return true,
    }
}

fn get_username() -> Option<String> {
    if let Ok(username) = env::var("USER") {
        return Some(username);
    }
    None
}
