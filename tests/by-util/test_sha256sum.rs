// This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;
use rand::Rng;
use std::env::set_var;
use std::fs;
use std::path::Path;
const C_SHA256_PATH: &str = "/usr/bin/sha256sum";

fn generate_random_string(length: usize) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789-+;,[]{}\\?!@#$%^&*()";
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect();
    random_string
}

#[test]
fn test_sha256sum_hash_stdin() {
    let input = "Hello, World!\r\n";
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_SHA256_PATH).pipe_in(input).succeeds();
        new_ucmd!()
            .pipe_in(input)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd()
            .pipe_in(input)
            .succeeds()
            .stdout_only("92b772380a3f8e27a93e57e6deeca6c01da07f5aadce78bb2fbb20de10a66925  -\n");
    }
}

#[test]
fn test_sha256sum_hash_large_stdin() {
    let input: &str = &generate_random_string(1000000);
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_SHA256_PATH).pipe_in(input).succeeds();
        new_ucmd!()
            .pipe_in(input)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd().pipe_in(input).succeeds();
    }
}

#[test]
fn test_sha256sum_hash_file() {
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() && Path::new("/usr/bin/dd").exists() {
        let task = TestScenario::new(util_name!());
        let _ = task
            .cmd("dd")
            .arg("if=/dev/random")
            .arg("of=/tmp/sha256sum_4M")
            .arg("bs=1M")
            .arg("count=4")
            .run();
        let res = task.cmd(C_SHA256_PATH).arg("/tmp/sha256sum_4M").succeeds();
        new_ucmd!()
            .arg("/tmp/sha256sum_4M")
            .succeeds()
            .stdout_only(res.stdout_str());
        task.cmd("rm")
            .arg("-rf")
            .arg("/tmp/sha256sum_4M")
            .succeeds();
    } else {
        let task = TestScenario::new(util_name!());
        let _dd = task
            .cmd("dd")
            .arg("if=/dev/random")
            .arg("of=/tmp/sha256sum_4M")
            .arg("bs=1M")
            .arg("count=4")
            .run();
        task.ucmd().arg("/tmp/sha256sum_4M").succeeds();
        task.cmd("rm")
            .arg("-rf")
            .arg("/tmp/sha256sum_4M")
            .succeeds();
    }
}

#[test]
fn test_sha256sum_hash_large_file() {
    // Testing file that is too large will time out
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() && Path::new("/usr/bin/dd").exists() {
        let task = TestScenario::new(util_name!());
        let _ = task
            .cmd("dd")
            .arg("if=/dev/random")
            .arg("of=/tmp/sha256sum_1G")
            .arg("bs=1M")
            .arg("count=1000")
            .run();
        let res = task.cmd(C_SHA256_PATH).arg("/tmp/sha256sum_1G").succeeds();
        new_ucmd!()
            .arg("/tmp/sha256sum_1G")
            .succeeds()
            .stdout_only(res.stdout_str());
        task.cmd("rm")
            .arg("-rf")
            .arg("/tmp/sha256sum_1G")
            .succeeds();
    } else {
        let task = TestScenario::new(util_name!());
        let _dd = task
            .cmd("dd")
            .arg("if=/dev/random")
            .arg("of=/tmp/sha256sum_1G")
            .arg("bs=1M")
            .arg("count=1000")
            .run();
        task.ucmd().arg("/tmp/sha256sum_1G").succeeds();
        task.cmd("rm")
            .arg("-rf")
            .arg("/tmp/sha256sum_1G")
            .succeeds();
    }
}

#[test]
fn test_sha256sum_hash_stdin_file_mix() {
    let input = "Hello, World!\r\n";
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let res = task
            .cmd(C_SHA256_PATH)
            .arg("input1.txt")
            .arg("-")
            .arg("input2.txt")
            .pipe_in(input)
            .succeeds();
        new_ucmd!()
            .arg("input1.txt")
            .arg("-")
            .arg("input2.txt")
            .pipe_in(input)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        task.ucmd()
            .arg("input1.txt")
            .arg("-")
            .arg("input2.txt")
            .pipe_in(input)
            .succeeds();
    }
}

#[test]
fn test_sha256sum_hash_binary() {
    let input = "Hello, World!\r\n";
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_SHA256_PATH).arg("-b").pipe_in(input).succeeds();
        new_ucmd!()
            .arg("-b")
            .pipe_in(input)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd()
            .arg("-b")
            .pipe_in(input)
            .succeeds()
            .stdout_only("92b772380a3f8e27a93e57e6deeca6c01da07f5aadce78bb2fbb20de10a66925 *-\n");
    }
}

