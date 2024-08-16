// This file is part of the uutils coreutils package.
//
// (c) Zhihua Zhao <YuukaC@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use std::io::Read;

use crate::common::util::*;

const C_FILE_PATH: &str = "/usr/bin/file";

#[test]
fn test_json() {
    let task = TestScenario::new(util_name!());
    let args = ["json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_cmd() {
    let task = TestScenario::new(util_name!());
    let args = ["cmd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_pgp() {
    let task = TestScenario::new(util_name!());
    let args = ["pgp.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_zstd() {
    let task = TestScenario::new(util_name!());
    let args = ["zstd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_brief() {
    let task = TestScenario::new(util_name!());
    let args = ["-b", "rpm.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_exclude() {
    let task = TestScenario::new(util_name!());
    let args = ["-e", "json", "json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    assert!(!task
        .ucmd()
        .args(&["-e", "???", "json.file"])
        .run()
        .succeeded());
}

#[test]
fn test_arg_exclude_quiet() {
    let task = TestScenario::new(util_name!());

    let cres = task.cmd(C_FILE_PATH).args(&["json.file"]).succeeds();
    let ures = task
        .ucmd()
        .args(&["--exclude-quiet", "???", "json.file"])
        .run();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_files_from() {
    let task = TestScenario::new(util_name!());
    let args = ["-f", "files.txt"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_print0() {
    let task = TestScenario::new(util_name!());
    let args = ["-0", "rpm.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    let args = ["-0", "-0", "rpm.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_uncompress_and_no_sandbox() {
    let task = TestScenario::new(util_name!());
    let args = ["-z", "-S", "rpm.tar.gz"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_uncompress_noreport_and_no_sandbox() {
    let task = TestScenario::new(util_name!());
    let args = ["-Z", "-S", "rpm.tar.gz"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_mime() {
    let task = TestScenario::new(util_name!());
    let args = ["-i", "json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_apple() {
    let task = TestScenario::new(util_name!());
    let args = ["--apple", "json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_mime_type() {
    let task = TestScenario::new(util_name!());
    let args = ["--mime-type", "json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_mime_encoding() {
    let task = TestScenario::new(util_name!());
    let args = ["--mime-encoding", "json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_magic_file() {
    let task = TestScenario::new(util_name!());
    let args = ["-m", "rpm.magic", "rpm.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    let ures = task.ucmd().args(&["rpm.file"]).succeeds();
    assert!(ures.succeeded());
    assert_ne!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_no_pad() {
    let task = TestScenario::new(util_name!());
    let args = [
        "-N",
        "cmd.file",
        "json.file",
        "pgp.file",
        "pgp.sl",
        "rpm.file",
        "rpm.magic",
        "rpm.tar.gz",
        "zstd.file",
    ];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_keep_going() {
    let task = TestScenario::new(util_name!());
    let args = ["-k", "json.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_checking_printout() {
    let task = TestScenario::new(util_name!());
    let args = ["-m", "rpm.magic", "-c"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_debug() {
    let task = TestScenario::new(util_name!());
    let args = ["-d", "cmd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();

    assert_eq!(cres.stdout_str(), ures.stdout_str());
    assert_eq!(cres.stderr_str().len(), ures.stderr_str().len()); // The debug info will not be completely same
}

#[test]
fn test_arg_list() {
    let task = TestScenario::new(util_name!());
    let args = ["-l"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[ignore]
#[test]
fn test_arg_preserve_date() {
    let task = TestScenario::new(util_name!());
    let args = ["json.file"];

    let original = task
        .cmd("stat")
        .arg("json.file")
        .succeeds()
        .stdout_str()
        .to_owned();

    std::thread::sleep(std::time::Duration::from_millis(200));

    task.cmd(C_FILE_PATH).args(&args).succeeds();
    let cres = task
        .cmd("stat")
        .arg("json.file")
        .succeeds()
        .stdout_str()
        .to_owned();

    std::thread::sleep(std::time::Duration::from_millis(200));

    task.ucmd().args(&args).succeeds();
    let ures = task
        .cmd("stat")
        .arg("json.file")
        .succeeds()
        .stdout_str()
        .to_owned();

    assert_eq!(original, cres);
    assert_eq!(original, ures);
}

#[test]
fn test_arg_dereference() {
    let task = TestScenario::new(util_name!());
    let args = ["-L", "pgp.sl"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_no_dereference() {
    let task = TestScenario::new(util_name!());
    let args = ["-h", "pgp.sl"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_extension() {
    let task = TestScenario::new(util_name!());
    let args = ["--extension", "zstd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_compile() {
    let task = TestScenario::new(util_name!());
    let args = ["-m", "rpm.magic", "-C"];

    let mut mgc_path = task.cmd("pwd").succeeds().stdout_str().to_owned();
    mgc_path.pop();
    mgc_path += "/rpm.magic.mgc";

    task.cmd(C_FILE_PATH).args(&args).succeeds();
    let mut cres = Vec::new();
    std::fs::File::open(&mgc_path)
        .unwrap()
        .read_to_end(&mut cres)
        .unwrap();

    task.cmd("rm").arg(&mgc_path).succeeds();

    task.ucmd().args(&args).succeeds();
    let mut ures = Vec::new();
    std::fs::File::open(&mgc_path)
        .unwrap()
        .read_to_end(&mut ures)
        .unwrap();

    assert_eq!(cres, ures);
}

#[test]
fn test_arg_separator() {
    let task = TestScenario::new(util_name!());
    let args = ["-F", "YuukaC", "pgp.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_no_buffer() {
    let task = TestScenario::new(util_name!());
    let args = [
        "-n",
        "cmd.file",
        "json.file",
        "pgp.file",
        "pgp.sl",
        "rpm.file",
        "rpm.magic",
        "rpm.tar.gz",
        "zstd.file",
    ];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_raw() {
    let task = TestScenario::new(util_name!());

    let ures1 = task.ucmd().args(&["\u{2000}"]).succeeds();
    let ures2 = task.ucmd().args(&["-r", "\u{2000}"]).succeeds();
    assert_ne!(ures1.stdout_str(), ures2.stdout_str());
    ures1.stdout_contains("\\");
    ures2.stdout_does_not_contain("\\");
}

#[test]
fn test_arg_parameter() {
    let task = TestScenario::new(util_name!());
    let args = ["-P", "bytes=0", "cmd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    let args = ["-P", "bytes=1", "cmd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    let args = ["-P", "bytes=10", "cmd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    let args = ["-P", "bytes=100", "cmd.file"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}

#[test]
fn test_arg_special_files() {
    let task = TestScenario::new(util_name!());
    let args = ["/dev/null"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());

    let args = ["-s", "/dev/null"];

    let cres = task.cmd(C_FILE_PATH).args(&args).succeeds();
    assert_ne!(cres.stdout_str(), ures.stdout_str());
    let ures = task.ucmd().args(&args).succeeds();
    assert_eq!(cres.stdout_str(), ures.stdout_str());
}
