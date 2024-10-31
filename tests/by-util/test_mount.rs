//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::sync::Mutex;

use nix::unistd::getpid;

use crate::{
    common::util::*, test_attr::run_cmd_as_root_ignore_ci, test_hwclock::run_ucmd_as_root_ignore_ci,
};
static KEEP_SINGLE_THREAD: Mutex<bool> = Mutex::new(false);
pub const C_MOUNT_PATH: &str = "/usr/bin/mount";
pub const C_UMOUNT_PATH: &str = "/usr/bin/umount";
pub const C_MKDIR_PATH: &str = "/usr/bin/mkdir";
pub const C_DD_PATH: &str = "/usr/bin/dd";
pub const C_MKFS_PATH: &str = "/usr/sbin/mkfs.ext4";
pub const C_LOSETUP_PATH: &str = "/usr/sbin/losetup";
pub const TEST_MOUNT_POINT: &str = "mount_point";
pub const TEST_MOUNT_SRC: &str = "/dev/loop";

pub fn setup_loop_device(ts: &TestScenario) -> String {
    const TEST_TEMP_FILE: &str = "ext4.img";
    ts.cmd(C_MKDIR_PATH).arg(TEST_MOUNT_POINT).run();
    ts.cmd(C_DD_PATH)
        .args(&[
            "if=/dev/zero",
            &format!("of={}", TEST_TEMP_FILE),
            "bs=1M",
            "count=5",
        ])
        .run();
    ts.cmd(C_MKFS_PATH).arg(TEST_TEMP_FILE).run();
    let losetup_res =
        run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-f", "--show", TEST_TEMP_FILE]).unwrap();
    losetup_res.stdout_str().trim().to_string()
}

pub fn compare_mount_result(
    c_res: CmdResult,
    rust_res: CmdResult,
    c_mount_res: CmdResult,
    rust_mount_res: CmdResult,
) {
    println!("c_res: {}\n{}\n", c_res.stdout_str(), c_res.stderr_str());
    println!(
        "rust_res: {}\n{}\n",
        rust_res.stdout_str(),
        rust_res.stderr_str()
    );

    c_mount_res.stdout_is(rust_mount_res.stdout_str());
    c_res.stderr_is(rust_res.stderr_str());
    c_res.stdout_is(rust_res.stdout_str());
}

fn run_and_compare(ts: &TestScenario, in_args: &[&str]) {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let loopdevice = &setup_loop_device(ts);
    let mut args = Vec::from(in_args);
    for i in 0..args.len() {
        if args[i] == TEST_MOUNT_SRC {
            args[i] = loopdevice;
        }
    }

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[TEST_MOUNT_POINT]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, &args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[TEST_MOUNT_POINT]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();

    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}

#[test]
fn test_mount_print_all() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = TestScenario::new(util_name!());
    let c_res = ts.cmd(C_MOUNT_PATH).run();
    let rust_res = ts.ucmd().run();
    c_res.stdout_is(rust_res.stdout_str());
}

#[test]
fn test_mount_print_all_only_types() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = TestScenario::new(util_name!());
    let args = &["-t", "ext4"];
    let c_res = ts.cmd(C_MOUNT_PATH).args(args).run();
    let rust_res = ts.ucmd().args(args).run();
    c_res.stdout_is(rust_res.stdout_str());
}

#[test]
fn test_mount_show_labels() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = TestScenario::new(util_name!());
    let args = &["-l"];
    let c_res = ts.cmd(C_MOUNT_PATH).args(args).run();
    let rust_res = ts.ucmd().args(args).run();
    c_res.stdout_is(rust_res.stdout_str());
}

#[test]
fn test_mount_verbose() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-v", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_read_only() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-r", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_all() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-a"]);
}

#[test]
fn test_mount_types() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-t", "ext4", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_options() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-o", "ro,noexec", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_bind() {
    let ts = TestScenario::new(util_name!());
    ts.cmd(C_MKDIR_PATH).arg("source").run();
    run_and_compare(&ts, &["--bind", "source", TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_move() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    const NEW_MOUNT_POINT_A: &str = "mount_point/new_target_a";
    const NEW_MOUNT_POINT_B: &str = "mount_point/new_target_b";
    let args = &["--move", NEW_MOUNT_POINT_A, NEW_MOUNT_POINT_B];
    let loopdevice = &setup_loop_device(ts);

    // Prepare
    run_cmd_as_root_ignore_ci(
        ts,
        C_MOUNT_PATH,
        &["-B", "--make-private", TEST_MOUNT_POINT, TEST_MOUNT_POINT],
    )
    .unwrap();
    ts.cmd(C_MKDIR_PATH)
        .arg(NEW_MOUNT_POINT_A)
        .arg(NEW_MOUNT_POINT_B)
        .run();
    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, NEW_MOUNT_POINT_A]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, NEW_MOUNT_POINT_A]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[TEST_MOUNT_POINT]).unwrap();
    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}

