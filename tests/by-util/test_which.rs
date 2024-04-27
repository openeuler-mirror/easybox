// This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

const DEFAULT_PATH_ENV: &str = ".:~/bin:/bin:/usr/bin";
const C_WHICH_PATH: &str = "/usr/bin/which";

use std::env::{set_var, var};

use crate::{
    common::util::{TestScenario, UCommand},
    test_attr,
};
use test_attr::run_cmd_as_root_ignore_ci;

pub fn run_and_compare(
    ts: &TestScenario,
    args: &[&str],
    path_env: Option<&str>,
    stdin_file: Option<&str>,
    cur_dir: Option<&str>,
) {
    set_var("PATH", path_env.unwrap_or(DEFAULT_PATH_ENV));
    let mut c_command;
    let mut rust_command;
    if let Some(pwd) = cur_dir {
        c_command = UCommand::new(C_WHICH_PATH, &None::<&String>, pwd, false);
        rust_command = UCommand::new(&ts.bin_path, &Some(&ts.util_name), pwd, false);
    } else {
        c_command = ts.cmd_keepenv(C_WHICH_PATH);
        rust_command = ts.ucmd_keepenv();
    }
    c_command.args(args);
    rust_command.args(args);
    if let Some(v) = stdin_file {
        c_command.pipe_in_fixture(v);
        rust_command.pipe_in_fixture(v);
    }
    let c_res = c_command.run();
    let rust_res = rust_command.run();
    // For debug, see the output
    print!("{}", rust_res.stdout_str());
    print!("{}", rust_res.stderr_str());
    let c_res_stderr = c_res.stderr_str().replace(C_WHICH_PATH, util_name!());
    rust_res
        .code_is(c_res.code())
        .stdout_is_bytes(c_res.stdout())
        .stderr_is_bytes(c_res_stderr.as_bytes());
}

#[test]
fn test_split_command_options() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["--", "--version"], None, None, None);
}

#[test]
fn test_show_tilde_show_dot() {
    let ts = TestScenario::new(util_name!());
    ts.cmd("/usr/bin/touch").arg("cat");
    ts.cmd("/usr/bin/chmod").args(&["755", "cat"]);
    run_and_compare(&ts, &["cat"], None, None, None);
    run_and_compare(&ts, &["--show-tilde", "cat"], None, None, None);
    run_and_compare(&ts, &["--show-dot", "cat"], None, None, None);
    run_and_compare(
        &ts,
        &["--show-tilde", "--show-dot", "cat"],
        None,
        None,
        None,
    );
    run_and_compare(&ts, &["--skip-dot", "cat"], None, None, None);
}

#[test]
fn test_show_dot_skip_dot() {
    let ts = TestScenario::new(util_name!());
    let cur_dir = Some("/bin");
    run_and_compare(&ts, &["cat"], None, None, cur_dir);
    run_and_compare(&ts, &["--show-dot", "cat"], None, None, cur_dir);
    run_and_compare(
        &ts,
        &["--show-dot", "cat"],
        Some(".:/bin:/usr/bin"),
        None,
        cur_dir,
    );
    run_and_compare(
        &ts,
        &["--show-dot", "cat"],
        Some("/bin:.:/usr/bin"),
        None,
        cur_dir,
    );
    run_and_compare(
        &ts,
        &["--skip-dot", "--show-dot", "cat"],
        Some(".:/bin:/usr/bin"),
        None,
        cur_dir,
    );
}

#[test]
fn test_error_handle() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["ls"], None, None, None);
    run_and_compare(&ts, &["xxx"], None, None, None);
    run_and_compare(&ts, &["./ls"], None, None, None);
}

#[test]
fn test_absolute_path_show_dot() {
    let ts = TestScenario::new(util_name!());
    let cur_dir = Some("/");
    run_and_compare(&ts, &["bin/ls"], None, None, cur_dir);
    run_and_compare(&ts, &["--show-dot", "bin/ls"], None, None, cur_dir);
    run_and_compare(&ts, &["--show-dot", "/bin/ls"], None, None, cur_dir);
    run_and_compare(&ts, &["--show-dot", "bin/xxx"], None, None, cur_dir);
    run_and_compare(&ts, &["--show-dot", "/bin/xxx"], None, None, cur_dir);
}

