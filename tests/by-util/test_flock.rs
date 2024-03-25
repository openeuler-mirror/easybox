// This file is part of the easybox package.
//
// (c) Allen Xu <xubo3006@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;
use std::time::{Duration, SystemTime};

#[test]
fn test_lock() {
    let ts = TestScenario::new(util_name!());
    let start = SystemTime::now();
    let _child = ts
        .ucmd()
        .args(&[
            "--shared",
            "--conflict-exit-code",
            "123",
            "lockfile.txt",
            "-c",
            "echo \"Locking\"; sleep 3 ;echo \"Unlocking\" > outputfile.txt 2>&1 &",
        ])
        .run_no_wait();

    std::thread::sleep(Duration::from_millis(500));

    // test nonblock
    ts.ucmd()
        .args(&[
            "--nonblock",
            "--conflict-exit-code",
            "123",
            "lockfile.txt",
            "echo",
            "\"You will never see this!\"",
        ])
        .fails()
        .code_is(123);

    // test no-fork
    ts.ucmd()
        .args(&[
            "--no-fork",
            "--nonblock",
            "--conflict-exit-code",
            "123",
            "lockfile.txt",
            "echo",
            "\"You will never see this!\"",
        ])
        .fails()
        .code_is(123);

    // test shared
    ts.ucmd()
        .args(&["--shared", "lockfile.txt", "echo", "Have shared lock"])
        .succeeds()
        .code_is(0);

    // test exclusive
    ts.ucmd()
        .args(&[
            "--nonblock",
            "--exclusive",
            "--conflict-exit-code",
            "123",
            "lockfile.txt",
            "echo",
            "\"You will never see this!\"",
        ])
        .fails()
        .code_is(123);

    // test timeout
    ts.ucmd()
        .args(&[
            "--timeout",
            "5",
            "--conflict-exit-code",
            "5",
            "lockfile.txt",
            "echo",
            "\"After timeout\"",
        ])
        .succeeds()
        .code_is(0);

    // test time check
    let end = SystemTime::now();
    if let Ok(timediff) = end.duration_since(start) {
        let n = timediff.as_secs();
        if n < 3 {
            panic!("general lock failed [{} sec]", n);
        } else if n > 5 {
            panic!("wait too long [{} sec]", n)
        }
    }
}