#[test]
fn test_mount_label() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    const TEST_LABEL: &str = "easyblock";
    let args = &["-L", TEST_LABEL, TEST_MOUNT_POINT];
    let loopdevice = &setup_loop_device(ts);

    // Set a new label
    run_cmd_as_root_ignore_ci(ts, "/usr/sbin/e2label", &[loopdevice, TEST_LABEL]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}

#[test]
fn test_mount_uuid() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    const TEST_UUID: &str = "b191c10c-448d-4e80-a8d6-e5303260cf5f";
    let args = &["-U", TEST_UUID, TEST_MOUNT_POINT];
    let loopdevice = &setup_loop_device(ts);

    // Set a new label
    run_cmd_as_root_ignore_ci(ts, "/usr/sbin/tune2fs", &[loopdevice, "-U", TEST_UUID]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}

#[test]
fn test_mount_no_mtab() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["--no-mtab", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_no_canonicalize() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(
        &ts,
        &["--no-canonicalize", TEST_MOUNT_SRC, TEST_MOUNT_POINT],
    );
}

#[test]
fn test_mount_fake() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-f", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_fork() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-a", "-F"]);
}

#[test]
fn test_mount_internal_only() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["--internal-only", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_make_private() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    let args = &["--make-private", TEST_MOUNT_POINT];
    let loopdevice = &setup_loop_device(ts);

    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}

#[test]
fn test_mount_read_write() {
    let ts = TestScenario::new(util_name!());
    run_and_compare(&ts, &["-w", TEST_MOUNT_SRC, TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_namespace() {
    let ts = TestScenario::new(util_name!());
    let res = ts.cmd("/usr/bin/realpath").arg(TEST_MOUNT_POINT).run();
    let realpath = res.stdout_str().trim();
    run_and_compare(
        &ts,
        &[
            "-N",
            &getpid().as_raw().to_string(),
            TEST_MOUNT_SRC,
            realpath,
        ],
    );
}

#[test]
fn test_mount_fstab_alternative() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    const NEW_FSTAB: &str = "new_fstab";
    let loopdevice = &setup_loop_device(ts);
    let args = &["-T", NEW_FSTAB, loopdevice];

    ts.cmd("/usr/bin/tee")
        .arg(NEW_FSTAB)
        .run_piped_stdin(format!(
            "{} {} ext4 rw,nosuid,noexec,relatime 0 0\n",
            loopdevice, TEST_MOUNT_POINT
        ));

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}

#[test]
fn test_mount_rbind() {
    let ts = TestScenario::new(util_name!());
    ts.cmd(C_MKDIR_PATH).arg("source").run();
    run_and_compare(&ts, &["-R", "source", TEST_MOUNT_POINT]);
}

#[test]
fn test_mount_make_shared() {
    let _lock = KEEP_SINGLE_THREAD.lock();
    let ts = &TestScenario::new(util_name!());
    let args = &["--make-shared", TEST_MOUNT_POINT];
    let loopdevice = &setup_loop_device(ts);

    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run C programe
    let c_res = run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, args).unwrap();
    let c_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    run_cmd_as_root_ignore_ci(ts, C_MOUNT_PATH, &[loopdevice, TEST_MOUNT_POINT]).unwrap();

    // Run rust programe
    let rust_res = run_ucmd_as_root_ignore_ci(ts, args).unwrap();
    let rust_mount_res = ts.cmd(C_MOUNT_PATH).run();
    run_cmd_as_root_ignore_ci(ts, C_UMOUNT_PATH, &[loopdevice]).unwrap();

    // Clean
    run_cmd_as_root_ignore_ci(ts, C_LOSETUP_PATH, &["-d", loopdevice]).unwrap();
    compare_mount_result(c_res, rust_res, c_mount_res, rust_mount_res);
}
