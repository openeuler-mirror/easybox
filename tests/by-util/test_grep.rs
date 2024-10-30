//! This file is part of the easybox package.
//
// (c) SodaGreeny574 <1968629133@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use pretty_assertions::assert_eq;
use std::fs;
use std::fs::File;
use std::io::Write;

const UTIL: &str = "grep";
const C_GREP_PATH: &str = "/usr/bin/grep";

pub fn run_and_compare(ts: &TestScenario, args: &[&str]) {
    let actual_result = ts.ucmd().args(args).run();
    let expect_result = ts.cmd_keepenv(C_GREP_PATH).args(args).run();

    if expect_result.stdout() != actual_result.stdout() {
        eprintln!(
            "--- Expected stdout ---\n{}",
            String::from_utf8_lossy(expect_result.stdout())
        );
        eprintln!(
            "--- Actual stdout ---\n{}",
            String::from_utf8_lossy(actual_result.stdout())
        );
    }

    assert_eq!(
        expect_result.stdout(),
        actual_result.stdout(),
        "Stdout does not match"
    );

    if expect_result.code() != actual_result.code() {
        eprintln!("--- Expected exit code ---\n{}", expect_result.code());
        eprintln!("--- Actual exit code ---\n{}", actual_result.code());
    }

    assert_eq!(
        expect_result.code(),
        actual_result.code(),
        "Exit code does not match"
    );
}

fn setup_test_file(file_name: &str, content: &str) -> String {
    let mut file = File::create(file_name).expect("Failed to create test file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to test file");
    file.flush().expect("Failed to flush test file");
    file_name.to_string()
}

