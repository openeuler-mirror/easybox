// This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;
use std::process::Command;

const UTIL: &str = "find";
const C_FIND_PATH: &str = "/usr/bin/find";

pub fn run_and_compare(ts: &TestScenario, args: &[&str]) {
    let actual_result = ts.ucmd().args(args).run();
    let expect_result = ts.cmd_keepenv(C_FIND_PATH).args(args).run();

    assert_eq!(expect_result.stdout(), actual_result.stdout());
}

pub fn run_and_compare_with_buf(ts: &TestScenario, args: &[&str], buf: &[u8]) {
    let actual_result = ts.ucmd().args(args).run();
    assert_eq!(actual_result.stdout(), buf)
}

pub fn run_and_fail(ts: &TestScenario, args: &[&str]) {
    ts.ucmd().args(args).run().failure();
}

fn run_cmd_as_root(
    ts: &TestScenario,
    command: &str,
    args: &[&str],
) -> std::result::Result<CmdResult, String> {
    if !is_ci() {
        log_info("run", "sudo -E --non-interactive whoami");
        match Command::new("sudo")
            .env("LC_ALL", "C")
            .args(["-E", "--non-interactive", "whoami"])
            .output()
        {
            Ok(output) if String::from_utf8_lossy(&output.stdout).eq("root\n") => Ok(ts
                .cmd_keepenv("sudo")
                .env("LC_ALL", "C")
                .arg("-E")
                .arg("--non-interactive")
                .arg(command)
                .args(args)
                .run()),
            Ok(output)
                if String::from_utf8_lossy(&output.stderr).eq("sudo: a password is required\n") =>
            {
                Err("Cannot run non-interactive sudo".to_string())
            }
            Ok(_output) => Err("\"sudo whoami\" didn't return \"root\"".to_string()),
            Err(e) => Err(format!("{}: {}", "Find test warning", e)),
        }
    } else {
        Err(format!("{}: {}", "Find test info", "cannot run inside CI"))
    }
}

#[test]
fn test_newer() {
    let ts = TestScenario::new(UTIL);

    // Prepare the files with different timestamp.
    ts.cmd_keepenv("touch").args(&["file1"]).run();
    ts.cmd_keepenv("sleep").args(&["1"]).run();
    ts.cmd_keepenv("touch").args(&["file2"]).run();
    ts.cmd_keepenv("sleep").args(&["1"]).run();
    ts.cmd_keepenv("touch").args(&["file3"]).run();

    let newer_filters = &[
        "newer", "anewer", "cnewer", "neweraa", "newerac", "neweram", "newerca", "newercc",
        "newercm", "newerma", "newermc", "newermm",
    ];

    for filter in newer_filters {
        run_and_compare(&ts, &[filter, "file2", "-name", "file*"]);
    }

    let res = ts.cmd_keepenv("stat").args(&["-c", "%y", "file2"]).run();
    let tref = res.stdout();
    let tref = String::from_utf8(tref.to_vec());
    if let Ok(tref) = tref {
        let filters = &["newerat", "newerct", "newermt"];

        for filter in filters {
            run_and_compare(&ts, &[filter, &tref])
        }
    }
}

#[test]
fn test_arg_nan() {
    let ts = TestScenario::new(UTIL);
    let filters = &[
        "-used", "-amin", "-cmin", "-mmin", "-atime", "-ctime", "-mtime",
    ];
    let arg = "NaN";

    for filter in filters {
        run_and_fail(&ts, &[filter, arg]);
    }
}

#[test]
fn test_refuse_noop() {
    let ts = TestScenario::new(UTIL);
    let filters = &["-noop", "--noop"];

    for filter in filters {
        run_and_fail(&ts, &[filter]);
    }
}

#[test]
fn test_precedence() {
    let ts = TestScenario::new(UTIL);

    // Only print once because maxdepth is 0.
    run_and_compare(
        &ts,
        &[
            "-maxdepth",
            "0",
            "-printf",
            "Hello",
            "-printf",
            "Open",
            "-printf",
            "Euler",
            "-printf",
            "!\n",
        ],
    );
    run_and_compare(
        &ts,
        &[
            "-maxdepth",
            "0",
            "-printf",
            "Hello",
            "-o",
            "what\n",
            "-a",
            "World!\n",
        ],
    );
}