#[test]
fn test_sha256sum_hash_tag() {
    let input = "Hello, World!\r\n";
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let task = TestScenario::new(util_name!());
        let res = task
            .cmd(C_SHA256_PATH)
            .arg("--tag")
            .pipe_in(input)
            .succeeds();
        new_ucmd!()
            .arg("--tag")
            .pipe_in(input)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd()
            .arg("--tag")
            .pipe_in(input)
            .succeeds()
            .stdout_only(
                "SHA256 (-) = 92b772380a3f8e27a93e57e6deeca6c01da07f5aadce78bb2fbb20de10a66925\n",
            );
    }
}

#[test]
fn test_sha256sum_hash_zero() {
    let input = "Hello, World!\r\n";
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let task = TestScenario::new(util_name!());
        let res = task
            .cmd(C_SHA256_PATH)
            .arg("--zero")
            .pipe_in(input)
            .succeeds()
            .stdout_str()
            .to_owned();
        new_ucmd!()
            .arg("--zero")
            .pipe_in(input)
            .succeeds()
            .stdout_only(res);
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd()
            .arg("-z")
            .pipe_in(input)
            .succeeds()
            .stdout_only("92b772380a3f8e27a93e57e6deeca6c01da07f5aadce78bb2fbb20de10a66925  -\0");
    }
}

#[test]
fn test_sha256sum_check_stdin() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    println!("{}", task.cmd("pwd").run().stdout_str());
    task.cmd("cp")
        .arg("check3.txt")
        .arg("/tmp/")
        .run()
        .success();
    // task.cmd("pwd").run().stdout_str();
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let res = task
            .cmd(C_SHA256_PATH)
            .arg("-c")
            .pipe_in(fs::read("/tmp/check3.txt").unwrap())
            .succeeds();
        new_ucmd!()
            .arg("-c")
            .pipe_in(fs::read("/tmp/check3.txt").unwrap())
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        task.ucmd()
            .arg("-c")
            .pipe_in(fs::read("/tmp/check3.txt").unwrap())
            .succeeds()
            .stdout_only("input1.txt: OK\ninput2.txt: OK\ninput3.txt: OK\n");
    }
    task.cmd("rm").arg("-rf").arg("/tmp/check3.txt").succeeds();
}

#[test]
fn test_sha256sum_check_file() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let sha256sum_path = Path::new(C_SHA256_PATH);
    if sha256sum_path.exists() {
        let res = task
            .cmd(C_SHA256_PATH)
            .arg("-c")
            .arg("check3.txt")
            .succeeds();
        new_ucmd!()
            .arg("-c")
            .arg("check3.txt")
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        task.ucmd().arg("-c").arg("check3.txt").succeeds();
    }
}

#[test]
fn test_sha256sum_check_warn() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let warn_len = task
        .ucmd()
        .arg("-c")
        .arg("-w")
        .arg("check3_improper.txt")
        .run()
        .stderr_str()
        .len();
    assert!(warn_len > 0);
}

#[test]
fn test_sha256sum_check_ignore_missing() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let mut stderr_len = task
        .ucmd()
        .arg("-c")
        .arg("--ignore-missing")
        .arg("check3_missing.txt")
        .run()
        .stderr_str()
        .len();
    assert!(stderr_len == 0);
    stderr_len = task
        .ucmd()
        .arg("-c")
        .arg("check3_missing.txt")
        .run()
        .stderr_str()
        .len();
    assert!(stderr_len > 0);
}

#[test]
fn test_sha256sum_check_quiet() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let mut stdout_len = task
        .ucmd()
        .arg("-c")
        .arg("--quiet")
        .arg("check3.txt")
        .run()
        .stdout_str()
        .len();
    assert!(stdout_len == 0);
    stdout_len = task
        .ucmd()
        .arg("-c")
        .arg("check3.txt")
        .run()
        .stdout_str()
        .len();
    assert!(stdout_len > 0);
}

#[test]
fn test_sha256sum_check_status() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let mut stdout_len = task
        .ucmd()
        .arg("-c")
        .arg("--status")
        .arg("check3.txt")
        .run()
        .code_is(0)
        .stdout_str()
        .len();
    assert!(stdout_len == 0);
    stdout_len = task
        .ucmd()
        .arg("-c")
        .arg("--status")
        .arg("check3_fail.txt")
        .run()
        .code_is(1)
        .stdout_str()
        .len();
    assert!(stdout_len == 0);
}

#[test]
fn test_sha256sum_check_strict() {
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    task.ucmd()
        .arg("-c")
        .arg("--strict")
        .arg("check3.txt")
        .run()
        .code_is(0);
    task.ucmd()
        .arg("-c")
        .arg("--strict")
        .arg("check3_improper.txt")
        .run()
        .code_is(1);
    task.ucmd()
        .arg("-c")
        .arg("check3_improper.txt")
        .run()
        .code_is(0);
}
