// This file is part of the easybox package.
//
// (c) Yuyichen2025 <vyu112@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::{CmdResult, TestScenario};
use regex::Regex;
use std::env::set_var;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const C_EXPECT_PATH: &str = "/usr/bin/expect";
const C_SCRIPT_PATH: &str = "/usr/bin/script";
const C_LESS_PATH: &str = "/usr/bin/less";
const FIXTURE_PATH: &str = "tests/fixtures/less/";
/// Run an Expect script and return the output
fn run_expect_script(script: &str) -> CmdResult {
    println!("running GNU expect to spawn less");
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    println!("task fixtures: {}", task.fixtures.as_string());

    let res = task.cmd(C_EXPECT_PATH).arg("-c").arg(script).succeeds();

    println!(
        "\x1b[33;1mStd Out:\x1b[0m {}",
        String::from_utf8_lossy(&res.stdout())
    );
    if !res.stderr().is_empty() {
        println!(
            "\x1b[31;1mStd Err:\x1b[0m {}",
            String::from_utf8_lossy(&res.stderr())
        );
    }
    res
}

/// Run a less command with Expect script and return the output
fn run_less_with_script(script: &str) -> std::io::Result<String> {
    println!("running GNU script to spawn less");
    let task = TestScenario::new(util_name!());
    set_var("HOME", task.fixtures.as_string());
    let res = task
        .cmd(C_SCRIPT_PATH)
        .arg("-q")
        .arg("-c")
        .arg(script)
        .arg("/dev/null")
        .succeeds();

    println!(
        "\n\x1b[33;1mStd Out:\x1b[0m {}",
        String::from_utf8_lossy(&res.stdout())
    );
    if !res.stderr().is_empty() {
        println!(
            "\x1b[31;1mStd Err:\x1b[0m {}",
            String::from_utf8_lossy(&res.stderr())
        );
    }

    Ok(String::from_utf8_lossy(&res.stdout()).into())
}

fn run_child_thread(args: &str, bin: &str) -> std::io::Result<String> {
    match bin {
        "GNU" => {
            println!("\n\x1b[33;1mRun GNU\x1b[0m");
            let task = TestScenario::new(util_name!());
            set_var("HOME", task.fixtures.as_string());
            let mut child = task
                .cmd(C_LESS_PATH)
                .args(&args.split_whitespace().collect::<Vec<_>>())
                .set_stdin(Stdio::piped())
                .set_stdout(Stdio::piped())
                .set_stderr(Stdio::piped())
                .run_no_wait();

            let mut stdin = child.stdin.take().unwrap();
            let mut stdout = child.stdout.take().unwrap();
            let mut stderr = child.stderr.take().unwrap();

            stdin.write_all(b"q\r")?;
            stdin.flush()?;
            drop(stdin);

            let mut output = String::new();
            stdout
                .read_to_string(&mut output)
                .expect("Failed to read stdout");
            println!("stdout: {}", output);
            let mut error_output = String::new();
            stderr
                .read_to_string(&mut error_output)
                .expect("Failed to read stderr");
            if !error_output.is_empty() {
                println!("stderr: {}", error_output);
            }

            let status = child.wait()?;
            assert!(status.success(), "Child process did not exit successfully");

            Ok(output)
        }
        "Rust" => {
            println!("\n\x1b[33;1mRun Rust\x1b[0m");
            let task = TestScenario::new(util_name!());
            set_var("HOME", task.fixtures.as_string());
            let mut child = task
                .ucmd()
                .env("CARGO_TEST", "1")
                .args(&args.split_whitespace().collect::<Vec<_>>())
                .set_stdin(Stdio::piped())
                .set_stdout(Stdio::piped())
                .set_stderr(Stdio::piped())
                .run_no_wait();

            let mut stdin = child.stdin.take().unwrap();
            let mut stdout = child.stdout.take().unwrap();
            let mut stderr = child.stderr.take().unwrap();

            stdin.write_all(b"q\r")?;
            stdin.flush()?;
            drop(stdin);

            let mut output = String::new();
            stdout
                .read_to_string(&mut output)
                .expect("Failed to read stdout");
            println!("stdout: {}", output);
            let mut error_output = String::new();
            stderr
                .read_to_string(&mut error_output)
                .expect("Failed to read stderr");
            if !error_output.is_empty() {
                println!("stderr: {}", error_output);
            }

            let status = child.wait()?;
            assert!(status.success(), "Child process did not exit successfully");

            Ok(output)
        }
        _ => {
            panic!("Unknown binary type: {}", bin);
        }
    }
}

