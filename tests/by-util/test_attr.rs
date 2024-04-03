// This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use std::ffi::OsStr;
use std::os::unix::fs::symlink;

use crate::test_hwclock;
use test_hwclock::run_ucmd_as_root_ignore_ci;

use crate::common::util::*;

const TEST_FILE: &str = "attr_test_file";
const TEST_LINK_FILE: &str = "attr_test_link";
const C_ATTR_PATH: &str = "/usr/bin/attr";
const UUTILS_WARNING: &str = "uutils-tests-warning";

#[cfg(unix)]
pub fn run_cmd_as_root_ignore_ci<S: AsRef<OsStr>>(
    ts: &TestScenario,
    bin: S,
    args: &[&str],
) -> std::result::Result<CmdResult, String> {
    use std::process::Command;

    // check if we can run 'sudo'
    log_info("run", "sudo -E --non-interactive whoami");
    match Command::new("sudo")
        .env("LC_ALL", "C")
        .args(["-E", "--non-interactive", "whoami"])
        .output()
    {
        Ok(output) if String::from_utf8_lossy(&output.stdout).eq("root\n") => {
            // we can run sudo and we're root
            // run ucmd as root:
            Ok(ts
                .cmd_keepenv("sudo")
                .env("LC_ALL", "C")
                .arg("-E")
                .arg("--non-interactive")
                .arg(bin)
                .args(args)
                .run())
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

fn create_link_for_file(ts: &TestScenario) {
    let run_path = &ts.fixtures;
    symlink(run_path.plus(TEST_FILE), run_path.plus(TEST_LINK_FILE)).unwrap();
}

#[test]
fn test_option_sg() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &["-s", "testname", TEST_FILE];
    let stdin_pipe = "testval";
    let c_res1 = c_ts
        .cmd(C_ATTR_PATH)
        .args(test_args1)
        .pipe_in(stdin_pipe)
        .succeeds();
    ts.ucmd()
        .args(test_args1)
        .pipe_in(stdin_pipe)
        .succeeds()
        .stdout_only_bytes(c_res1.stdout());

    let test_args2 = &["-g", "testname", TEST_FILE];
    let c_res2 = c_ts.cmd(C_ATTR_PATH).args(test_args2).succeeds();
    ts.ucmd()
        .args(test_args2)
        .succeeds()
        .stdout_only_bytes(c_res2.stdout());
}

#[test]
#[allow(non_snake_case)]
fn test_secure_quiet_option_sgVSq() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &["-S", "-s", "securename", "-V", "secureval", TEST_FILE];
    let c_res1 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args1).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args1)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res1.stdout());

    let test_args2 = &["-Sq", "-g", "securename", TEST_FILE];
    let c_res2 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args2).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args2)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res2.stdout());
}

#[test]
#[allow(non_snake_case)]
fn test_root_followlink_option_sglVRL() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &["-LR", "-s", "rootname", "-V", "rootval", TEST_LINK_FILE];
    create_link_for_file(&ts);
    create_link_for_file(&c_ts);

    let c_res1 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args1).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args1)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res1.stdout());

    let test_args2 = &["-R", "-g", "rootname", TEST_FILE];
    let c_res2 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args2).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args2)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res2.stdout());

    let test_args3 = &["-LR", "-l", TEST_LINK_FILE];
    let c_res3 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args3).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args3)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res3.stdout());
}

#[test]
#[allow(non_snake_case)]
fn test_quiet_option_slVq() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &["-s", "testattr", "-V", "testval", TEST_FILE];
    let c_res1 = c_ts.cmd(C_ATTR_PATH).args(test_args1).succeeds();
    ts.ucmd()
        .args(test_args1)
        .succeeds()
        .stdout_only_bytes(c_res1.stdout());

    let test_args2 = &["-q", "-l", TEST_FILE];
    let c_res2 = c_ts.cmd(C_ATTR_PATH).args(test_args2).succeeds();
    ts.ucmd()
        .args(test_args2)
        .succeeds()
        .stdout_only_bytes(c_res2.stdout());
}