#[test]
fn test_extended_regexp_upper_e() {
    let file_name = "extended_regexp_E.txt";
    let content = "apple\nbanana\ncherry\napricot\nblueberry\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-E", "^[ab].*", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_fixed_strings_upper_f() {
    let file_name = "fixed_strings_F.txt";
    let content = "foo\nbar\nfoo.bar\nfoobar\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-F", "foo.bar", file_name]);
    run_and_compare(&ts, &["-F", "foo.bar", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_basic_regexp_upper_g() {
    let file_name = "basic_regexp_G.txt";
    let content = "cat\nbat\nhat\nrat\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-G", "^[ch]at$", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_perl_regexp_upper_p() {
    let file_name = "perl_regexp_P.txt";
    let content = "foo123\nbar456\nbaz789\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-P", "\\d+", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_combined_options_upper_e_and_n() {
    let file_name = "combined_options_E_and_n.txt";
    let content = "apple\nbanana\ncherry\napricot\nblueberry\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-E", "^[ab].*", "-n", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_i() {
    let file_name = "test_option_i.txt";
    let content = "Hello World\nhello world\nHELLO WORLD\nHeLLo WoRLd\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-i", "hello", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_no_ignore_case() {
    let file_name = "test_option_no_ignore_case.txt";
    let content = "Hello World\nhello world\nHELLO WORLD\nHeLLo WoRLd\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["--no-ignore-case", "hello", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_w() {
    let file_name = "test_option_w.txt";
    let content = "word\nsword\nwording\na word a day\nword.\nword!\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);

    run_and_compare(&ts, &["-w", "word", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_x() {
    let file_name = "test_option_x.txt";
    let content = "exactline\nexactline with extra\n another exactline\nexactline\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-x", "exactline", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_z() {
    let file_name = "test_option_z.txt";
    let content = "line1\x00line2\x00pattern\x00line3\x00";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-z", "pattern", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_s() {
    let non_existent_file = "nonexistent_option_s.txt";
    let ts = TestScenario::new(UTIL);
    let actual_result = ts.ucmd().args(&["-s", "pattern", non_existent_file]).run();
    let expect_result = ts
        .cmd_keepenv(C_GREP_PATH)
        .args(&["-s", "pattern", non_existent_file])
        .run();

    assert_eq!(
        expect_result.stdout(),
        actual_result.stdout(),
        "Stdout does not match for -s option"
    );

    assert_eq!(
        expect_result.code(),
        actual_result.code(),
        "Exit code does not match for -s option"
    );

    let _ = fs::remove_file(non_existent_file);
}

#[test]
fn test_combined_i_v() {
    let file_name = "test_combined_i_v.txt";
    let content = "Apple pie\nBanana smoothie\napple cider\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-i", "-v", "apple", file_name]);
    fs::remove_file(file_name).expect("Failed to clean up test file");
}

#[test]
fn test_option_i_and_w_combined() {
    let file_name = "test_option_i_w.txt";
    let content = "Word\nword\nsword\nWording\na Word a day\nword.\nWORD!\n";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-i", "-w", "word", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_z_with_no_match() {
    let file_name = "test_option_z_no_match.txt";
    let content = "line1\x00line2\x00line3\x00";
    setup_test_file(file_name, content.as_ref());

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-z", "pattern", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_v() {
    let file_name = "test_option_v.txt";
    let content = "apple\nbanana\ncherry\napricot\nblueberry\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-v", "banana", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_m() {
    let file_name = "test_option_m.txt";
    let content = "match1\nmatch2\nmatch3\nmatch4\nmatch5\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-m", "3", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_b() {
    let file_name = "test_option_b.txt";
    let content = "match1\nnomatch\nmatch2\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-b", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_n() {
    let file_name = "test_option_n.txt";
    let content = "line1\nline2\nmatch\nline4\nmatch\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-n", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_line_buffered() {
    let file_name = "test_option_line_buffered.txt";
    let content = "buffered line 1\nbuffered line 2\nmatch\nbuffered line 4\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["--line-buffered", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_h() {
    let file_name1 = "test_option_H_file1.txt";
    let file_name2 = "test_option_H_file2.txt";
    let content1 = "match in file1\n";
    let content2 = "match in file2\n";
    setup_test_file(file_name1, content1);
    setup_test_file(file_name2, content2);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-H", "match", file_name1, file_name2]);
    let _ = fs::remove_file(file_name1);
    let _ = fs::remove_file(file_name2);
}

#[test]
fn test_option_h() {
    let file_name1 = "test_option_h_file1.txt";
    let file_name2 = "test_option_h_file2.txt";
    let content1 = "match in file1\n";
    let content2 = "match in file2\n";
    setup_test_file(file_name1, content1);
    setup_test_file(file_name2, content2);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-h", "match", file_name1, file_name2]);
    let _ = fs::remove_file(file_name1);
    let _ = fs::remove_file(file_name2);
}

#[test]
fn test_option_label() {
    let file_name = "test_option_label.txt";
    let content = "pattern match here\nanother pattern match\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["--label=LABEL", "pattern", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_o() {
    let file_name = "test_option_o.txt";
    let content = "apple orange banana apple orange";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-o", "apple", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_q() {
    let file_name = "test_option_q.txt";
    let content = "match this line\nno match here\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-q", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_binary_files_without_match() {
    let file_name = "test_option_binary_files_without_match.bin";
    let content = [0x00, 0xFF, 0xA5, 0x33];
    let mut file = File::create(file_name).expect("Failed to create test file");
    file.write_all(&content)
        .expect("Failed to write to test file");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["--binary-files=without-match", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_a() {
    let file_name = "test_option_a.bin";
    let content = [0x61, 0x62, 0x63, 0x0A, 0x64, 0x65];
    let mut file = File::create(file_name).expect("Failed to create test file");
    file.write_all(&content)
        .expect("Failed to write to test file");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-a", "b", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_d_skip() {
    let dir_name = "test_option_d_skip_dir";
    if std::path::Path::new(dir_name).exists() {
        fs::remove_dir_all(dir_name).expect("Failed to remove old test directory");
    }
    fs::create_dir_all(dir_name).expect("Failed to create test directory");
    let file_name = format!("{}/file.txt", dir_name);
    setup_test_file(&file_name, "this is a match\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-d", "skip", "match", dir_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_d_read() {
    let fifo_name = "test_fifo";
    let _ = fs::remove_file(fifo_name);
    mkfifo(fifo_name, Mode::S_IRUSR | Mode::S_IWUSR).expect("Failed to create FIFO file");
    std::thread::spawn(move || {
        let mut fifo_file = File::create(fifo_name).expect("Failed to open FIFO file");
        fifo_file
            .write_all(b"this is a match\n")
            .expect("Failed to write to FIFO file");
    });

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-D", "read", "match", fifo_name]);
}

#[test]
fn test_option_r() {
    let dir_name = "test_option_r_dir";
    fs::create_dir_all(dir_name).expect("Failed to create test directory");
    let file1 = format!("{}/file1.txt", dir_name);
    let file2 = format!("{}/file2.txt", dir_name);
    setup_test_file(&file1, "this is a match\n");
    setup_test_file(&file2, "no match here\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-r", "match", dir_name]);
}

#[test]
fn test_option_upper_r() {
    let dir_name = "test_option_R_dir";
    if std::path::Path::new(dir_name).exists() {
        fs::remove_dir_all(dir_name).expect("Failed to remove old test directory");
    }
    fs::create_dir_all(dir_name).expect("Failed to create test directory");

    let file_name = format!("{}/file.txt", dir_name);
    setup_test_file(&file_name, "this is a match\n");
    let link_name = format!("{}/link_to_file.txt", dir_name);
    if std::path::Path::new(&link_name).exists() {
        fs::remove_file(&link_name).expect("Failed to remove old symbolic link");
    }
    std::os::unix::fs::symlink(&file_name, &link_name).expect("Failed to create symbolic link");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-R", "match", dir_name]);
    let _ = fs::remove_dir_all(dir_name);
}

#[test]
fn test_option_include_glob() {
    let dir_name = "test_option_include_dir";
    fs::create_dir_all(dir_name).expect("Failed to create test directory");
    let file1 = format!("{}/file1.txt", dir_name);
    let file2 = format!("{}/file2.log", dir_name);
    setup_test_file(&file1, "this is a match\n");
    setup_test_file(&file2, "this is a match too\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-r", "--include=*.txt", "match", dir_name]);
}

#[test]
fn test_option_exclude_glob() {
    let dir_name = "test_option_exclude_dir";
    fs::create_dir_all(dir_name).expect("Failed to create test directory");
    let file1 = format!("{}/file1.txt", dir_name);
    let file2 = format!("{}/file2.log", dir_name);
    setup_test_file(&file1, "this is a match\n");
    setup_test_file(&file2, "this is a match too\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-r", "--exclude=*.log", "match", dir_name]);
}

#[test]
fn test_option_exclude_dir() {
    let dir_name = "test_option_exclude_dir_main";
    let sub_dir_name = format!("{}/exclude_subdir", dir_name);
    fs::create_dir_all(&sub_dir_name).expect("Failed to create test directory");
    let file1 = format!("{}/file1.txt", dir_name);
    let file2 = format!("{}/exclude_subdir/file2.txt", dir_name);
    setup_test_file(&file1, "this is a match\n");
    setup_test_file(&file2, "this should be excluded\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(
        &ts,
        &["-r", "--exclude-dir=exclude_subdir", "match", dir_name],
    );
}

#[test]
fn test_option_upper_l() {
    let file_name1 = "test_option_L1.txt";
    let file_name2 = "test_option_L2.txt";
    setup_test_file(file_name1, "this is a match\n");
    setup_test_file(file_name2, "no match here\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-L", "match", file_name1, file_name2]);
    let _ = fs::remove_file(file_name1);
    let _ = fs::remove_file(file_name2);
}

#[test]
fn test_option_l() {
    let file_name = "test_option_l.txt";
    setup_test_file(file_name, "this is a match\nno match here\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-l", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_c() {
    let file_name = "test_option_c.txt";
    setup_test_file(
        file_name,
        "match this line\nmatch that line\nno match here\n",
    );

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-c", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_t() {
    let file_name = "test_option_T.txt";
    setup_test_file(file_name, "match this line\nanother match\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-T", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_z() {
    let file_name = "test_option_Z.txt";
    setup_test_file(file_name, "match this line\nanother match\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-Z", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_b() {
    let file_name = "test_option_B.txt";
    setup_test_file(file_name, "line1\nline2\nmatch\nline4\nline5\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-B", "2", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_a() {
    let file_name = "test_option_A.txt";
    setup_test_file(file_name, "line1\nline2\nmatch\nline4\nline5\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-A", "2", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_c() {
    let file_name = "test_option_C.txt";
    setup_test_file(file_name, "line1\nline2\nmatch\nline4\nline5\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-C", "2", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_color() {
    let file_name = "test_option_color.txt";
    setup_test_file(file_name, "this is a match\nno match here\n");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["--color=always", "match", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_i() {
    let file_name = "test_option_I.bin";
    let content = [0x00, 0x61, 0x62, 0x63, 0x0A, 0x64, 0x65, 0x66, 0x67];
    let mut file = File::create(file_name).expect("Failed to create test file");
    file.write_all(&content)
        .expect("Failed to write to test file");

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-I", "abc", file_name]);
    let _ = fs::remove_file(file_name);
}

#[test]
fn test_option_upper_u() {
    let file_name = "test_option_U.txt";
    let content = "match this line\r\nand this line\r\nno match here\r\n";
    setup_test_file(file_name, content);

    let ts = TestScenario::new(UTIL);
    run_and_compare(&ts, &["-U", "match", file_name]);
    let _ = fs::remove_file(file_name);
}
