// This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::{
    common::util::{TestScenario, UCommand, UUTILS_WARNING},
    test_attr::run_cmd_as_root_ignore_ci,
};
use login::logindefs::{dump_list, load_defaults};
use std::io::Write;
use std::{result::Result, sync::Mutex};

const C_LOGIN_PATH: &str = "/usr/bin/login";
const EXPECT_PATH: &str = "/usr/bin/expect";
const USERADD_PATH: &str = "/usr/sbin/useradd";
// Username: login_test Password: 123
const CREATE_TEST_USER: [&str; 6] = ["login_test", "-MN", "-d", "/", "-p", "bIgJASE9rgy2g"];
const FILTER_WORD: [&str; 10] = [
    "spawn",
    "Last login:",
    "System information as of time:",
    "System load:",
    "Processes:",
    "Memory used:",
    "Swap used:",
    "Usage On:",
    "Users online:",
    "SUDO_COMMAND",
];
// From util-linux login test suite output
const LOGINDEF_PARSE_RESULT: &str = "$HELLO_WORLD: 'hello world!'
$STRING: 'this_is_string'
$NUMBER: '123456'
$BOOLEAN: 'yEs'
$CRAZY1: 'this is crazy format'
$CRAZY2: 'fooBar'
$CRAZY3: 'FoooBaaar'
$EMPTY: '(null)'
$END: 'the is end'
";
static TEST_USER_ADDED: Mutex<bool> = Mutex::new(false);

#[cfg(unix)]
pub fn build_cmd_as_root(ts: &TestScenario) -> Result<UCommand, String> {
    use std::process::Command;
    match Command::new("sudo")
        .env("LC_ALL", "C")
        .args(["-E", "--non-interactive", "whoami"])
        .output()
    {
        Ok(output) if String::from_utf8_lossy(&output.stdout).eq("root\n") => {
            // we can run sudo and we're root
            // run ucmd as root:
            let mut res = ts.cmd_keepenv("sudo");
            res.env("LC_ALL", "C").arg("-E").arg("--non-interactive");
            Ok(res)
        }
        Ok(output)
            if String::from_utf8_lossy(&output.stderr).eq("sudo: a password is required\n") =>
        {
            Err("Cannot run non-interactive sudo".to_string())
        }
        Ok(_output) => Err("\"sudo whoami\" didn't return \"root\"".to_string()),
        Err(e) => Err(format!("{}: {}", UUTILS_WARNING, e)),
    }
}

fn run_and_compare(ts: &TestScenario, args: &[&str], exp_script: &str) {
    let mut inited_lock = TEST_USER_ADDED.lock();
    if let Ok(inited) = inited_lock.as_deref_mut() {
        if *inited == false {
            run_cmd_as_root_ignore_ci(ts, USERADD_PATH, &CREATE_TEST_USER).unwrap();
            *inited = true;
        }
    }

    let mut c_command = build_cmd_as_root(ts).unwrap();
    c_command
        .arg(EXPECT_PATH)
        .arg(exp_script)
        .arg(C_LOGIN_PATH)
        .args(args);
    let mut rust_command = build_cmd_as_root(ts).unwrap();
    rust_command
        .arg(EXPECT_PATH)
        .arg(exp_script)
        .arg(&ts.bin_path)
        .arg(&ts.util_name)
        .args(args);

    let c_res = c_command.run();
    let rust_res = rust_command.run();

    c_res.code_is(rust_res.code());
    c_res.stderr_is(rust_res.stderr_str());
    assert_eq!(
        filter_output(c_res.stdout_str()),
        filter_output(rust_res.stdout_str())
    );
}

fn filter_output(origin_out: &str) -> Vec<u8> {
    let mut res = Vec::new();
    for line in origin_out.lines() {
        let mut should_contain = true;
        for ban_word in FILTER_WORD {
            if line.starts_with(ban_word) {
                should_contain = false;
                break;
            }
        }
        if should_contain {
            write!(res, "{}", line).unwrap();
        }
    }
    res
}

#[test]
fn test_normal_login_procedure() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test1.exp");
}

#[test]
fn test_login_failed_procedure() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test2.exp");
}

#[test]
fn test_login_specific_username_procedure() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test3.exp");
}

#[test]
fn test_login_keep_environment() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test4.exp");
}

#[test]
fn test_login_skip_auth() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test5.exp");
}

#[test]
fn test_login_supress_hostname() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test6.exp");
}

#[test]
fn test_login_utmp_hostname() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &[], "test7.exp");
}

#[test]
fn test_logindefs_parse() {
    let ts = TestScenario::new(util_name!());
    load_defaults(ts.fixtures.plus("logindefs.data"));
    let res = dump_list();
    // print!("{}", String::from_utf8(res.clone()).unwrap());
    assert_eq!(res.as_slice(), LOGINDEF_PARSE_RESULT.as_bytes());
}