#[test]
fn test_parentheses() {
    let ts = TestScenario::new(UTIL);

    run_and_compare(
        &ts,
        &[
            "-maxdepth",
            "0",
            "-printf",
            "Hello",
            "(",
            "-printf",
            "World!\n",
            "-printf",
            "what?\n",
            ")",
        ],
    )
}

#[test]
fn test_escape_octal() {
    let ts = TestScenario::new(UTIL);

    for i in 0o0..0o200 {
        run_and_compare(&ts, &["-printf", &format!("\\{:03o}", i)]);
    }
}

#[test]
fn test_escape_special() {
    let ts = TestScenario::new(UTIL);

    run_and_compare(
        &ts,
        &[
            "-printf", "\\a\\n", "-printf", "\\r\\n", "-printf", "\\f\\n", "-printf", "\\t\\n",
            "-printf", "\\v\\n", "-printf", "\\b\\n", "-printf", "\\\n", "-printf", "\\z\\n",
        ],
    );
}

#[test]
fn test_maxdepth_0() {
    let ts = TestScenario::new(UTIL);

    run_and_compare_with_buf(&ts, &[".", "-maxdepth", "0"], b".\n");
}

#[test]
fn test_true_and_false() {
    let ts = TestScenario::new(UTIL);

    run_and_compare(&ts, &[".", "-true"]);
    run_and_compare(&ts, &[".", "-false"]);

    run_and_compare(&ts, &[".", "-true", "-o", "-false"]);
    run_and_compare(&ts, &[".", "-false", "-a", "-true"]);
}

#[test]
fn test_empty() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").arg("file1").run();
    ts.cmd_keepenv("dd")
        .args(&["if=/dev/zero", "of=file2", "bs=1", "count=1"])
        .run();
    run_and_compare(&ts, &["-empty"]);
}

#[test]
fn test_name() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("mkdir").arg("dir").run();
    ts.cmd_keepenv("touch").arg("dir/file").run();

    run_and_compare(&ts, &["-name", "file"]);
    run_and_compare(&ts, &["-name", "fi*"]);
    run_and_compare(&ts, &["-name", "*le"]);
}

#[test]
fn test_name_slash() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("mkdir").arg("dir").run();
    ts.cmd_keepenv("touch").arg("dir/file").run();

    run_and_compare(&ts, &["-name", "dir/file"]);
}

#[test]
fn test_name_lbracket_literal() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("mkdir").arg("dir").run();
    ts.cmd_keepenv("touch").arg("dir/[").run();

    run_and_compare(&ts, &["-name", "[[]"]);
}

#[test]
fn test_path() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("mkdir").arg("dir").run();
    ts.cmd_keepenv("touch").arg("dir/file").run();

    run_and_compare(&ts, &["-path", "file"]);
    run_and_compare(&ts, &["-path", "fi*"]);
    run_and_compare(&ts, &["-path", "*le"]);
    run_and_compare(&ts, &["-path", "dir/file"]);
}

#[test]
fn test_samefile() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").arg("file").run();
    ts.cmd_keepenv("ln").args(&["file", "link"]).run();
    ts.cmd_keepenv("touch").arg("another_file").run();

    run_and_compare(&ts, &["-samefile", "file"]);
}

#[test]
fn test_perm_exact() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").arg("file").run();
    ts.cmd_keepenv("chmod").args(&["764", "file"]).run();

    run_and_compare(&ts, &["-perm", "764"]);
    run_and_compare(&ts, &["-perm", "664"]);
    run_and_compare(&ts, &["-perm", "777"]);
}

