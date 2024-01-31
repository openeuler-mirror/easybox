// This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use std::mem::{size_of, zeroed};

use libc::{cpu_set_t, getpid, sched_getaffinity, sched_setaffinity};
use procfs::process::Process;

use crate::common::util::*;

const C_TASKSET_PATH: &str = "/usr/bin/taskset";

#[test]
fn test_use_mask_launch_new_process() {
    new_ucmd!()
        .args(&["1", "sleep", "1"])
        .succeeds()
        .no_stdout();
}

#[test]
fn test_use_list_launch_new_process() {
    new_ucmd!()
        .args(&["-c", "0-1", "sleep", "1"])
        .succeeds()
        .no_stdout();
}

#[test]
fn test_retrieve_mask_from_pid() {
    let test_args = &["-p", "1"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_TASKSET_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    task.ucmd()
        .args(test_args)
        .succeeds()
        .stdout_is_bytes(c_res.stdout());
}

#[test]
fn test_retrieve_list_from_pid() {
    let test_args = &["-pc", "1"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_TASKSET_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    task.ucmd()
        .args(test_args)
        .succeeds()
        .stdout_is_bytes(c_res.stdout());
}

// Test helper function
unsafe fn helper_get_cpu_affinity(pid: i32) -> Box<cpu_set_t> {
    let set: Box<cpu_set_t> = Box::new(zeroed());
    let pset = Box::into_raw(set);
    sched_getaffinity(pid, size_of::<cpu_set_t>(), pset);
    return Box::from_raw(pset);
}

// Test helper function
unsafe fn helper_set_cpu_affinity(pid: i32, set: Box<cpu_set_t>) {
    let pset = Box::into_raw(set);
    sched_setaffinity(pid, size_of::<cpu_set_t>(), pset);
}

#[test]
fn test_set_mask_for_pid() {
    let mypid = unsafe { getpid() };
    let test_args = &["-p", "2", &mypid.to_string()];
    // Get original cpu affinity of this process
    let set = unsafe { helper_get_cpu_affinity(mypid) };
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_TASKSET_PATH).args(test_args).succeeds();
    // Restore cpu affinity of this process
    unsafe {
        helper_set_cpu_affinity(mypid, set.clone());
    }
    // Run Rust taskset and compare the output with origin one
    task.ucmd()
        .args(test_args)
        .succeeds()
        .stdout_is_bytes(c_res.stdout());
    // Restore cpu affinity of this process
    unsafe {
        helper_set_cpu_affinity(mypid, set);
    }
}

#[test]
fn test_set_list_for_pid() {
    let mypid = unsafe { getpid() };
    let test_args = &["-pc", "0-3:2", &mypid.to_string()];
    // Get original cpu affinity of this process
    let set = unsafe { helper_get_cpu_affinity(mypid) };
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_TASKSET_PATH).args(test_args).succeeds();
    // Restore cpu affinity of this process
    unsafe { helper_set_cpu_affinity(mypid, set.clone()) }
    // Run Rust taskset and compare the output with origin one
    task.ucmd()
        .args(test_args)
        .succeeds()
        .stdout_is_bytes(c_res.stdout());
    // Restore cpu affinity of this process
    unsafe {
        helper_set_cpu_affinity(mypid, set);
    }
}

// Test helper function
fn helper_set_cpu_affinity_for_all_tasks(pid: i32, set: Box<cpu_set_t>) {
    let pc = Process::new(pid).unwrap();
    let pset = Box::into_raw(set);
    for task in pc.tasks().unwrap() {
        unsafe {
            sched_setaffinity(task.unwrap().tid, size_of::<cpu_set_t>(), pset);
        }
    }
}

#[test]
fn test_set_mask_for_pid_all_tasks() {
    let mypid = unsafe { getpid() };
    let test_args = &["-ap", "2", &mypid.to_string()];
    // Get original cpu affinity of this process
    let set = unsafe { helper_get_cpu_affinity(mypid) };
    let task = TestScenario::new(util_name!());
    // Note that there are many threads exist with this process,
    // So we do not need to start any other thread.
    // Run original C taskset from system path
    let c_res = task.cmd(C_TASKSET_PATH).args(test_args).succeeds();
    // Restore cpu affinity of this process's all tasks
    helper_set_cpu_affinity_for_all_tasks(mypid, set.clone());
    // Run Rust taskset and compare the output with origin one
    task.ucmd()
        .args(test_args)
        .succeeds()
        .stdout_is_bytes(c_res.stdout());
    // Restore cpu affinity of this process's all tasks
    helper_set_cpu_affinity_for_all_tasks(mypid, set);
}

#[test]
fn test_set_invalid_mask_for_pid() {
    let mypid = unsafe { getpid() };
    let test_args = &["-p", "0", &mypid.to_string()];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_TASKSET_PATH).args(test_args).fails();
    // Run Rust taskset and compare the output with origin one
    task.ucmd()
        .args(test_args)
        .fails()
        .code_is(c_res.code())
        .stdout_is_bytes(c_res.stdout())
        .stderr_is_bytes(c_res.stderr());
}
