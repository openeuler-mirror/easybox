// This file is part of the easybox package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use lazy_static::lazy_static;

use crate::common::util::*;
use std::{
    env, fs,
    io::{Read, Write},
    panic,
    process::{Command, ExitStatus, Stdio},
};
const SHADOW_PATH: &str = "/etc/shadow";
const PASSWD_PATH: &str = "/etc/passwd";
const C_CHAGE_PATH: &str = "/usr/bin/chage";
const EASYBOX_PATH: &str = "target/debug/easybox";

const SCRIPTS_PATH: &str = "tests/fixtures/chage/scripts";
const SHADOW_FILE: &str = "/etc/shadow";

const SHADOW_LOCK: &str = "/etc/shadow.lock";
const PASSWD_LOCK: &str = "/etc/passwd.lock";

lazy_static! {
    static ref CURRENT_DIR: String = {
        let path = env::current_dir().expect("Failed to get current directory");
        path.to_str()
            .expect("Failed to convert path to string")
            .to_string()
    };
}

fn sudo_read_file(filename: &str) -> String {
    let output = Command::new("sudo")
        .args(["-E", "--non-interactive", "cat", filename])
        .output()
        .expect("failed to execute command");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn sudo_write_file(filename: &str, content: &str) {
    Command::new("sudo")
        .args([
            "-E",
            "--non-interactive",
            "bash",
            "-c",
            format!("printf '%s' '{}' > {}", content, filename).as_str(),
        ])
        .output()
        .expect("failed to execute command");
}

fn sudo_create_file(file: &str) {
    TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&["-E", "--non-interactive", "touch", file])
        .run()
        .success();
}

fn sudo_remove_file(file: &str) {
    TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&["-E", "--non-interactive", "rm", file])
        .run()
        .success();
}

fn create_user(username: &str) {
    let useradd_res = TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&["-E", "--non-interactive", "/usr/sbin/useradd"])
        .arg(username)
        .run();
    useradd_res.success();
}

fn delete_user(username: &str) {
    let userdel_res = TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&["-E", "--non-interactive", "/usr/sbin/userdel"])
        .arg(username)
        .run();
    userdel_res.success();
}

fn copy_file(src: String, dst: String) {
    Command::new("sudo")
        .args(["-E", "--non-interactive", "cp", &src, &dst])
        .output()
        .unwrap();
}

fn append_file(src: String, dst: String) {
    let output = Command::new("sudo")
        .arg("bash")
        .arg("-c")
        .arg(format!("cat {} >> {}", src, dst))
        .output()
        .expect("failed to execute command");

    assert!(output.status.success());
}

fn save() {
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        copy_file(format!("/etc/{}", file), format!("/tmp/{}", file));
        copy_file(format!("/etc/{}-", file), format!("/tmp/{}-", file));
    }
}

fn restore() {
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        copy_file(format!("/tmp/{}", file), format!("/etc/{}", file));
        copy_file(format!("/tmp/{}-", file), format!("/etc/{}-", file));
    }
}

fn get_entry(username: &str, file_name: &str) -> String {
    let content = sudo_read_file(file_name);
    let lines: Vec<&str> = content.split("\n").collect();
    for line in lines {
        if line.starts_with(username) {
            return line.to_string();
        }
    }
    "".to_string()
}

fn expect_bytes<R: Read>(file: &mut R, bytes: &[u8]) {
    let size = bytes.len();
    let mut buf = [0; 1];
    for i in 0..size {
        file.read_exact(&mut buf).unwrap();
        assert_eq!(buf[0], bytes[i]);
    }
}

fn run_command(args: &[&str]) {
    Command::new("sudo")
        .args(["-E", "--non-interactive", EASYBOX_PATH, "chage"])
        .args(args)
        .output()
        .unwrap();
}