#[test]
fn test_perm_all() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").arg("file").run();
    ts.cmd_keepenv("chmod").args(&["764", "file"]).run();

    run_and_compare(&ts, &["-perm", "-764"]);
    run_and_compare(&ts, &["-perm", "-664"]);
    run_and_compare(&ts, &["-perm", "-777"]);
    run_and_compare(&ts, &["-perm", "-000"]);
}

#[test]
fn test_perm_any() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").arg("file").run();
    ts.cmd_keepenv("chmod").args(&["764", "file"]).run();

    run_and_compare(&ts, &["-perm", "/764"]);
    run_and_compare(&ts, &["-perm", "/664"]);
    run_and_compare(&ts, &["-perm", "/777"]);
    run_and_compare(&ts, &["-perm", "/000"]);
}

#[test]
fn test_perm_symbol() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").arg("dir/file").run();
    ts.cmd_keepenv("chmod").args(&["764", "dir/file"]).run();

    run_and_compare(&ts, &["dir", "-perm", "-u=rwx,g=rw,a=r"]);
    run_and_compare(&ts, &["dir", "-perm", "-u=rwx"]);
}

#[test]
fn test_size() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("dd")
        .args(&["if=/dev/zero", "of=file_1k", "bs=1k", "count=1"])
        .run();
    ts.cmd_keepenv("dd")
        .args(&["if=/dev/zero", "of=file_2k", "bs=2k", "count=1"])
        .run();
    ts.cmd_keepenv("touch").args(&["empty"]);

    run_and_compare(&ts, &["-size", "1k"]);
    run_and_compare(&ts, &["-size", "-1k"]);
    run_and_compare(&ts, &["-size", "+1k"]);
}

#[test]
fn test_size_round_up() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("dd")
        .args(&["if=/dev/zero", "of=file_512B", "bs=512", "count=1"])
        .run();

    run_and_compare(&ts, &["-size", "1k"]);
    run_and_compare(&ts, &["-size", "-1k"]);
    run_and_compare(&ts, &["-size", "+1k"]);
}

#[test]
fn test_type_link() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").args(&["file"]).run();
    ts.cmd_keepenv("ln")
        .args(&["-s", "file", "file_link"])
        .run();
    ts.cmd_keepenv("mkdir").args(&["dir"]).run();
    ts.cmd_keepenv("ln").args(&["-s", "dir", "dir_link"]).run();

    run_and_compare(&ts, &["-type", "f"]);
    run_and_compare(&ts, &["-type", "d"]);
    run_and_compare(&ts, &["-type", "l"]);

    run_and_compare(&ts, &["-L", "-type", "f"]);
    run_and_compare(&ts, &["-L", "-type", "d"]);
    run_and_compare(&ts, &["-L", "-type", "l"]);
}

#[test]
fn test_xtype_link() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch").args(&["file"]).run();
    ts.cmd_keepenv("ln")
        .args(&["-s", "file", "file_link"])
        .run();
    ts.cmd_keepenv("mkdir").args(&["dir"]).run();
    ts.cmd_keepenv("ln").args(&["-s", "dir", "dir_link"]).run();

    run_and_compare(&ts, &["-xtype", "f"]);
    run_and_compare(&ts, &["-xtype", "d"]);
    run_and_compare(&ts, &["-xtype", "l"]);

    run_and_compare(&ts, &["-L", "-xtype", "f"]);
    run_and_compare(&ts, &["-L", "-xtype", "d"]);
    run_and_compare(&ts, &["-L", "-xtype", "l"]);
}

#[test]
fn test_xdev() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("mkdir").args(&["-p", "tmp"]).run();
    if let Err(_) = run_cmd_as_root(
        &ts,
        "mount",
        &["-t", "tmpfs", "-o", "size=10M", "tmpfs", "tmp"],
    ) {
        return;
    }

    run_and_compare(&ts, &["-xdev", "-name", "tmp"]);

    let _ = run_cmd_as_root(&ts, "umount", &["tmp"]);
}

