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
    let args = vec!["-c", "80", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_cols_50() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");
    let args = vec!["-c", "50", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_cols_250() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");
    let args = vec!["-c", "250", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_rows_80() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");
    let args = vec!["--fillrows", "-c", "80", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_rows_50() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");
    let args = vec!["--fillrows", "-c", "50", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_fill_rows_250() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/onecolumn");
    let args = vec!["--fillrows", "-c", "250", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

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

    let actual_result = ts.ucmd_keepenv().pipe_in(special_input.clone()).run();

    assert_eq!(expect_result.stderr_str(), actual_result.stderr_str());
}

#[test]
fn test_column_multiple_files_input() {
    let ts = TestScenario::new(util_name!());
    let file1 = get_test_file_path("tests/fixtures/column/fivecols");
    let file2 = get_test_file_path("tests/fixtures/column/fivecols");
    let file3 = get_test_file_path("tests/fixtures/column/fivecols");
    let args = vec!["-x", "-c", "50", &file1, &file2, &file3];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stderr_str(), actual_result.stderr_str());
}

#[test]
fn test_column_default() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table");
    let args = vec!["--table", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_output_separator() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table");
    let args = vec!["--output-separator", "|", "--table", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_input_separator() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-sep");
    let args = vec!["--separator", ",", "--table", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_input_separator_space() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-sep-space");
    let args = vec!["--separator", "\t", "--table", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_lines() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-empty-lines");
    // `--table-empty-lines`` is deprecated, use `--keep-empty-lines` now.
    let args = vec!["--table", "--keep-empty-lines", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_noempty_lines() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/table-empty-lines");
    let args = vec!["--table", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_long() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec!["--table", &input_file_path];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_hide() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec!["--table", &input_file_path, "--table-hide", "1,2,3,4,7,8"];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_headers() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec![
        "--table",
        &input_file_path,
        "--table-columns",
        "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
        "--table-hide",
        "SEP,ID,PARENT,ROOT",
    ];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_truncate() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec![
        "--table",
        &input_file_path,
        "--table-columns",
        "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
        "--table-hide",
        "SEP,ID,PARENT,ROOT",
        "--table-truncate",
        "VFS-OPTS,FS-OPTS",
        "--output-width",
        "80",
    ];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_right() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec![
        "--table",
        &input_file_path,
        "--table-columns",
        "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
        "--table-hide",
        "SEP,ID,PARENT,ROOT,VFS-OPTS,FS-OPTS,PROP",
        "--table-right",
        "SOURCE,TYPE",
        "--output-width",
        "80",
    ];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_wrap() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec![
        "--table",
        &input_file_path,
        "--table-columns",
        "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
        "--table-hide=SEP,ID,PARENT,ROOT,VFS-OPTS,PROP",
        "--table-wrap",
        "FS-OPTS",
        "--output-width",
        "110",
    ];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_order() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec![
        "--table",
        &input_file_path,
        "--table-columns",
        "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
        "--table-hide=SEP,ID,PARENT,ROOT,PROP,FS-OPTS,MAJMIN",
        "--table-order",
        "TARGET,SOURCE,TYPE,VFS-OPTS",
        "--output-width",
        "110",
    ];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_tree() {
    let ts = TestScenario::new(util_name!());
    let input_file_path = get_test_file_path("tests/fixtures/column/mountinfo");
    let args = vec![
        "--table",
        &input_file_path,
        "--table-columns",
        "ID,PARENT,MAJMIN,ROOT,TARGET,VFS-OPTS,PROP,SEP,TYPE,SOURCE,FS-OPTS",
        "--table-hide=SEP,ID,PARENT,ROOT,PROP,FS-OPTS,MAJMIN",
        "--table-order",
        "TARGET,SOURCE,TYPE,VFS-OPTS",
        "--tree",
        "TARGET",
        "--tree-id",
        "ID",
        "--tree-parent",
        "PARENT",
        "--output-width",
        "110",
    ];

    let expect_result = ts.cmd_keepenv(C_COLUMN_PATH).args(&args).run();
    // Use ucmd_keepenv to get correct charset.
    let actual_result = ts.ucmd_keepenv().args(&args).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_column() {
    let ts = TestScenario::new(util_name!());
    let args = vec!["--table", "--separator", ":", "--output-separator", ":"];
    let stdin = ":a:b\n";

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&args)
        .pipe_in(stdin)
        .run();
    let actual_result = ts.ucmd_keepenv().args(&args).pipe_in(stdin).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_column_at_eol() {
    let ts = TestScenario::new(util_name!());
    let args = vec!["--separator", "|", "--output-separator", "|", "--table"];
    let stdin = "|";

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&args)
        .pipe_in(stdin)
        .run();
    let actual_result = ts.ucmd_keepenv().args(&args).pipe_in(stdin).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_column_empty_column_at_eol2() {
    let ts = TestScenario::new(util_name!());
    let args = vec!["--separator", "|", "--output-separator", "|", "--table"];
    let stdin = "||";

    let expect_result = ts
        .cmd_keepenv(C_COLUMN_PATH)
        .args(&args)
        .pipe_in(stdin)
        .run();
    let actual_result = ts.ucmd_keepenv().args(&args).pipe_in(stdin).run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}