/// Get the path to the Rust less executable and the project root directory
fn get_rust_less_path() -> (String, String) {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let executable_path = project_root
        .join("target")
        .join(if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        })
        .join("easybox less");
    (
        project_root.to_str().unwrap().to_string(),
        executable_path.to_str().unwrap().to_string(),
    )
}
/// Test behavior when invoking less without arguments
///
/// This test verifies that when less is invoked without any arguments,
/// the program correctly outputs an error message and compares this
/// behavior against GNU less.
#[test]
fn test_less_no_arg() {
    println!("\n\x1b[32;1m ======= test Rust less no arg ===\x1b[0m");

    let (_, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let rust_script = format!(
        r#"
        spawn {}
        expect {{
            "Missing filename" {{ exit 0 }}
            timeout {{ exit 1 }}
        }}
    "#,
        oe_less
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());
    assert!(
        stdout.contains("Missing filename"),
        "Unexpected stdout: {}",
        stdout
    );
    println!("\x1b[32;1m ======= test GNU less no arg ===\x1b[0m");
    let script = r#"
        spawn less
        expect {
            "Missing filename ("less --help" for help)" { exit 0 }
            timeout { exit 1 }
        }
    "#;

    let output = run_expect_script(script);

    let stdout = String::from_utf8_lossy(&output.stdout());
    assert!(
        stdout.contains("Missing filename"),
        "Unexpected stdout: {}",
        stdout
    );
    println!("\x1b[32;1m ======= test less no arg finish ===\x1b[0m");
}

/// Test behavior when passing a directory as an argument to less
///
/// This test verifies that when a directory is provided as an argument to less,
/// the program correctly identifies and reports the error, and compares this
/// behavior against GNU less.
#[test]
fn test_less_dir_arg() {
    println!("\n\x1b[32;1m === test Rust less dir arg ===\x1b[0m");

    let (_, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let rust_script = format!(
        r#"
        spawn {} .
        expect {{
            "'.' is a directory." {{ exit 0 }}
            timeout {{ exit 1 }}
        }}
    "#,
        oe_less
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());
    assert!(
        stdout.contains("is a directory"),
        "Unexpected stdout: {}",
        stdout
    );
    println!("\x1b[32;1m === test GNU less dir arg ===\x1b[0m");

    let script = r#"
        spawn less .
        expect {
            "'.' is a directory." { exit 0 }
            timeout { exit 1 }
        }
    "#;

    let output = run_expect_script(script);

    let stdout = String::from_utf8_lossy(&output.stdout());
    assert!(
        stdout.contains("is a directory"),
        "Unexpected stdout: {}",
        stdout
    );

    println!("\x1b[32;1m === test less dir arg finish ===\x1b[0m");
}

/// Test less behavior when accessing a file with insufficient permissions
///
/// This test verifies that when attempting to access a file with insufficient permissions,
/// less correctly reports a permission error and compares this behavior against GNU less.
#[test]
#[ignore]
fn test_less_invalid_file_perms() {
    use std::fs::{set_permissions, Permissions};
    use std::os::unix::fs::PermissionsExt;

    println!("\n\x1b[32;1m === test Rust less invalid file perms ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "invalid-perms.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    set_permissions(&file_path, Permissions::from_mode(0o244)).unwrap();

    Command::new(oe_less)
        .arg(file_path.display().to_string())
        .output()
        .expect_err("Permission Denied");

    println!("\x1b[32;1m === test GNU less invalid file perms ===\x1b[0m");

    let _expect_err = format!("{}: Permission denied\n", file_path.display());

    let gnu_output = Command::new(C_LESS_PATH)
        .arg(file_path.display().to_string())
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&gnu_output.stderr);
    assert!(
        stderr.contains("Permission denied"),
        "Expected 'Permission denied' in stderr, but got: {}",
        stderr
    );

    set_permissions(&file_path, Permissions::from_mode(0o644)).unwrap();
    println!("\x1b[32;1m === test less invalid file perms finish ===\x1b[0m");
}

/// Test less behavior when using the force option to open a file
///
/// This test verifies that when using the -force option, less can open files
/// or directories with insufficient permissions and compares this behavior
/// against GNU less.
#[test]
#[ignore]
fn test_less_force_open() {
    use std::fs::{set_permissions, Permissions};
    use std::os::unix::fs::PermissionsExt;

    println!("\n\x1b[32;1m === test Rust less force open ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "force-open.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    println!("File path: {}", file_path.display());

    let permissions = Permissions::from_mode(0o644);
    set_permissions(&file_path, permissions).unwrap();

    let _rust_output = Command::new(oe_less)
        .arg("-f")
        .arg(file_path.display().to_string())
        .output();

    println!("\x1b[32;1m === test GNU less force open ===\x1b[0m");

    let _gnu_output = Command::new(C_LESS_PATH)
        .arg("-f")
        .arg(file_path.display().to_string())
        .output();

    set_permissions(&file_path, Permissions::from_mode(0o644)).unwrap();

    println!("\x1b[32;1m === test less force open finish ===\x1b[0m");
}

/// Test less behavior when passing invalid arguments
///
/// This test verifies that when invalid options or argument values are passed,
/// less correctly reports errors and compares this behavior against GNU less.
#[test]
fn test_less_invalid_arg() {
    println!("\n\x1b[32;1m === test Rust less invalid arg ===\x1b[0m");

    let (_, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let invalid_args = vec![
        "--invalid",
        "--lines -10",
        "--number -10",
        "--from-line -10",
    ];

    for arg in &invalid_args {
        let rust_script = format!(
            r#"
            spawn {} {}
            expect {{
                "invalid option" {{ exit 0 }}
                timeout {{ exit 1 }}
            }}
            "#,
            oe_less, arg
        );

        let rust_output = run_expect_script(&rust_script);

        let stdout = String::from_utf8_lossy(&rust_output.stdout());
        assert!(
            stdout.contains("which wasn't expected"),
            "Unexpected stdout: {}",
            stdout
        );
    }

    println!("\x1b[32;1m === test GNU less invalid arg ===\x1b[0m");

    for arg in &invalid_args {
        let gnu_script = format!(
            r#"
            spawn less {}
            expect {{
                "invalid option" {{ exit 0 }}
                timeout {{ exit 1 }}
            }}
            "#,
            arg
        );

        let output = run_expect_script(&gnu_script);

        let stdout = String::from_utf8_lossy(&output.stdout());
        assert!(
            stdout.contains("There is no"),
            "Unexpected stdout: {}",
            stdout
        );
    }

    println!("\x1b[32;1m === test less invalid arg finish ===\x1b[0m");
}

/// Test less behavior when passing valid arguments
///
/// This test verifies that when valid options or argument values are passed,
/// less correctly processes them and compares this behavior against GNU less.
#[test]
fn test_less_valid_arg() {
    println!("\n\x1b[32;1m === test Rust less valid arg ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "test_file.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    let valid_args = vec![
        "-b 128",
        "--buffer-size 128",
        "-c",
        "--clean-print",
        "-e",
        "--exit_at_eof",
        "--print-over",
        "-q",
        "--silent",
    ];

    for arg in &valid_args {
        let rust_script = format!(
            r#"
            spawn {} --non-interactive {} {}
            expect {{
                "End of file" {{ send "q\r"; exit 0 }}
                timeout {{ exit 1 }}
            }}
            "#,
            oe_less,
            arg,
            file_path.display()
        );

        // let rust_output =
        //     run_child_thread(format!("{} {}", arg, file_path.display()).as_str(), "Rust")
        //         .expect("Failed to run Rust less with line numbers");
        let rust_output = run_expect_script(&rust_script);

        let _stdout = String::from_utf8_lossy(&rust_output.stdout());
    }

    println!("\x1b[32;1m === test GNU less valid arg ===\x1b[0m");

    let valid_args = vec![
        "-bFX 128",
        "-FX --buffers 128",
        "-FX--clear-screen",
        "-eFX",
        "-FX --quit-at-eof",
        "-qFX",
        "-FX --silent",
    ];
    for arg in &valid_args {
        let gnu_script = format!("less {} {}", arg, file_path.display());

        let gnu_output = run_less_with_script(&gnu_script).expect("Failed to run less");

        println!("GNU stdout: {}", gnu_output);
    }

    println!("\x1b[32;1m === test less valid arg finish ===\x1b[0m");
}

/// Test less line squeezing functionality
///
/// This test verifies that when using the -s or --squeeze option,
/// less can correctly compress consecutive blank lines and compares
/// this behavior against GNU less.
#[test]
fn test_less_squeeze() {
    println!("\n\x1b[32;1m === test Rust less squeeze ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    let test_file = "squeeze.txt";
    let file_path = format!("{}/tests/fixtures/less/{}", proj_dir, test_file);

    let file = fs::read_to_string(&file_path).expect("Failed to read file");
    let original_line_count = file.lines().count();
    println!("original_line_count: {}", original_line_count);
    let blank_line_count = file.lines().filter(|line| line.is_empty()).count();
    println!("blank_line_count: {}", blank_line_count);
    let expected_line_count = original_line_count - blank_line_count + 1; // +1 for the single squeezed blank line

    let rust_script = format!(
        r#"
            spawn {} --non-interactive -s {}
            expect {{
                "End of file" {{ send "q\r"; exp_continue }}
                eof {{ exit 0 }}
                timeout {{ exit 1 }}
            }}
            "#,
        oe_less, test_file
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());

    let output_line_count = stdout.lines().count();
    println!("Rust stdout line count: {}", output_line_count);
    assert_eq!(
        output_line_count - 2,
        expected_line_count,
        "Unexpected line count in GNU less output: {}",
        output_line_count
    );

    println!("\x1b[32;1m === test GNU less squeeze ===\x1b[0m");
    let gnu_script = format!("less -sFX {}", test_file);

    let gnu_stdout = run_less_with_script(&gnu_script).expect("Failed to run less");
    let gnu_output_line_count = gnu_stdout.lines().count();
    println!("GNU stdout line count:  {}", gnu_output_line_count);
    assert_eq!(
        gnu_output_line_count - 3,
        expected_line_count,
        "Unexpected line count in GNU less output: {}",
        gnu_stdout
    );

    println!("\x1b[32;1m === test less squeeze finish ===\x1b[0m");
}

/// Test less output redirection functionality
/// This test verifies that when using the -o option, less can correctly
/// output content to a specified file and compares this behavior against
/// GNU less.
#[test]
fn test_less_outputs() {
    println!("\n\x1b[32;1m === test Rust less outputs ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let input = PathBuf::from(&proj_dir)
        .join(FIXTURE_PATH)
        .join("input.txt");
    let output = PathBuf::from(&proj_dir)
        .join(FIXTURE_PATH)
        .join("output.txt");

    println!("Input path: {}", input.display());
    println!("Output path: {}", output.display());

    // run_child_thread(
    //     format!("{} -o {}", input.display(), output.display()).as_str(),
    //     "Rust",
    // )
    // .expect("Failed to run Rust less with output redirection");

    let rust_script = format!(
        r#"
        spawn {} --non-interactive {} -o {}
        expect {{
            "End of file" {{ send "q\r"; exit 0 }}
            timeout {{ exit 1 }}
        }}
    "#,
        oe_less,
        input.display(),
        output.display()
    );

    run_expect_script(&rust_script);

    let output_content = fs::read_to_string(&output).expect("Failed to read Rust output file");
    assert!(output_content.contains("Rust"));
    assert!(output_content.contains("Python"));
    assert!(output_content.contains("C++"));
    assert!(output_content.contains("Java"));
    assert!(output_content.contains("Go"));

    println!("\x1b[32;1m === test GNU less outputs ===\x1b[0m");

    let task = TestScenario::new(util_name!());
    task.cmd(C_LESS_PATH)
        .arg(format!("{} > {}", input.display(), output.display()))
        .succeeds();

    // Verify the output file content
    let gnu_output_content = fs::read_to_string(&output).expect("Failed to read GNU output file");
    // println!("GNU Output content:\n{}", gnu_output_content);
    assert!(gnu_output_content.contains("Rust"));
    assert!(gnu_output_content.contains("Python"));
    assert!(gnu_output_content.contains("C++"));
    assert!(gnu_output_content.contains("Java"));
    assert!(gnu_output_content.contains("Go"));

    println!("\x1b[32;1m === test less outputs finish ===\x1b[0m");
}
/// Test less line number display functionality
///
/// This test verifies that when using the -N or --show-line-numbers option,
/// less can correctly display line numbers and compares this behavior against
/// GNU less.
///
fn strip_ansi_sequences(input: &str) -> String {
    let ansi_regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    ansi_regex.replace_all(input, "").to_string()
}

#[test]
fn test_less_line_numbers() {
    println!("\n\x1b[32;1m === test Rust less line numbers ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "line_numbers.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    let rust_script = format!(
        r#"
        spawn {} --non-interactive -N {}
        expect {{
            "End of file" {{ send "q\r"; exit 0 }}
            timeout {{ exit 1 }}
        }}
    "#,
        oe_less,
        file_path.display()
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());

    for i in 1..=5 {
        let expected_line = format!("{:>4}  Line{}", i, i);

        let clean_output = strip_ansi_sequences(&stdout);
        let clean_expected = strip_ansi_sequences(&expected_line);

        assert!(
            clean_output.contains(&clean_expected),
            "Rust less output does not contain expected line: {}",
            clean_expected
        );
    }

    println!("\x1b[32;1m === test GNU less line numbers ===\x1b[0m");

    let gnu_script = format!("less -NFX {}", file_path.display());

    let gnu_output = run_less_with_script(&gnu_script).expect("Failed to run less");

    for i in 1..=5 {
        let expected_line = format!("{:>4} Line{}", i, i);

        let clean_output = strip_ansi_sequences(&gnu_output);
        let clean_expected = strip_ansi_sequences(&expected_line);

        assert!(
            clean_output.contains(&clean_expected),
            "Rust less output does not contain expected line: {}",
            clean_expected
        );
    }

    println!("\x1b[32;1m === test less line numbers finish ===\x1b[0m");
}

/// Test less percentage display functionality
///
/// This test verifies that when using the -m or --show-percentage option,
/// less can correctly display the reading percentage and compares this
/// behavior against GNU less.
#[test]
fn test_less_show_percentage() {
    println!("\n\x1b[32;1m === test Rust less show percentage ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "show_percentage.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    // let _rust_output = run_child_thread("-m show_percentage.txt", "Rust")
    //     .expect("Failed to run Rust less with line numbers");

    let rust_script = format!(
        r#"
        spawn {} -m {}
        expect {{
            "%" {{ send "q\r"; exit 0 }}
            timeout {{ exit 1 }}
        }}
    "#,
        oe_less,
        file_path.display()
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());

    assert!(
        stdout.contains("%"),
        "Rust less output does not contain expected line: {}",
        stdout
    );

    println!("\x1b[32;1m === test GNU less show percentage ===\x1b[0m");

    let _gnu_output = run_child_thread(format!("-m {}", file_path.display()).as_str(), "GNU")
        .expect("Failed to run Rust less with line numbers");
    println!("\x1b[32;1m === test less show percentage finish ===\x1b[0m");
}

/// Test less functionality to start display from a specified line
///
/// This test verifies that when using the -F or --from-line option,
/// less can correctly start displaying content from the specified line
/// and compares this behavior against GNU less.
#[test]
fn test_less_argument_from_file() {
    println!("\n\x1b[32;1m === test Rust less argument from file ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "argument_from_file.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    // Define scenarios for different starting lines
    let scenarios = vec![
        ("2", vec!["Line2", "Line3", "Line4", "Line5"], vec!["Line1"]),
        ("4", vec!["Line4", "Line5"], vec!["Line1", "Line2", "Line3"]),
    ];

    for (start_line, should_contain, should_not_contain) in &scenarios {
        println!(
            "\x1b[32;1m === test Rust less with -F {} ===\x1b[0m",
            start_line
        );

        let rust_script = format!(
            r#"
            spawn {} --non-interactive -F {} {}
            expect {{
                "End of file" {{ send "q\r"; exit 0 }}
                timeout {{ exit 1 }}
            }}
            "#,
            oe_less, start_line, file
        );

        let rust_output = run_expect_script(&rust_script);

        let stdout = String::from_utf8_lossy(&rust_output.stdout());

        for line in should_contain {
            assert!(
                stdout.contains(line),
                "Expected to find '{}' in Rust less output for -F {}: {}",
                line,
                start_line,
                stdout
            );
        }
        for line in should_not_contain {
            assert!(
                !stdout.contains(line),
                "Did not expect to find '{}' in Rust less output for -F {}: {}",
                line,
                start_line,
                stdout
            );
        }
    }

    println!("\x1b[32;1m === test GNU less argument from file ===\x1b[0m");

    for (start_line, _should_contain, _should_not_contain) in &scenarios {
        println!(
            "\x1b[32;1m === test GNU less with -F {} ===\x1b[0m",
            start_line
        );

        let _gnu_output = run_child_thread(
            format!("+{} {}", start_line, file_path.display()).as_str(),
            "GNU",
        )
        .expect("Failed to run Rust less with line numbers");
    }

    println!("\x1b[32;1m === test less argument from file finish ===\x1b[0m");
}

/// Test error handling for multiple invalid file arguments
///
/// This test verifies less's error handling behavior when provided with
/// multiple invalid arguments (such as non-existent files) and compares
/// this behavior against GNU less.
#[test]
fn test_less_error_on_multiple_files() {
    println!("\n\x1b[32;1m === test Rust less error on multiple files ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let test_dir = format!("{}/tests/fixtures/less/test_dir", proj_dir);
    let invalid_file1 = format!("{}/non_existent_file1", test_dir);
    let invalid_file2 = format!("{}/non_existent_file2", test_dir);

    // Ensure the test directory exists
    std::fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Run Rust less with multiple invalid file arguments
    let rust_script = format!(
        r#"
        spawn {} {} {}
        expect {{
            "No such file or directory" {{ exp_continue }}
            "End of file" {{ send "q\r"; exp_continue }}
            eof {{ exit 0 }}
            timeout {{ exit 1 }}
        }}
        "#,
        oe_less, invalid_file1, invalid_file2
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());
    assert!(
        stdout.contains("No such file or directory"),
        "Unexpected stdout for multiple invalid files: {}",
        stdout
    );

    println!("\x1b[32;1m === test GNU less error on multiple files ===\x1b[0m");

    // Run GNU less with multiple invalid file arguments
    let gnu_script = format!(
        r#"
        spawn less {} {}
        expect {{
            -re "\\(END\\)" {{ send "q\r"; exp_continue }}
            "No such file or directory" {{ exp_continue }}
            eof {{ exit 0 }}
            timeout {{ exit 1 }}
        }}
        "#,
        invalid_file1, invalid_file2
    );

    let gnu_output = run_expect_script(&gnu_script);

    let gnu_stdout = String::from_utf8_lossy(&gnu_output.stdout());
    assert!(
        gnu_stdout.contains("No such file or directory"),
        "Unexpected stdout for multiple invalid files: {}",
        gnu_stdout
    );

    println!("\x1b[32;1m === test less error on multiple files finish ===\x1b[0m");
}

/// Test pattern search functionality (pattern found)
///
/// This test verifies that when using the -P option to provide an existing pattern,
/// less can correctly locate and display the matching content and compares this
/// behavior against GNU less.
#[test]
fn test_less_pattern_found() {
    println!("\n\x1b[32;1m === test Rust less pattern found ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "pattern.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    // let rust_output = run_child_thread(
    //     format!("-P Python {}", file_path.display()).as_str(),
    //     "Rust",
    // )
    // .expect("Failed to run Rust less with line numbers");

    let rust_script = format!(
        r#"
            spawn {} --non-interactive -P Python {}
            expect {{
                "End of file" {{ send "q\r"; exit 0 }}
                timeout {{ exit 1 }}
            }}
            "#,
        oe_less,
        file_path.display()
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());

    assert!(
        stdout.contains("Python"),
        "Expected to find 'Python' in Rust less output: {}",
        stdout
    );
    assert!(
        !stdout.contains("Rust"),
        "Did not expect to find 'Rust' in Rust less output: {}",
        stdout
    );

    println!("\x1b[32;1m === test GNU less pattern found ===\x1b[0m");

    // Run GNU less with the -P option
    // let gnu_script = format!(
    //     r#"
    //     spawn less -P Python {}
    //     expect {{
    //         -re "Python" {{ send "q\r"; exp_continue }}
    //         eof {{ exit 0 }}
    //         timeout {{ exit 1 }}
    //     }}
    //     "#,
    //     file_path.display()
    // );

    // let _gnu_output = run_expect_script(&gnu_script);
    let gnu_output = run_child_thread(format!("-P Python {}", file_path.display()).as_str(), "GNU")
        .expect("Failed to run Rust less with line numbers");

    // let gnu_script = format!("less -PFX Python {}", file_path.display());

    // let gnu_output = run_less_with_script(&gnu_script).expect("Failed to run less");

    assert!(
        gnu_output.contains("Python"),
        "Expected to find 'Python' in Rust less output: {}",
        gnu_output
    );
    assert!(
        !gnu_output.contains("Php"),
        "Did not expect to find 'Php' in Rust less output: {}",
        gnu_output
    );

    println!("\x1b[32;1m === test less pattern found finish ===\x1b[0m");
}

/// Test pattern search functionality (pattern not found)
///
/// This test verifies that when using the -P option to provide a non-existent pattern,
/// less can correctly report that no match was found and compares this behavior
/// against GNU less.
#[test]
fn test_less_pattern_not_found() {
    println!("\n\x1b[32;1m === test Rust less pattern not found ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let test_file = "tests/fixtures/less/pattern.txt";
    let file_path = format!("{}/{}", proj_dir, test_file);

    // Prepare the test file content
    let content = "Rust\nPython\nJava\nC++\nGo\n";
    std::fs::write(&file_path, content).expect("Failed to write test file");

    // Run Rust less with the -P option for a non-existent pattern
    let rust_script = format!(
        r#"
        spawn {} -P NonExistentPattern {}
        expect {{
            "Pattern not found" {{ exp_continue }}
            "End of file" {{ send "q\r"; exp_continue }}
            eof {{ exit 0 }}
            timeout {{ exit 1 }}
        }}
        "#,
        oe_less, file_path
    );

    let rust_output = run_expect_script(&rust_script);

    let stdout = String::from_utf8_lossy(&rust_output.stdout());
    assert!(
        stdout.contains("Pattern not found"),
        "Expected to find 'Pattern not found' in Rust less output: {}",
        stdout
    );

    println!("\x1b[32;1m === test GNU less pattern not found ===\x1b[0m");

    // Run GNU less with the -P option for a non-existent pattern
    let gnu_script = format!(
        r#"
        spawn less -P NonExistentPattern {}
        send "q\r"
        expect {{
            eof {{ exit 0 }}
            timeout {{ exit 1 }}
        }}
        "#,
        file_path
    );

    let _gnu_output = run_expect_script(&gnu_script);

    println!("\x1b[32;1m === test less pattern not found finish ===\x1b[0m");
}

/// Test less functionality in CI environment
#[test]
#[ignore]
fn test_less_in_ci() {
    println!("\n\x1b[32;1m === test Rust less pattern not found ===\x1b[0m");

    let (proj_dir, oe_less) = get_rust_less_path();
    println!("Rust less executable path: {}", oe_less);

    let file = "test_ci_env.txt";
    let file_path = PathBuf::from(&proj_dir).join(FIXTURE_PATH).join(file);

    let task = TestScenario::new(util_name!());
    let output = task
        .ucmd()
        .env("CARGO_TEST", "1")
        .arg("--non-interactive")
        .arg("-F")
        .arg("8")
        .arg("-N")
        .arg("-P")
        .arg("Python")
        .arg(file_path.display().to_string())
        .succeeds();
    let stdout = String::from_utf8_lossy(&output.stdout());
    println!("Rust less output: {}", stdout);
    let expected = "     8  Python\n";
    let clean_output = strip_ansi_sequences(&stdout);
    assert!(
        clean_output.contains(&expected),
        "Rust less output does not contain expected line: {}",
        expected
    );
    println!("\x1b[32;1m === test less pattern not found finish ===\x1b[0m");
}
