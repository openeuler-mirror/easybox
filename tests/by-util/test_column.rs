//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use crate::common::util::*;
use std::{env, path::PathBuf};

const C_COLUMN_PATH: &str = "/usr/bin/column";

fn get_test_file_path(relative_path: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative_path);
    path.to_str().unwrap().to_string()
}

#[test]
fn test_column_fill_cols_80() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["-c", "80", &input_file_path])
        .run();

    let actual_result = ts.ucmd().args(&["-c", "80", &input_file_path]).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_cols_50() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["-c", "50", &input_file_path])
        .run();

    let actual_result = ts.ucmd().args(&["-c", "50", &input_file_path]).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_cols_250() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["-c", "250", &input_file_path])
        .run();

    let actual_result = ts.ucmd().args(&["-c", "250", &input_file_path]).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_rows_80() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--fillrows", "-c", "80", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--fillrows", "-c", "80", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_rows_50() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--fillrows", "-c", "50", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--fillrows", "-c", "50", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_rows_250() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--fillrows", "-c", "250", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--fillrows", "-c", "250", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_invalid_multibyte() {
    let ts = TestScenario::new(util_name!());
    // The byte sequence "\x94\x7e\n" corresponds to the bytes 0x94, 0x7e, and a newline character.
    let special_input = vec![0x94, 0x7e, 0x0a];

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .pipe_in(special_input.clone())
        .run();

    // Actual result using the tested command
    let actual_result = ts.ucmd().pipe_in(special_input.clone()).run();

    assert_eq!(expect_result.stderr_str(), actual_result.stderr_str());
}

#[test]
fn test_column_multiple_files_input() {
    let ts = TestScenario::new(util_name!());
    let file1 = get_test_file_path("tests/fixtures/column/fivecols");
    let file2 = get_test_file_path("tests/fixtures/column/fivecols");
    let file3 = get_test_file_path("tests/fixtures/column/fivecols");

    // Expect result using the actual column command
    let expect_result = ts
        .cmd_keepenv("/usr/bin/column")
        .args(&["-x", "-c", "50", &file1, &file2, &file3])
        .run();

    // Actual result using the tested command
    let actual_result = ts
        .ucmd()
        .args(&["-x", "-c", "50", &file1, &file2, &file3])
        .run();

    assert_eq!(expect_result.stderr_str(), actual_result.stderr_str());
}

#[test]
fn test_column_default() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", &input_file_path])
        .run();

    let actual_result = ts.ucmd().args(&["--table", &input_file_path]).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_output_separator() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", "--output-separator", "|", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--output-separator", "|", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_input_separator() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-sep");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", "--separator", ",", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--separator", ",", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_input_separator_space() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-sep-space");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", "--separator", "\t", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--separator", "\t", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_lines() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-empty-lines");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", "--keep-empty-lines", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--keep-empty-lines", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_noempty_lines() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-empty-lines");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", &input_file_path])
        .run();

    let actual_result = ts.ucmd().args(&["--table", &input_file_path]).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_long() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", &input_file_path])
        .run();

    let actual_result = ts.ucmd().args(&["--table", &input_file_path]).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_hide() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&["--table", "--table-hide", "1,2,3,4,7,8", &input_file_path])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--table-hide", "1,2,3,4,7,8", &input_file_path])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_headers() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT",
            &input_file_path,
        ])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT",
            &input_file_path,
        ])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_truncate() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT",
            "--table-truncate",
            "VFS-OPTS,FS-OPTS",
            "--output-width",
            "80",
            &input_file_path,
        ])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT",
            "--table-truncate",
            "VFS-OPTS,FS-OPTS",
            "--output-width",
            "80",
            &input_file_path,
        ])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_right() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT,VFS-OPTS,FS-OPTS,PROP",
            "--table-right",
            "SOURCE,TYPE",
            "--output-width",
            "80",
            &input_file_path,
        ])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT,VFS-OPTS,FS-OPTS,PROP",
            "--table-right",
            "SOURCE,TYPE",
            "--output-width",
            "80",
            &input_file_path,
        ])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_wrap() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT,VFS-OPTS,PROP",
            "--table-wrap",
            "FS-OPTS",
            "--output-width",
            "110",
            &input_file_path,
        ])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT,VFS-OPTS,PROP",
            "--table-wrap",
            "FS-OPTS",
            "--output-width",
            "110",
            &input_file_path,
        ])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_order() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT,PROP,FS-OPTS,MAJMIN",
            "-table-order",
            "TARGET,SOURCE,TYPE,VFS-OPTS",
            "--output-width",
            "110",
            &input_file_path,
        ])
        .run();

    let actual_result = ts
        .ucmd()
        .args(&[
            "--table",
            "--table-columns",
            "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
            "--table-hide",
            "SEP,ID,PARENT,ROOT,PROP,FS-OPTS,MAJMIN",
            "-table-order",
            "TARGET,SOURCE,TYPE,VFS-OPTS",
            "--output-width",
            "110",
            &input_file_path,
        ])
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_column() {
    let ts = TestScenario::new(util_name!());

    let expect_result = ts
        .cmd_keepenv("/usr/bin/column")
        .args(&["--table", "--separator", ":", "--output-separator", ":"])
        .pipe_in(":a:b\n")
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--separator", ":", "--output-separator", ":"])
        .pipe_in(":a:b\n")
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_column_at_eol() {
    let ts = TestScenario::new(util_name!());

    let expect_result = ts
        .cmd_keepenv("/usr/bin/column")
        .args(&["--table", "--separator", "|", "--output-separator", "|"])
        .pipe_in("|")
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--separator", "|", "--output-separator", "|"])
        .pipe_in("|")
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_column_at_eol2() {
    let ts = TestScenario::new(util_name!());

    let expect_result = ts
        .cmd_keepenv("/usr/bin/column")
        .args(&["--table", "--separator", "|", "--output-separator", "|"])
        .pipe_in("||")
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["--table", "--separator", "|", "--output-separator", "|"])
        .pipe_in("||")
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}
