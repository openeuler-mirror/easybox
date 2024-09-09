//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use nix::unistd::getpid;

use crate::{
    common::util::*,
    test_attr::run_cmd_as_root_ignore_ci,
    test_hwclock::run_ucmd_as_root_ignore_ci,
    test_mount::{
        compare_mount_result, setup_loop_device, C_LOSETUP_PATH, C_MOUNT_PATH, C_UMOUNT_PATH,
        TEST_MOUNT_POINT,
    },
};
use std::sync::Mutex;
static KEEP_SINGLE_THREAD: Mutex<bool> = Mutex::new(false);

fn run_and_compare(ts: &TestScenario, args: &[&str]) {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let loopdevice = &setup_loop_device(ts);

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, &args).unwrap();
    let rust_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &args).unwrap();
    let c_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();

    compare_mount_result(c_res, rust_res, c_umount_res, rust_umount_res);
}

#[test]
fn test_umount_all() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = TestScenario::new(util_name!());
    let arg = "-a";

    let c_res = ts.cmd(C_UMOUNT_PATH).arg(arg).run();
    let rust_res = ts.ucmd().arg(arg).run();
    c_res.stdout_is(rust_res.stdout_str());
    c_res.stderr_is(rust_res.stderr_str());
}
#[test]
fn test_umount_test_opts() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = TestScenario::new(util_name!());
    let args = &["-a", "-O", "ext4"];

    let c_res = ts.cmd(C_UMOUNT_PATH).args(args).run();
    let rust_res = ts.ucmd().args(args).run();
    c_res.stdout_is(rust_res.stdout_str());
    c_res.stderr_is(rust_res.stderr_str());
}
#[test]
fn test_umount_all_targets() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    let loopdevice = &setup_loop_device(ts);
    let args = &["-A", loopdevice];
    const TEST_MOUNT_POINT_A: &str = "mount_point_a";
    const TEST_MOUNT_POINT_B: &str = "mount_point_b";
    ts.cmd("/usr/bin/mkdir")
        .arg(TEST_MOUNT_POINT_A)
        .arg(TEST_MOUNT_POINT_B)
        .run();

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT_A]).unwrap();
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT_B]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT_A]).unwrap();
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT_B]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, args).unwrap();
    let c_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_umount_res, rust_umount_res);
}
#[test]
fn test_umount_no_canonicalize() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-c", TEST_MOUNT_POINT]);
}
#[test]
fn test_umount_detach_loop() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    let loopdevice = &setup_loop_device(ts);
    let args = &["-d", TEST_MOUNT_POINT];

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &[loopdevice, "ext4.img"]).unwrap();
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, args).unwrap();
    let c_umount_res = ts.cmd(C_MOUNT_PATH).run();

    compare_mount_result(c_res, rust_res, c_umount_res, rust_umount_res);
}

#[test]
fn test_umount_force() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-f", TEST_MOUNT_POINT]);
}

#[test]
fn test_umount_fake() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    let loopdevice = &setup_loop_device(ts);
    let args = &["--fake", TEST_MOUNT_POINT];

    // Prepare a mountpoint to umount
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, args).unwrap();
    let c_umount_res = ts.cmd(C_MOUNT_PATH).run();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_umount_res, rust_umount_res);
}

#[test]
fn test_umount_internal_only() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-i", TEST_MOUNT_POINT]);
}

#[test]
fn test_umount_lazy() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-l", TEST_MOUNT_POINT]);
}

#[test]
fn test_umount_no_mtab() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-n", TEST_MOUNT_POINT]);
}

#[test]
fn test_umount_namespace() {
    let ts = TestScenario::new(util_name!());
    let res = ts.cmd("/usr/bin/realpath").arg(TEST_MOUNT_POINT).run();
    let realpath = res.stdout_str().trim();
    run_and_compare(&ts, &["-N", &getpid().as_raw().to_string(), realpath]);
}

#[test]
fn test_umount_quiet() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-q", TEST_MOUNT_POINT]);
}

#[test]
fn test_umount_read_only() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-r", TEST_MOUNT_POINT]);
}

#[test]
fn test_umount_recursive() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-R", TEST_MOUNT_POINT]);
}
#[test]
fn test_umount_types() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-t", "ext4", TEST_MOUNT_POINT]);
}
#[test]
fn test_umount_verbose() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-v", TEST_MOUNT_POINT]);
}