#[test]
fn test_daystart() {
    let ts = TestScenario::new(UTIL);

    ts.cmd_keepenv("touch")
        .args(&["-d", "yesterday 00:01", "file"])
        .run();

    let buf = "./file\n".as_bytes();
    run_and_compare_with_buf(&ts, &["-daystart", "-atime", "0", "-name", "file"], buf);

    run_and_compare(&ts, &["-atime", "0", "-name", "file"]);
}

fn create_prune_delete_env(ts: &TestScenario) {
    ts.cmd_keepenv("mkdir").args(&["dir"]).run();
    ts.cmd_keepenv("touch").args(&["file"]).run();
    ts.cmd_keepenv("touch").args(&["dir/file"]).run();
}

#[test]
fn test_delete() {
    let ts_oe = TestScenario::new(UTIL);
    let ts_findutil = TestScenario::new(UTIL);

    create_prune_delete_env(&ts_oe);
    create_prune_delete_env(&ts_findutil);

    ts_oe.ucmd().args(&["-name", r#""file""#, "-delete"]).run();
    ts_findutil
        .cmd_keepenv(C_FIND_PATH)
        .args(&["-name", r#""file""#, "-delete"])
        .run();

    let findutil_res = ts_findutil.cmd_keepenv(C_FIND_PATH).run().stdout().to_vec();
    run_and_compare_with_buf(&ts_oe, &[], &findutil_res);
}

#[test]
fn test_prune() {
    let ts = TestScenario::new(UTIL);

    create_prune_delete_env(&ts);

    run_and_compare(&ts, &["-name", r#""file""#]);
    run_and_compare(
        &ts,
        &[
            "(",
            "-name",
            r#""dir""#,
            "-prune",
            ")",
            "-o",
            "-name",
            r#""file""#,
        ],
    );
}

#[test]
fn test_quit() {
    let ts = TestScenario::new(UTIL);

    create_prune_delete_env(&ts);

    run_and_compare(&ts, &["-name", r#""file""#]);
    run_and_compare(
        &ts,
        &[
            "(",
            "-name",
            r#""dir""#,
            "-quit",
            ")",
            "-o",
            "-name",
            r#""file""#,
        ],
    );
}

#[test]
fn test_large_directory() {
    let ts = TestScenario::new(UTIL);

    ts.ucmd().args(&["/home"]).run();
}

// === Gnu Tests ===

pub fn run_script(script_path: &str) {
    let ts = TestScenario::new(UTIL);
    let util = &format!("{} {}", ts.bin_path.display(), UTIL);
    ts.cmd_keepenv("bash")
        .args(&[script_path])
        .env("FIND", util)
        .run()
        .success();
}

#[test]
fn test_gnu_arg_nan() {
    run_script("gnu/arg-nan.sh");
}

#[test]
fn test_gnu_depth_unreadable_dir() {
    run_script("gnu/depth_unreadable_dir.sh");
}

#[test]
fn test_gnu_exec_plus_last_file() {
    run_script("gnu/exec-plus-last-file.sh");
}

#[test]
fn test_gnu_files0_from() {
    run_script("gnu/files0-from.sh");
}

#[test]
fn test_gnu_inode_zero() {
    run_script("gnu/inode-zero.sh");
}

#[test]
fn test_gnu_name_lbracket_literal() {
    run_script("gnu/name-lbracket-literal.sh");
}

#[test]
fn test_gnu_name_slash() {
    run_script("gnu/name-slash.sh");
}

#[test]
fn test_gnu_newer() {
    run_script("gnu/newer.sh");
}

#[test]
fn test_gnu_printf_escape_chars() {
    run_script("gnu/printf_escape_chars.sh");
}

#[test]
fn test_gnu_printf_escape_c() {
    run_script("gnu/printf_escape_c.sh");
}

#[test]
fn test_gnu_printf_inode() {
    run_script("gnu/printf_inode.sh");
}

#[test]
fn test_gnu_refuse_noop() {
    run_script("gnu/refuse-noop.sh");
}

#[test]
fn test_gnu_type_list() {
    run_script("gnu/type_list.sh");
}

#[test]
fn test_gnu_used() {
    run_script("gnu/used.sh");
}