fn run_and_compare(test_args: &[&str]) {
    let shadow_copy = sudo_read_file(SHADOW_PATH);
    let passwd_copy = sudo_read_file(PASSWD_PATH);

    let rust_res = TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&[
            "-E",
            "--non-interactive",
            format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
            "chage",
        ])
        .args(test_args)
        .run();
    // let rust_res = TestScenario::new(util_name!()).ucmd().args(test_args).run();

    let rust_shadow = sudo_read_file(SHADOW_PATH);
    let rust_passwd = sudo_read_file(PASSWD_PATH);

    sudo_write_file(SHADOW_PATH, &shadow_copy);
    sudo_write_file(PASSWD_PATH, &passwd_copy);
    let c_res = TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&["-E", "--non-interactive", C_CHAGE_PATH])
        .args(test_args)
        .run();

    assert_eq!(rust_res.stdout_str(), c_res.stdout_str());
    assert_eq!(rust_res.stderr_str(), c_res.stderr_str());
    assert_eq!(rust_res.code(), c_res.code());

    let c_shadow = sudo_read_file(SHADOW_PATH);
    let c_passwd = sudo_read_file(PASSWD_PATH);
    assert_eq!(rust_shadow, c_shadow);
    assert_eq!(rust_passwd, c_passwd);
}

fn run_scripts_and_compare(script1: String, script2: String) {
    let res1 = TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&[
            "-E",
            "--non-interactive",
            &format!("{}/{}", *CURRENT_DIR, script1),
        ])
        .arg(&*CURRENT_DIR)
        .run();
    let res2 = TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&[
            "-E",
            "--non-interactive",
            &format!("{}/{}", *CURRENT_DIR, script2),
        ])
        .arg(&*CURRENT_DIR)
        .run();

    // assert_eq!(res1.stdout_str(), res2.stdout_str());
    assert_eq!(res1.stderr_str(), res2.stderr_str());
    assert_eq!(res1.code(), res2.code());
}

fn run_and_get_stdout(test_args: &[&str]) -> String {
    let res = Command::new("sudo")
        .args([EASYBOX_PATH, "chage"])
        .args(test_args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&res.stdout).to_string()
}

fn run_and_get_stderr(test_args: &[&str]) -> String {
    let res = Command::new("sudo")
        .args([EASYBOX_PATH, "chage"])
        .args(test_args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&res.stderr).to_string()
}

fn run_and_get_result(test_args: &[&str]) -> CmdResult {
    TestScenario::new(util_name!())
        .cmd("sudo")
        .args(&[
            "-E",
            "--non-interactive",
            format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
            "chage",
        ])
        .args(test_args)
        .run()
}