#[test]
fn test_all_multi_commands() {
    let ts = TestScenario::new(util_name!());
    ts.cmd("/usr/bin/touch").arg("cat");
    ts.cmd("/usr/bin/chmod").args(&["755", "cat"]);
    run_and_compare(&ts, &["--all", "cat", "ls", "xxx", "yyy"], None, None, None);
}

#[test]
fn test_permission_filter() {
    let ts = TestScenario::new(util_name!());
    ts.cmd("/usr/bin/touch").args(&["cat", "xxx"]).run();
    ts.cmd("/usr/bin/chmod").args(&["755", "cat"]).run();
    ts.cmd("/usr/bin/chmod").args(&["644", "xxx"]).run();

    run_and_compare(&ts, &["./xxx"], None, None, None);

    ts.cmd("/usr/bin/chmod").args(&["711", "xxx"]).run();
    run_and_compare(&ts, &["./xxx"], None, None, None);

    ts.cmd("/usr/bin/chmod").args(&["655", "cat"]).run();
    run_and_compare(&ts, &["cat"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chown", &["root", "cat"]).unwrap();
    run_and_compare(&ts, &["cat"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chmod", &["545", "cat"]).unwrap();
    run_and_compare(&ts, &["cat"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chgrp", &["bin", "cat"]).unwrap();
    run_and_compare(&ts, &["cat"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chmod", &["541", "cat"]).unwrap();
    run_and_compare(&ts, &["cat"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chown", &["root", "xxx"]).unwrap();
    run_and_compare(&ts, &["./xxx"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chmod", &["700", "xxx"]).unwrap();
    run_and_compare(&ts, &["./xxx"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chmod", &["750", "xxx"]).unwrap();
    run_cmd_as_root_ignore_ci(
        &ts,
        "/usr/bin/chgrp",
        &[&var("GROUPS").unwrap_or(String::from("0")), "xxx"],
    )
    .unwrap();
    run_and_compare(&ts, &["./xxx"], None, None, None);

    run_cmd_as_root_ignore_ci(&ts, "/usr/bin/chgrp", &["bin", "xxx"]).unwrap();
    run_and_compare(&ts, &["./xxx"], None, None, None);
}

#[test]
fn test_read_alias_skip_alias() {
    let ts = TestScenario::new(util_name!());
    let stdin_file = Some("test_read_alias.in");
    run_and_compare(
        &ts,
        &[
            "--tty-only",
            "--read-alias",
            "--show-tilde",
            "--show-dot",
            "which",
        ],
        None,
        stdin_file,
        None,
    );
    run_and_compare(
        &ts,
        &[
            "--tty-only",
            "--read-alias",
            "--show-tilde",
            "--show-dot",
            "test1",
        ],
        None,
        stdin_file,
        None,
    );
    run_and_compare(
        &ts,
        &[
            "--tty-only",
            "--read-alias",
            "--show-tilde",
            "--show-dot",
            "test2",
        ],
        None,
        stdin_file,
        None,
    );
    run_and_compare(
        &ts,
        &[
            "--tty-only",
            "--read-alias",
            "--show-tilde",
            "--show-dot",
            "test3",
        ],
        None,
        stdin_file,
        None,
    );
    run_and_compare(
        &ts,
        &[
            "--tty-only",
            "--read-alias",
            "--show-tilde",
            "--show-dot",
            "test4",
        ],
        None,
        stdin_file,
        None,
    );
    run_and_compare(
        &ts,
        &[
            "--tty-only",
            "--read-alias",
            "--show-tilde",
            "--show-dot",
            "--skip-alias",
            "test4",
        ],
        None,
        stdin_file,
        None,
    );
}

#[test]
fn test_skip_tilde() {
    let ts = TestScenario::new(util_name!());
    set_var("HOME", ts.fixtures.as_string());
    run_and_compare(&ts, &["aabb"], None, None, None);
    run_and_compare(&ts, &["--skip-tilde", "aabb"], None, None, None);
}

#[test]
fn test_read_functions_skip_functions() {
    let ts = TestScenario::new(util_name!());
    let stdin_file = Some("test_read_functions.in");
    run_and_compare(&ts, &["--read-functions", "test1"], None, stdin_file, None);
    run_and_compare(&ts, &["--read-functions", "test2"], None, stdin_file, None);
    run_and_compare(&ts, &["--read-functions", "test3"], None, stdin_file, None);
    run_and_compare(
        &ts,
        &["--read-functions", "--skip-functions", "test1"],
        None,
        stdin_file,
        None,
    );
}