#[test]
#[allow(non_snake_case)]
fn test_remove_option_srgV() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &["-s", "testattr", "-V", "testval", TEST_FILE];
    let c_res1 = c_ts.cmd(C_ATTR_PATH).args(test_args1).succeeds();
    ts.ucmd()
        .args(test_args1)
        .succeeds()
        .stdout_only_bytes(c_res1.stdout());

    let test_args2 = &["-r", "testattr", TEST_FILE];
    c_ts.cmd(C_ATTR_PATH)
        .args(test_args2)
        .succeeds()
        .no_stderr()
        .no_stdout();
    ts.ucmd()
        .args(test_args2)
        .succeeds()
        .no_stdout()
        .no_stderr();

    let test_args3 = &["-g", "testattr", TEST_FILE];
    let c_res3 = c_ts.cmd(C_ATTR_PATH).args(test_args3).fails();
    ts.ucmd()
        .args(test_args3)
        .fails()
        .stderr_only_bytes(c_res3.stderr());
}

#[test]
#[allow(non_snake_case)]
fn test_remove_secure_quiet_followlink_option_srSVLq() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &[
        "-LSq",
        "-s",
        "secureattr",
        "-V",
        "secureval",
        TEST_LINK_FILE,
    ];
    create_link_for_file(&ts);
    create_link_for_file(&c_ts);

    run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args1)
        .unwrap()
        .success()
        .no_stderr()
        .no_stdout();
    run_ucmd_as_root_ignore_ci(&ts, test_args1)
        .unwrap()
        .success()
        .no_stdout()
        .no_stderr();

    let test_args2 = &["-LS", "-r", "secureattr", TEST_LINK_FILE];
    run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args2)
        .unwrap()
        .success()
        .no_stderr()
        .no_stdout();
    run_ucmd_as_root_ignore_ci(&ts, test_args2)
        .unwrap()
        .success()
        .no_stdout()
        .no_stderr();
}

#[test]
#[allow(non_snake_case)]
fn test_secure_root_quiet_option_slVSRq() {
    let ts = TestScenario::new(util_name!());
    let c_ts = TestScenario::new(util_name!());
    let test_args1 = &["-s", "testattr", "-V", "testval", TEST_FILE];
    let c_res1 = c_ts.cmd(C_ATTR_PATH).args(test_args1).succeeds();
    ts.ucmd()
        .args(test_args1)
        .succeeds()
        .stdout_only_bytes(c_res1.stdout());

    let test_args2 = &["-S", "-s", "secureattr", "-V", "secureval", TEST_FILE];
    let c_res2 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args2).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args2)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res2.stdout());

    let test_args3 = &["-R", "-s", "rootattr", "-V", "rootval", TEST_FILE];
    let c_res3 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args3).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args3)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res3.stdout());

    let test_args4 = &["-ql", TEST_FILE];
    let c_res4 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args4).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args4)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res4.stdout());

    let test_args5 = &["-Sql", TEST_FILE];
    let c_res5 = run_cmd_as_root_ignore_ci(&c_ts, C_ATTR_PATH, test_args5).unwrap();
    run_ucmd_as_root_ignore_ci(&ts, test_args5)
        .unwrap()
        .success()
        .stdout_only_bytes(c_res5.stdout());
}

#[test]
fn test_wrong_option_ls() {
    let expect_contains = "Only one of -s, -g, -r, or -l allowed";
    let test_args = &["-l", "-s", "testattr", TEST_FILE];
    TestScenario::new(util_name!())
        .cmd(C_ATTR_PATH)
        .args(test_args)
        .fails()
        .stderr_contains(expect_contains);
    new_ucmd!()
        .args(test_args)
        .fails()
        .no_stdout()
        .stderr_contains(expect_contains);
}