// #[test]
fn test_chage() {
    let username = "myuser";
    // prepare
    save();
    create_user(username);

    let result = panic::catch_unwind(|| {
        // test list
        // chage -l username
        run_and_compare(&["-l", username]);
        // chage -l root
        run_and_compare(&["-l", "root"]);
        // chage -l nonexist
        run_and_compare(&["-l", "nonexist"]);
        // chage -l -i root
        run_and_compare(&["-l", "-i", "root"]);

        // test lastday
        // chage -d 0 username
        run_and_compare(&["-d", "0", username]);
        // chage -d 10000000 username
        run_and_compare(&["-d", "10000000", username]);
        // chage -d 2024-9-10 username
        run_and_compare(&["-d", "2024-9-10", username]);
        // chage -d -1 username
        run_and_compare(&["-d", "-1", username]);
        // chage -l username
        run_and_compare(&["-l", username]);

        // test expiredate
        // chage -E 0 username
        run_and_compare(&["-E", "0", username]);
        // chage -E 10000000 username
        run_and_compare(&["-E", "10000000", username]);
        // chage -E 2024-9-10 username
        run_and_compare(&["-E", "2024-9-10", username]);
        // chage -E -1 username
        run_and_compare(&["-E", "-1", username]);
        // chage -l username
        run_and_compare(&["-l", username]);

        // test mindays
        // chage -m 0 username
        run_and_compare(&["-m", "0", username]);
        // chage -m 10000000 username
        run_and_compare(&["-m", "10000000", username]);
        // chage -m -1 username
        run_and_compare(&["-m", "-1", username]);
        // chage -l username
        run_and_compare(&["-l", username]);

        // test maxdays
        // chage -M 0 username
        run_and_compare(&["-M", "0", username]);
        // chage -M 10000000 username
        run_and_compare(&["-M", "10000000", username]);
        // chage -M -1 username
        run_and_compare(&["-M", "-1", username]);
        // chage -l username
        run_and_compare(&["-l", username]);

        // test warndays
        // chage -W 0 username
        run_and_compare(&["-W", "0", username]);
        // chage -W 10000000 username
        run_and_compare(&["-W", "10000000", username]);
        // chage -W -1 username
        run_and_compare(&["-W", "-1", username]);
        // chage -l username
        run_and_compare(&["-l", username]);

        // test inactive
        // chage -I 0 username
        run_and_compare(&["-I", "0", username]);
        // chage -I 10000000 username
        run_and_compare(&["-I", "10000000", username]);
        // chage -I -1 username
        run_and_compare(&["-I", "-1", username]);
        // chage -l username
        run_and_compare(&["-l", username]);
    });
    //restore
    delete_user(username);
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_01() {
    Command::new("sudo")
        .args(["bash", "-c", "\"su -\""])
        .output()
        .unwrap();
    save();
    let data_path: &str = "tests/fixtures/chage/01data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // test option -l
        assert_eq!(
            fs::read_to_string(format!("{}/chage1", data_path)).unwrap(),
            run_and_get_stdout(&["-l", "myuser1"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage2", data_path)).unwrap(),
            run_and_get_stdout(&["-l", "myuser2"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage3", data_path)).unwrap(),
            run_and_get_stdout(&["-l", "myuser3"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage4", data_path)).unwrap(),
            run_and_get_stdout(&["-l", "myuser4"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage5", data_path)).unwrap(),
            run_and_get_stdout(&["-l", "myuser5"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage6", data_path)).unwrap(),
            run_and_get_stdout(&["-l", "myuser6"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage7", data_path)).unwrap(),
            run_and_get_stdout(&["--list", "myuser7"])
        );

        assert_eq!(
            fs::read_to_string(format!("{}/chage8", data_path)).unwrap(),
            run_and_get_stderr(&["-l", "myuser8"])
        );

        // test option -d
        // chage -d 2001-10-02 myuser7
        run_command(&["-d", "2001-10-02", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:11597:0:99999:7:1::"
        );
        // chage -d -1 myuser7
        run_command(&["-d", "-1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.::0:99999:7:1::"
        );
        // chage -d 0 myuser7
        run_command(&["-d", "0", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:0:0:99999:7:1::"
        );
        // chage --lastday 2011-11-02 myuser7
        run_command(&["--lastday", "2011-11-02", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:1::"
        );

        // test option -E
        // chage -E 2010-10-02 myuser7
        run_command(&["-E", "2010-10-02", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:1:14884:"
        );
        // chage _E -1 myuser7
        run_command(&["-E", "-1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:1::"
        );
        // chage -E 0 myuser7
        run_command(&["-E", "0", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:1:0:"
        );
        // chage --expiredate 2020-02-02 myuser7
        run_command(&["--expiredate", "2020-02-02", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:1:18294:"
        );

        // test option -I
        // chage -I 42 myuser7
        run_command(&["-I", "42", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:42:18294:"
        );
        // chage -I -1 myuser7
        run_command(&["-I", "-1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7::18294:"
        );
        // chage -I 0 myuser7
        run_command(&["-I", "0", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:0:18294:"
        );
        // chage --inactive 12 myuser7
        run_command(&["--inactive", "12", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:12:18294:"
        );

        // test option -m
        // chage -m 24 myuser7
        run_command(&["-m", "24", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:24:99999:7:12:18294:"
        );
        // chage -m -1 myuser7
        run_command(&["-m", "-1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280::99999:7:12:18294:"
        );
        // chage -m 0 myuser7
        run_command(&["-m", "0", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:0:99999:7:12:18294:"
        );
        // chage -min 1 myuser7
        run_command(&["--min", "1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:99999:7:12:18294:"
        );

        // test option -M
        // chage -M 25 myuser7
        run_command(&["-M", "25", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:25:7:12:18294:"
        );
        // chage -M -1 myuser7
        run_command(&["-M", "-1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1::7:12:18294:"
        );
        // chage -M 0 myuser7
        run_command(&["-M", "0", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:0:7:12:18294:"
        );
        // chage --max 2 myuser7
        run_command(&["--max", "2", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:2:7:12:18294:"
        );

        // test option -W
        // chage -W 26 myuser7
        run_command(&["-W", "26", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:2:26:12:18294:"
        );
        // chage -W -1 myuser7
        run_command(&["-W", "-1", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:2::12:18294:"
        );
        // chage -W 0 myuser7
        run_command(&["-W", "0", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:2:0:12:18294:"
        );
        // chage --warndays 3 myuser7
        run_command(&["--warndays", "3", "myuser7"]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:15280:1:2:3:12:18294:"
        );

        // test with all options
        // chage -d 2030-03-02 -E 1979-11-24 -I 10 -m 11 -M 12 --warndays 4 myuser7
        run_command(&[
            "-d",
            "2030-03-02",
            "-E",
            "1979-11-24",
            "-I",
            "10",
            "-m",
            "11",
            "-M",
            "12",
            "--warndays",
            "4",
            "myuser7",
        ]);
        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:21975:11:12:4:10:3614:"
        );
        // test interactive test

        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser7")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser7\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [11]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [12]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2030-03-02]: ",
        );
        stdin.write_all(b"2005-07-26\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [4]: ");
        stdin.write_all(b"9\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [10]: ");
        stdin.write_all(b"35\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [1979-11-24]: ",
        );
        stdin.write_all(b"2012-07-27\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();

        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:12990:13:14:9:35:15548:"
        );
        // all default

        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser7")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser7\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [13]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [14]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-26]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [9]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [35]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [2012-07-27]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();

        assert_eq!(
            get_entry("myuser7", SHADOW_FILE),
            "myuser7:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:12990:13:14:9:35:15548:"
        );
        assert_eq!(
            run_and_get_stdout(&["-l", "myuser7"]),
            fs::read_to_string(format!("{}/chage7b", data_path)).unwrap()
        );
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

fn expect_error_chage_field(status: &ExitStatus, msg: &str) {
    assert!(!status.success());
    assert_eq!(status.code().unwrap(), 1);
    assert_eq!("chage: error changing fields\n", msg);
}

// #[test]
fn test_02() {
    save();
    let data_path: &str = "tests/fixtures/chage/02data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();

        expect_bytes(&mut stdout, b"Changing the aging information for myuser\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"-2\n").unwrap();
        stdin.flush().unwrap();
        let exit_status = child.wait().unwrap();
        let mut buf = String::new();
        stderr.read_to_string(&mut buf).unwrap();
        expect_error_chage_field(&exit_status, &buf);

        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"foo\n").unwrap();
        stdin.flush().unwrap();
        let exit_status = child.wait().unwrap();
        let mut buf = String::new();
        stderr.read_to_string(&mut buf).unwrap();
        expect_error_chage_field(&exit_status, &buf);

        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"11\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"-2\n").unwrap();
        let exit_status = child.wait().unwrap();
        let mut buf = String::new();
        stderr.read_to_string(&mut buf).unwrap();
        expect_error_chage_field(&exit_status, &buf);

        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"foo\n").unwrap();
        let exit_status = child.wait().unwrap();
        let mut buf = String::new();
        stderr.read_to_string(&mut buf).unwrap();
        expect_error_chage_field(&exit_status, &buf);
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_invalid_date() {
    save();
    let data_path: &str = "tests/fixtures/chage/02data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -d 2011-09 myuser
        run_and_compare(&["-d", "2011-09", "myuser"]);
        // chage -d 2011-09-09-11 myuser
        run_and_compare(&["-d", "2011-09-09-11", "myuser"]);
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_invalid_numeric_arg() {
    save();
    let data_path: &str = "tests/fixtures/chage/02data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -I -12 myuser
        run_and_compare(&["-I", "-12", "myuser"]);
        // chage -m -12 myuser
        run_and_compare(&["-m", "-12", "myuser"]);
        // chage -M -12 myuser
        run_and_compare(&["-M", "-12", "myuser"]);
        // chage -W -12 myuser
        run_and_compare(&["-W", "-12", "myuser"]);
        // chage -I a myuser
        run_and_compare(&["-I", "a", "myuser"]);
        // chage -m 12.5 myuser
        run_and_compare(&["-m", "12.5", "myuser"]);
        // chage -M 12a myuser
        run_and_compare(&["-M", "12a", "myuser"]);
        // chage -W a12 myuser
        run_and_compare(&["-W", "a12", "myuser"]);
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_list() {
    save();
    let data_path: &str = "tests/fixtures/chage/03data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -l myuser1
        run_and_compare(&["-l", "myuser1"]);
        // chage -l myuser2
        run_and_compare(&["-l", "myuser2"]);
        // chage -l myuser3
        run_and_compare(&["-l", "myuser3"]);
        // chage -l myuser4
        run_and_compare(&["-l", "myuser4"]);
        // chage -l myuser5
        run_and_compare(&["-l", "myuser5"]);
        // chage -l myuser6
        run_and_compare(&["-l", "myuser6"]);
        // chage -l myuser7
        run_and_compare(&["-l", "myuser7"]);
        // chage -l myuser8
        run_and_compare(&["-l", "myuser8"]);
        // chage -l myuser9
        run_and_compare(&["-l", "myuser9"]);
        // chage -l myuser10
        run_and_compare(&["-l", "myuser10"]);
        // chage -l myuser11
        run_and_compare(&["-l", "myuser11"]);
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_invalid_user() {
    save();
    let data_path: &str = "tests/fixtures/chage/03data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -l myuser12 (invalid user)
        run_and_compare(&["-l", "myuser12"]);
        // chage -I 12 foo
        run_and_compare(&["-I", "12", "foo"]);
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_lock() {
    save();
    let data_path: &str = "tests/fixtures/chage/03data";
    for file in ["passwd", "shadow", "group", "gshadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        sudo_create_file(SHADOW_LOCK);
        let cmd_result = run_and_get_result(&["myuser1"]);
        assert_eq!(cmd_result.code(), 1);
        assert_eq!(cmd_result.stdout(), b"");
        assert_eq!(
            cmd_result.stderr(),
            b"chage: cannot lock /etc/shadow; try again later.\n"
        );
        sudo_remove_file(SHADOW_LOCK);

        sudo_create_file(PASSWD_LOCK);
        let cmd_result = run_and_get_result(&["myuser1"]);
        assert_eq!(cmd_result.code(), 1);
        assert_eq!(cmd_result.stdout(), b"");
        assert_eq!(
            cmd_result.stderr(),
            b"chage: cannot lock /etc/passwd; try again later.\n"
        );
        sudo_remove_file(PASSWD_LOCK);
    });

    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_no_shadow_i() {
    save();
    let data_path: &str = "tests/fixtures/chage/04data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -I 12 myuser2
        run_and_compare(&["-I", "12", "myuser2"]);
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_no_shadow_m() {
    save();
    let data_path: &str = "tests/fixtures/chage/04data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -m 12 myuser2
        run_and_compare(&["-m", "12", "myuser2"]);
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_no_shadow_um() {
    save();
    let data_path: &str = "tests/fixtures/chage/04data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -M 12 myuser2
        run_and_compare(&["-M", "12", "myuser2"]);
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_no_shadow_d() {
    save();
    let data_path: &str = "tests/fixtures/chage/04data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -d 2021-09-12 myuser2
        run_and_compare(&["-d", "2021-09-12", "myuser2"]);
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_no_shadow_w() {
    save();
    let data_path: &str = "tests/fixtures/chage/04data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -W 12 myuser2
        run_and_compare(&["-W", "12", "myuser2"]);
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_no_shadow_e() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        // chage -E 2021-09-12 myuser2
        run_and_compare(&["-E", "2021-09-12", "myuser2"]);
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"2005-07-26\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"9\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"35\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"2012-07-27\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();

        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:12990:13:14:9:35:15548:"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_1() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"0\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"9\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"35\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"0\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();

        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:0:13:14:9:35:0:"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_2() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"-1\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"9\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"35\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"-1\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();

        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.::13:14:9:35::"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_epoch() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"1970-01-01\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"9\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"35\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"1970-01-01\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();

        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:0:13:14:9:35:0:"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_pre_epoch() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"1900-01-01\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_epoch2() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"1970-01-01\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"9\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"35\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"1900-01-01\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
        child.wait().unwrap();
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_invalid_date() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"2000-13-42\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_invalid_date2() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"2000-mm-42\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_w_invalid() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"0\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"9a\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
        child.wait().unwrap();
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_w_invalid_2() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"13\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"14\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"0\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"-2\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
        child.wait().unwrap();
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_w_1() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"-1\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();
        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:12991:0:99999::::"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_i_invalid_1() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"9a\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
        child.wait().unwrap();
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_i_invalid_2() {
    save();
    let data_path: &str = "tests/fixtures/chage/05data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [-1]: ");
        stdin.write_all(b"-2\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stderr, b"chage: error changing fields");
        child.wait().unwrap();
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_i_1() {
    save();
    let data_path: &str = "tests/fixtures/chage/06data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [3]: ");
        stdin.write_all(b"-1\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();
        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.:12991:0:99999:7:::"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_interactive_d_1() {
    save();
    let data_path: &str = "tests/fixtures/chage/06data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        let mut child = TestScenario::new(util_name!())
            .cmd("sudo")
            .args(&[
                "-E",
                "--non-interactive",
                format!("{}/{}", *CURRENT_DIR, EASYBOX_PATH).as_str(),
                "chage",
            ])
            .arg("myuser1")
            .set_stdin(Stdio::piped())
            .set_stdout(Stdio::piped())
            .run_no_wait();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        expect_bytes(&mut stdout, b"Changing the aging information for myuser1\n");
        expect_bytes(
            &mut stdout,
            b"Enter the new value, or press ENTER for the default\n\n",
        );
        expect_bytes(&mut stdout, b"\tMinimum Password Age [0]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tMaximum Password Age [99999]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tLast Password Change (YYYY-MM-DD) [2005-07-27]: ",
        );
        stdin.write_all(b"-1\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Expiration Warning [7]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(&mut stdout, b"\tPassword Inactive [3]: ");
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        expect_bytes(
            &mut stdout,
            b"\tAccount Expiration Date (YYYY-MM-DD) [-1]: ",
        );
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
        child.wait().unwrap();
        assert_eq!(
            get_entry("myuser1", SHADOW_FILE),
            "myuser1:$1$yQnIAZWV$gDAMB2IkqaONgrQiRdo4y.::0:99999:7:3::"
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// #[test]
fn test_user() {
    save();

    let data_path: &str = "tests/fixtures/chage/06data";
    for file in ["passwd", "group", "gshadow", "shadow"].iter() {
        append_file(format!("{}/{}", data_path, file), format!("/etc/{}", file));
    }
    let result = panic::catch_unwind(|| {
        run_scripts_and_compare(
            format!("{}/{}", SCRIPTS_PATH, "test_user_r.sh"),
            format!("{}/{}", SCRIPTS_PATH, "test_user_c.sh"),
        );
    });
    restore();
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

/// By default, cargo test runs tests concurrently using multiple threads,
/// which can cause the chage tests to produce unexpected results. Therefore,
/// all tests are called within a single test function here to ensure they
/// run sequentially.
#[test]
fn all_test() {
    test_01();
    test_02();
    test_chage();
    test_interactive();
    test_interactive_1();
    test_interactive_2();
    test_interactive_epoch();
    test_interactive_pre_epoch();
    test_interactive_epoch2();
    test_interactive_invalid_date();
    test_interactive_invalid_date2();
    test_interactive_w_invalid();
    test_interactive_w_invalid_2();
    test_interactive_w_1();
    test_interactive_i_invalid_1();
    test_interactive_i_invalid_2();
    test_interactive_i_1();
    test_interactive_d_1();
    test_no_shadow_d();
    test_no_shadow_w();
    test_no_shadow_e();
    test_no_shadow_i();
    test_no_shadow_m();
    test_no_shadow_um();
    test_no_shadow_d();
    test_no_shadow_w();
    test_no_shadow_e();
    test_invalid_date();
    test_invalid_numeric_arg();
    test_list();
    test_invalid_user();
    test_lock();
    test_user();
}
