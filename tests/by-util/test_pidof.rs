// This file is part of the easybox package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use std::process::Command;

use crate::common::util::*;

const C_PIDOF_PATH: &str = "/usr/bin/pidof";

fn run_and_compare(test_args: &[&str]) {
    let c_res = TestScenario::new(util_name!())
        .cmd(C_PIDOF_PATH)
        .args(test_args)
        .run();
    let _rust_res = TestScenario::new(util_name!())
        .ucmd()
        .args(test_args)
        .run()
        .stdout_only_bytes(c_res.stdout());
}

#[test]
fn test_pidof() {
    // pidof easybox
    TestScenario::new(util_name!())
        .ucmd()
        .args(&["easybox"])
        .succeeds();
}

#[test]
fn test_pidof_single_prog() {
    // pidof systemd
    run_and_compare(&["systemd"]);
    // pidof bash
    run_and_compare(&["bash"]);
    // pidof /usr/bin/bash
    run_and_compare(&["/usr/bin/bash"]);
    // pidof /usr/bin/bash
    run_and_compare(&["/bin/bash"]);
}

#[test]
fn test_pidof_multi_prog() {
    // pidof node systemd bash
    run_and_compare(&["systemd", "bash"]);
    // pidof sh zsh ssh sshd bash
    run_and_compare(&["zsh", "ssh", "sshd", "bash"])
}

#[test]
fn test_pidof_single() {
    // pidof -s bash systemd sh
    run_and_compare(&["bash", "systemd", "sh", "-s"]);
    // pidof --single-shot bash systemd sh
    run_and_compare(&["bash", "systemd", "sh", "--single-shot"]);
}

#[test]
fn test_pidof_quiet() {
    // pidof -q systemd
    run_and_compare(&["systemd", "-q"]);
}

#[test]
fn test_pidof_worker() {
    // pidof -w kthreadd
    run_and_compare(&["kthreadd", "-w"]);
    // pidof kthreadd kswapd0
    run_and_compare(&["kthreadd"]);
    // pidof --with-workers kthreadd
    run_and_compare(&["kthreadd", "--with-workers"]);
}

#[test]
fn test_pidof_omit() {
    // pidof systemd -o 1
    run_and_compare(&["systemd", "-o", "1"]);
    // pidof systemd --omit-pid 1
    run_and_compare(&["systemd", "--omit-pid", "1"]);
    // pidof -o 1 systemd
    run_and_compare(&["-o", "1", "systemd"]);
    // pidof systemd kthreadd -w -o 1,2
    run_and_compare(&["systemd", "kthreadd", "-w", "-o", "1,2"]);
    // pidof systemd node bash sh -o 1
    run_and_compare(&["systemd", "node", "bash", "sh", "-o", "1"]);
}

#[test]
fn test_pidof_sep() {
    // pidof systemd sleep bash sh easybox -S -
    run_and_compare(&["systemd", "sleep", "bash", "-S", "-"]);
    // pidof systemd sleep bash sh easybox --separator -
    run_and_compare(&["systemd", "sleep", "bash", "--separator", "-"]);
    // pidof systemd sleep bash sh easybox -S - -S *
    run_and_compare(&["systemd", "sleep", "bash", "-S", "-", "-S", "*"]);
    // pidof systemd sleep bash sh easybox -S - --separator *
    run_and_compare(&[
        "systemd",
        "sleep",
        "bash",
        "sh",
        "-S",
        "-",
        "--separator",
        "*",
    ]);
}

#[test]
fn test_pidof_scipt() {
    let _child = Command::new("tests/fixtures/pidof/bin/test_sleep.sh").spawn();
    // pidof test_sleep.sh -x
    run_and_compare(&["test_sleep.sh", "-x"]);
    // pidof test_sleep.sh
    run_and_compare(&["test_sleep.sh"]);
}

#[test]
fn test_pidof_mixed_options() {
    // pidof -w -s systemd
    run_and_compare(&["-w", "-s", "systemd"]);
    // pidof -w -q -o 1 -S "," bash
    run_and_compare(&["-w", "-q", "-o", "1", "-S", ",", "bash", "sh"]);
    // pidof -q -w -o 1 -x systemd node sh
    run_and_compare(&["-q", "-w", "-o", "1", "-x", "systemd", "node", "sh"]);
    // pidof kswapd0 -w -q
    run_and_compare(&["kswapd0", "-w", "-q"]);
}
