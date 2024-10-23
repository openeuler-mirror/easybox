//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::fs::{self, File};
use std::io::{self, BufRead};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

///
#[cfg(target_os = "linux")]
pub fn user_busy(name: &str, uid: u32) -> bool {
    user_busy_processes(name, uid)
}

///
#[cfg(not(target_os = "linux"))]
pub fn user_busy(name: &str) -> bool {
    user_busy_utmp(name)
}

///
#[cfg(target_os = "linux")]
pub fn user_busy_processes(name: &str, uid: u32) -> bool {
    let proc_path = Path::new("/proc");
    let sbroot = fs::metadata("/").expect("Failed to stat root directory");

    if let Ok(entries) = fs::read_dir(proc_path) {
        for entry in entries.flatten() {
            let entry_name = entry.file_name().into_string().unwrap();
            if entry_name == "." || entry_name == ".." {
                continue;
            }

            let pid = match entry_name.parse::<u32>() {
                Ok(pid) => pid,
                Err(_) => continue, // Skip non-PID directories
            };

            let root_path = format!("/proc/{}/root", pid);
            if let Ok(sbroot_process) = fs::metadata(&root_path) {
                if sbroot.dev() != sbroot_process.dev() || sbroot.ino() != sbroot_process.ino() {
                    continue;
                }
            } else {
                continue;
            }

            if check_status(name, &pid.to_string(), uid) {
                println!("user {} is currently used by process {}", name, pid);
                return true;
            }

            let task_path = format!("/proc/{}/task", pid);
            if let Ok(task_dir) = fs::read_dir(&task_path) {
                for task_entry in task_dir.flatten() {
                    let tid = match task_entry.file_name().into_string().unwrap().parse::<u32>() {
                        Ok(tid) => tid,
                        Err(_) => continue,
                    };
                    if tid == pid {
                        continue;
                    }
                    if check_status(name, &tid.to_string(), uid) {
                        println!("user {} is currently used by process {}", name, pid);
                        return true;
                    }
                }
            }
        }
    }
    false
}

///
#[cfg(target_os = "linux")]
pub fn check_status(_name: &str, sname: &str, uid: u32) -> bool {
    let status_path = format!("/proc/{}/status", sname);
    if let Ok(file) = File::open(&status_path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.starts_with("Uid:") {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    let ruid: u32 = fields[1].parse().unwrap_or(0);
                    let euid: u32 = fields[2].parse().unwrap_or(0);
                    let suid: u32 = fields[3].parse().unwrap_or(0);
                    if ruid == uid || euid == uid || suid == uid {
                        return true;
                    }
                }
                break;
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
pub fn user_busy_utmp(name: &str) -> bool {
    use libc::{getutxent, kill, setutxent, utmpx, USER_PROCESS};

    unsafe {
        setutxent();
        while let Some(ut) = getutxent().as_ref() {
            if ut.ut_type != USER_PROCESS {
                continue;
            }
            let ut_user = CString::from_raw(ut.ut_user.as_ptr() as *mut i8)
                .to_str()
                .unwrap_or_default();
            if ut_user != name {
                continue;
            }
            if kill(ut.ut_pid, 0) == 0 {
                println!("user {} is currently logged in", name);
                return true;
            }
        }
    }
    false
}
