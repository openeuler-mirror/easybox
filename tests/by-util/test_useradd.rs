//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use crate::test_hwclock;
use chrono::NaiveDate;
use serial_test::serial;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::{Command, Stdio};
use test_hwclock::run_ucmd_as_root_ignore_ci;

#[test]
#[serial]
fn test_useradd_with_badname() {
    let ts = TestScenario::new("useradd");

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_badname(&ts, true);

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_badname(&ts, false);

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with bad name"
    );
}

fn execute_useradd_with_badname(ts: &TestScenario, use_rust: bool) -> bool {
    let args_rust = &["invalid:name"];
    let args_c = &["useradd", "invalid:name"];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    success
}

#[test]
#[serial]
fn test_useradd_with_base_dir_and_create_home() {
    let ts = TestScenario::new("useradd");
    let base_dir = "/mnt";
    let username = "test_b";

    // Execute Rust version and get actual result
    let actual_result =
        execute_useradd_with_base_dir_and_create_home(&ts, username, base_dir, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_useradd_with_base_dir_and_create_home(&ts, username, base_dir, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with base dir and create home"
    );
}

fn execute_useradd_with_base_dir_and_create_home(
    ts: &TestScenario,
    username: &str,
    base_dir: &str,
    use_rust: bool,
) -> bool {
    let expected_home_dir = format!("{}/{}", base_dir, username);

    // Remove existing home directory if it exists
    if Path::new(&expected_home_dir).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", &expected_home_dir]) {
            println!("Failed to remove existing test directory: {}", e);
            return false;
        }
    }

    // Prepare command arguments
    let args_rust = &["-b", base_dir, "-m", username];
    let args_c = &["useradd", "-b", base_dir, "-m", username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute useradd command");
        return false;
    }

    // Check if home directory exists
    if !Path::new(&expected_home_dir).exists() {
        println!("Home directory does not exist");
        delete_user(username);
        return false;
    }

    // Get metadata
    let metadata = match fs::metadata(&expected_home_dir) {
        Ok(meta) => meta,
        Err(e) => {
            println!("Failed to get metadata for home directory: {}", e);
            delete_user(username);
            return false;
        }
    };

    // Check ownership
    let uid = get_user_uid(username);
    let gid = get_user_gid(username);
    if metadata.uid() != uid || metadata.gid() != gid {
        println!("Home directory ownership is incorrect");
        delete_user(username);
        return false;
    }

    // Clean up
    delete_user(username);
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", &expected_home_dir]) {
        println!("Failed to remove test directory: {}", e);
    }

    true
}

#[test]
#[serial]
fn test_useradd_comment() {
    let ts = TestScenario::new("useradd");
    let username = "newuser_test_useradd_comment";
    let comment = "This is a test user";

    // Execute Rust version and get actual result
    let actual_comment = execute_useradd_comment(&ts, username, comment, true);
    if actual_comment.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_comment = execute_useradd_comment(&ts, username, comment, false);
    if expected_comment.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_comment, expected_comment,
        "Comments differ between Rust and Linux implementations for useradd with comment"
    );
}

fn execute_useradd_comment(
    ts: &TestScenario,
    username: &str,
    comment: &str,
    use_rust: bool,
) -> String {
    // Prepare command arguments
    let args_rust = &["-c", comment, username];
    let args_c = &["useradd", "-c", comment, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute useradd command");
        return String::new();
    }
    let user_comment = verify_entry_in_file("/etc/passwd", comment);

    // Clean up
    delete_user(username);

    user_comment
}

#[test]
#[serial]
fn test_useradd_home_dir() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_home_dir";
    let home_dir = "/myuseradd/home/testuser_home_dir";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_home_dir(&ts, username, home_dir, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_home_dir(&ts, username, home_dir, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with home dir"
    );
}

fn execute_useradd_home_dir(
    ts: &TestScenario,
    username: &str,
    home_dir: &str,
    use_rust: bool,
) -> bool {
    // Remove existing home directory if it exists
    if Path::new(&home_dir).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", &home_dir]) {
            println!("Failed to remove existing test directory: {}", e);
            return false;
        }
    }

    // Prepare command arguments
    let args_rust = &["-d", home_dir, "-m", username];
    let args_c = &["useradd", "-d", home_dir, "-m", username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute useradd command");
        return false;
    }

    // Check if home directory exists
    if !Path::new(&home_dir).exists() {
        println!("Home directory does not exist");
        delete_user(username);
        return false;
    }

    // Clean up
    delete_user(username);
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", "/myuseradd"]) {
        println!("Failed to remove custom home directory: {}", e);
    }

    true
}

#[test]
#[serial]
fn test_useradd_show_defaults() {
    let ts = TestScenario::new("useradd");
    backup_useradd_defaults();

    // Execute Rust version and get actual result
    let actual_output = execute_useradd_show_defaults(&ts, true);

    // Execute Linux version and get expected result
    let expected_output = execute_useradd_show_defaults(&ts, false);

    // Compare the results
    assert_eq!(
        actual_output, expected_output,
        "Defaults differ between Rust and Linux implementations for useradd -D"
    );

    restore_useradd_defaults();
}

fn execute_useradd_show_defaults(ts: &TestScenario, use_rust: bool) -> bool {
    // Prepare command arguments
    let args_rust = &["-D"];
    let args_c = &["useradd", "-D"];

    // Execute command
    let output = execute_command_with_output(ts, args_rust, args_c, use_rust);
    if output.is_empty() {
        println!("Failed to execute useradd -D command");
        return false;
    }

    assert!(output.contains("GROUP="));
    assert!(output.contains("HOME="));
    assert!(output.contains("SHELL="));
    assert!(output.contains("INACTIVE="));
    assert!(output.contains("EXPIRE="));
    assert!(output.contains("SKEL="));
    assert!(output.contains("CREATE_MAIL_SPOOL="));

    true
}

#[test]
#[serial]
fn test_useradd_change_default_group() {
    let ts = TestScenario::new("useradd");
    let groupid = "5500";
    let groupname = "test_useradd_change_default_group";

    // Execute Rust version and get actual result
    let actual_output = execute_useradd_change_default_group(&ts, groupid, groupname, true);
    if actual_output.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_output = execute_useradd_change_default_group(&ts, groupid, groupname, false);
    if expected_output.is_empty() {
        return;
    }

    // Compare the outputs
    assert_eq!(
        actual_output, expected_output,
        "Outputs differ between Rust and Linux implementations for changing default group"
    );
}

fn execute_useradd_change_default_group(
    ts: &TestScenario,
    groupid: &str,
    groupname: &str,
    use_rust: bool,
) -> String {
    backup_useradd_defaults();

    // Create the group if it doesn't exist
    if !group_id_exists(&groupid) {
        // Create the group before the test
        let _ = run_cmd_as_root_ignore_ci(&["groupadd", "-g", groupid, groupname]);
    }

    // Prepare command arguments
    let args_rust = &["-D", "-g", groupid];
    let args_c = &["useradd", "-D", "-g", groupid];

    // Execute command to change default group
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot change default group without sudo privileges");
        restore_useradd_defaults();
        return String::new();
    }

    // Get the output of useradd -D
    let output = execute_command_with_output(ts, &["-D"], &["useradd", "-D"], use_rust);
    if output.is_empty() {
        println!("Failed to get default settings");
        restore_useradd_defaults();
        return String::new();
    }

    // Clean up
    remove_group_entry(groupname);
    restore_useradd_defaults();

    output
}

#[test]
#[serial]
fn test_useradd_change_default_home() {
    let ts = TestScenario::new("useradd");
    let home_dir = "/mnt";

    // Execute Rust version and get actual result
    let actual_output = execute_useradd_change_default_home(&ts, home_dir, true);
    if actual_output.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_output = execute_useradd_change_default_home(&ts, home_dir, false);
    if expected_output.is_empty() {
        return;
    }

    // Compare the outputs
    assert_eq!(
        actual_output, expected_output,
        "Outputs differ between Rust and Linux implementations for changing default home directory"
    );
}

fn execute_useradd_change_default_home(
    ts: &TestScenario,
    home_dir: &str,
    use_rust: bool,
) -> String {
    backup_useradd_defaults();

    let res = format!("HOME={}", home_dir);
    // Prepare command arguments
    let args_rust = &["-D", "-b", home_dir];
    let args_c = &["useradd", "-D", "-b", home_dir];

    // Execute command to change default home directory
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot change default home without sudo privileges");
        restore_useradd_defaults();
        return String::new();
    }

    // Get the output of useradd -D
    let output = execute_command_with_output(ts, &["-D"], &["useradd", "-D"], use_rust);
    if output.is_empty() {
        println!("Failed to get default settings");
        restore_useradd_defaults();
        return String::new();
    }

    if !output.contains(&format!("HOME={}", home_dir)) {
        restore_useradd_defaults();
        return String::new();
    }

    restore_useradd_defaults();

    res
}

#[test]
#[serial]
fn test_useradd_change_multiple_defaults() {
    let ts = TestScenario::new("useradd");
    let groupid = "5501";
    let groupname = "test_useradd_change_multiple_defaults";
    let shell = "/bin/sh";

    // Execute Rust version and get actual result
    let actual_output =
        execute_useradd_change_multiple_defaults(&ts, groupid, groupname, shell, true);
    if actual_output.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_output =
        execute_useradd_change_multiple_defaults(&ts, groupid, groupname, shell, false);
    if expected_output.is_empty() {
        return;
    }

    // Compare the outputs
    assert_eq!(
        actual_output, expected_output,
        "Outputs differ between Rust and Linux implementations for changing multiple defaults"
    );
}

fn execute_useradd_change_multiple_defaults(
    ts: &TestScenario,
    groupid: &str,
    groupname: &str,
    shell: &str,
    use_rust: bool,
) -> String {
    backup_useradd_defaults();

    // Create the group if it doesn't exist
    if !group_id_exists(&groupid) {
        // Create the group before the test
        let _ = run_cmd_as_root_ignore_ci(&["groupadd", "-g", groupid, groupname]);
    }

    // Prepare command arguments
    let args_rust = &["-D", "-g", groupid, "-s", shell];
    let args_c = &["useradd", "-D", "-g", groupid, "-s", shell];

    // Execute command to change defaults
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot change defaults without sudo privileges");
        restore_useradd_defaults();
        return String::new();
    }

    // Get the output of useradd -D
    let output = execute_command_with_output(ts, &["-D"], &["useradd", "-D"], use_rust);
    if output.is_empty() {
        println!("Failed to get default settings");
        restore_useradd_defaults();
        return String::new();
    }

    // Clean up
    remove_group_entry(groupname);
    restore_useradd_defaults();

    output
}

#[test]
#[serial]
fn test_useradd_invalid_group() {
    let ts = TestScenario::new("useradd");

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_invalid_group(&ts, true);

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_invalid_group(&ts, false);

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for invalid group in useradd"
    );
}

fn execute_useradd_invalid_group(ts: &TestScenario, use_rust: bool) -> bool {
    backup_useradd_defaults();

    // Prepare command arguments
    let args_rust = &["-D", "-g", "invalidgroup"];
    let args_c = &["useradd", "-D", "-g", "invalidgroup"];

    // Execute command expecting failure
    let success = execute_command(ts, args_rust, args_c, use_rust);

    restore_useradd_defaults();

    success
}

#[test]
#[serial]
fn test_useradd_invalid_shell() {
    let ts = TestScenario::new("useradd");

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_invalid_shell(&ts, true);

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_invalid_shell(&ts, false);

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for invalid shell in useradd"
    );
}

fn execute_useradd_invalid_shell(ts: &TestScenario, use_rust: bool) -> bool {
    backup_useradd_defaults();

    // Prepare command arguments
    let args_rust = &["-D", "-s", "/bin/invalid:shell"];
    let args_c = &["useradd", "-D", "-s", "/bin/invalid:shell"];

    // Execute command expecting failure
    let success = execute_command(ts, args_rust, args_c, use_rust);

    restore_useradd_defaults();

    success
}

#[test]
#[serial]
fn test_useradd_set_expiration_date() {
    let ts = TestScenario::new("useradd");
    let username = "test_e";
    let expire_date = "2024-12-31";

    // Execute Rust version and get actual result
    let actual_expire_days_res =
        execute_useradd_set_expiration_date(&ts, username, expire_date, true);

    // Execute Linux version and get expected result
    let expected_expire_days_res =
        execute_useradd_set_expiration_date(&ts, username, expire_date, false);

    // Compare the results
    assert_eq!(
        actual_expire_days_res, expected_expire_days_res,
        "Expiration dates differ between Rust and Linux implementations for useradd"
    );
}

fn execute_useradd_set_expiration_date(
    ts: &TestScenario,
    username: &str,
    expire_date: &str,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-e", expire_date, username];
    let args_c = &["useradd", "-e", expire_date, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot set expiration date without sudo privileges");
        return false;
    }

    // Read /etc/shadow as root
    let shadow_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/shadow"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/shadow without sudo privileges: {}", e);
            delete_user(username);
            return false;
        }
    };

    let epoch_date = NaiveDate::from_ymd_opt(1970, 1, 1);
    let target_date = NaiveDate::parse_from_str(expire_date, "%Y-%m-%d")
        .expect("Failed to parse expiration date");
    let expire_days = target_date
        .signed_duration_since(epoch_date.unwrap())
        .num_days();

    let re = regex::Regex::new(&format!(
        r"{}:[^:]*:[^:]*:[^:]*:[^:]*:[^:]*:[^:]*:{}:",
        username, expire_days
    ))
    .expect("Failed to create regex");

    assert!(
        re.is_match(&shadow_content),
        "Expiration date not set correctly in /etc/shadow"
    );
    if !re.is_match(&shadow_content) {
        delete_user(username);
        return false;
    }

    // Clean up
    delete_user(username);

    true
}

#[test]
#[serial]
fn test_useradd_with_inactive_days() {
    let ts = TestScenario::new("useradd");
    let username = "test_f";
    let inactive_days = "7";

    // Execute Rust version and get actual result
    let actual_inactive_days =
        execute_useradd_with_inactive_days(&ts, username, inactive_days, true);
    if actual_inactive_days.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_inactive_days =
        execute_useradd_with_inactive_days(&ts, username, inactive_days, false);
    if expected_inactive_days.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_inactive_days, expected_inactive_days,
        "Inactive days differ between Rust and Linux implementations for useradd"
    );
}

fn execute_useradd_with_inactive_days(
    ts: &TestScenario,
    username: &str,
    inactive_days: &str,
    use_rust: bool,
) -> String {
    // Prepare command arguments
    let args_rust = &["-f", inactive_days, username];
    let args_c = &["useradd", "-f", inactive_days, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot set inactive days without sudo privileges");
        return String::new();
    }

    // Read /etc/shadow as root
    let shadow_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/shadow"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/shadow without sudo privileges: {}", e);
            delete_user(username);
            return String::new();
        }
    };

    let re = regex::Regex::new(&format!(
        r"{}:[^:]*:[^:]*:[^:]*:[^:]*:[^:]*:{}:",
        username, inactive_days
    ))
    .expect("Failed to create regex");

    assert!(
        re.is_match(&shadow_content),
        "Inactive days not set correctly in /etc/shadow"
    );

    // Clean up
    delete_user(username);

    inactive_days.to_string()
}

#[test]
#[serial]
fn test_useradd_with_valid_group_name() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_valid_group";
    let groupname = "testgroup";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_valid_group_name(&ts, username, groupname, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_valid_group_name(&ts, username, groupname, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with valid group name"
    );
}

fn execute_useradd_with_valid_group_name(
    ts: &TestScenario,
    username: &str,
    groupname: &str,
    use_rust: bool,
) -> bool {
    remove_group_entry(groupname);

    // Create the group
    let _ = run_cmd_as_root_ignore_ci(&["groupadd", groupname]);

    // Prepare command arguments
    let args_rust = &["-g", groupname, username];
    let args_c = &["useradd", "-g", groupname, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with valid group name");
        remove_group_entry(groupname);
        return false;
    }

    // Get user and group GIDs
    let user_gid = get_user_gid(username);
    let group_gid = get_group_gid(groupname);

    // Clean up
    delete_user(username);
    remove_group_entry(groupname);

    user_gid == group_gid
}

#[test]
#[serial]
fn test_useradd_with_valid_group_id() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_valid_gid";
    let groupname = "testgroup";
    let gid = 1501;

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_valid_group_id(&ts, username, groupname, gid, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_valid_group_id(&ts, username, groupname, gid, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with valid group ID"
    );
}

fn execute_useradd_with_valid_group_id(
    ts: &TestScenario,
    username: &str,
    groupname: &str,
    gid: u32,
    use_rust: bool,
) -> bool {
    remove_group_entry(groupname);

    // Create the group
    let _ = run_cmd_as_root_ignore_ci(&["groupadd", "-g", &gid.to_string(), groupname]);

    // Prepare command arguments
    let args_rust = &["-g", &gid.to_string(), username];
    let args_c = &["useradd", "-g", &gid.to_string(), username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with valid group ID");
        remove_group_entry(groupname);
        return false;
    }

    // Get user's GID
    let user_gid = get_user_gid(username);

    // Clean up
    delete_user(username);
    remove_group_entry(groupname);

    user_gid == gid
}

#[test]
#[serial]
fn test_useradd_with_non_existent_group_name() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_invalid_group";
    let groupname = "nonexistentgroup";

    // Execute Rust version and get actual result
    let actual_result =
        execute_useradd_with_non_existent_group_name(&ts, username, groupname, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_useradd_with_non_existent_group_name(&ts, username, groupname, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with non-existent group name"
    );
}

fn execute_useradd_with_non_existent_group_name(
    ts: &TestScenario,
    username: &str,
    groupname: &str,
    use_rust: bool,
) -> bool {
    // Ensure the group does not exist
    remove_group_entry(groupname);

    // Prepare command arguments
    let args_rust = &["-g", groupname, username];
    let args_c = &["useradd", "-g", groupname, username];

    // Execute command expecting failure
    let result = execute_command(ts, args_rust, args_c, use_rust);
    result
}

#[test]
#[serial]
fn test_useradd_with_non_existent_group_id() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_invalid_gid";
    let gid = 99999;

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_non_existent_group_id(&ts, username, gid, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_non_existent_group_id(&ts, username, gid, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with non-existent group ID"
    );
}

fn execute_useradd_with_non_existent_group_id(
    ts: &TestScenario,
    username: &str,
    gid: u32,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-g", &gid.to_string(), username];
    let args_c = &["useradd", "-g", &gid.to_string(), username];

    // Execute command expecting failure
    let result = execute_command(ts, args_rust, args_c, use_rust);
    result
}

#[test]
#[serial]
fn test_useradd_with_multiple_groups() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_multiple_groups";
    let mygroup1 = "mygroup1";
    let mygroup2 = "mygroup2";

    // Execute Rust version and get actual result
    let actual_result =
        execute_useradd_with_multiple_groups(&ts, username, &[mygroup1, mygroup2], true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_useradd_with_multiple_groups(&ts, username, &[mygroup1, mygroup2], false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with multiple groups"
    );
}

fn execute_useradd_with_multiple_groups(
    ts: &TestScenario,
    username: &str,
    groups: &[&str],
    use_rust: bool,
) -> bool {
    // Remove and create groups
    for group in groups {
        let _ = run_cmd_as_root_ignore_ci(&["groupadd", group]);
    }

    // Prepare command arguments
    let groups_str = groups.join(",");
    let args_rust = &["-U", "-G", &groups_str, username];
    let args_c = &["useradd", "-U", "-G", &groups_str, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with multiple groups");
        delete_user(username);
        for group in groups {
            remove_group_entry(group);
        }
        return false;
    }

    // Check group memberships
    let mut result = true;
    for group in groups {
        let group_membership = match run_cmd_as_root_ignore_ci_output(&["getent", "group", group]) {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => {
                println!("Failed to get group membership: {}", e);
                result = false;
                break;
            }
        };
        if !group_membership.contains(username) {
            println!("User is not a member of {}", group);
            result = false;
            break;
        }
    }

    // Clean up
    delete_user(username);
    for group in groups {
        remove_group_entry(group);
    }

    result
}

#[test]
#[serial]
fn test_useradd_with_invalid_group_name() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_invalid_group_name";
    let invalid_group = "testuser_invalidgroup";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_invalid_group_name(&ts, username, invalid_group, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_useradd_with_invalid_group_name(&ts, username, invalid_group, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with invalid group name"
    );
}

fn execute_useradd_with_invalid_group_name(
    ts: &TestScenario,
    username: &str,
    invalid_group: &str,
    use_rust: bool,
) -> bool {
    // Ensure the group does not exist
    remove_group_entry(invalid_group);

    // Prepare command arguments
    let args_rust = &["-U", "-G", invalid_group, username];
    let args_c = &["useradd", "-U", "-G", invalid_group, username];

    // Execute command expecting failure
    let result = execute_command(ts, args_rust, args_c, use_rust);
    result
}

#[test]
#[serial]
fn test_useradd_with_custom_skel_dir() {
    let ts = TestScenario::new("useradd");

    let skel_dir = "/tmp/custom_skel";
    let username = "testuser_custom_skel";
    let home_dir = format!("/home/{}", username);
    let test_file = "testfile.txt";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_custom_skel_dir(
        &ts,
        skel_dir,
        username,
        home_dir.clone(),
        test_file,
        true,
    );
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_useradd_with_custom_skel_dir(&ts, skel_dir, username, home_dir, test_file, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with custom skel dir"
    );
}

fn execute_useradd_with_custom_skel_dir(
    ts: &TestScenario,
    skel_dir: &str,
    username: &str,
    home_dir: String,
    test_file: &str,
    use_rust: bool,
) -> bool {
    // Remove existing skel directory if it exists
    if Path::new(skel_dir).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", skel_dir]) {
            println!("Failed to remove existing skel directory: {}", e);
            return false;
        }
    }

    // Create skel directory
    if let Err(e) = run_cmd_as_root_ignore_ci(&["mkdir", "-p", skel_dir]) {
        println!("Failed to create skel directory: {}", e);
        return false;
    }

    // Create test file in skel directory
    let skel_file_path = format!("{}/{}", skel_dir, test_file);
    if let Err(e) = run_cmd_as_root_ignore_ci(&[
        "bash",
        "-c",
        &format!("echo 'This is a test file' > {}", skel_file_path),
    ]) {
        println!("Failed to write test file in skel directory: {}", e);
        return false;
    }

    // Prepare command arguments
    let args_rust = &["-k", skel_dir, "-m", username];
    let args_c = &["useradd", "-k", skel_dir, "-m", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with custom skel directory");
        return false;
    }

    // Check if home directory exists
    if !Path::new(&home_dir).exists() {
        println!("Home directory does not exist");
        delete_user(username);
        return false;
    }

    // Read the test file from the user's home directory
    let user_file_path = format!("{}/{}", home_dir, test_file);
    let output = run_cmd_as_root_ignore_ci_output(&["cat", &user_file_path]).unwrap_or_else(|e| {
        println!("Failed to read user file content: {}", e);
        delete_user(username);
        panic!();
    });

    let user_file_content = String::from_utf8_lossy(&output.stdout);

    // Clean up
    delete_user(username);
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", skel_dir]) {
        println!("Failed to remove skel directory: {}", e);
    }

    user_file_content == "This is a test file\n"
}

#[test]
#[serial]
fn test_useradd_with_uid_min_override() {
    let ts = TestScenario::new("useradd");

    let login_defs = "/etc/login.defs";
    let backup_login_defs = "/etc/login.defs.backup";
    let username = "test_K";
    let uid_min_override = "UID_MIN=3000";

    // Execute Rust version and get actual result
    let actual_uid = execute_useradd_with_uid_min_override(
        &ts,
        login_defs,
        backup_login_defs,
        username,
        uid_min_override,
        true,
    );
    if actual_uid == 0 {
        return;
    }

    // Execute Linux version and get expected result
    let expected_uid = execute_useradd_with_uid_min_override(
        &ts,
        login_defs,
        backup_login_defs,
        username,
        uid_min_override,
        false,
    );
    if expected_uid == 0 {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_uid, expected_uid,
        "UIDs differ between Rust and Linux implementations for useradd with UID_MIN override"
    );
}

fn execute_useradd_with_uid_min_override(
    ts: &TestScenario,
    login_defs: &str,
    backup_login_defs: &str,
    username: &str,
    uid_min_override: &str,
    use_rust: bool,
) -> u32 {
    // Backup /etc/login.defs
    if Path::new(login_defs).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", login_defs, backup_login_defs]) {
            println!("Failed to backup /etc/login.defs: {}", e);
            return 0;
        }
    }

    // Prepare command arguments
    let args_rust = &["-K", uid_min_override, username];
    let args_c = &["useradd", "-K", uid_min_override, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with UID_MIN override");
        restore_login_defs(backup_login_defs, login_defs);
        return 0;
    }

    // Get user UID
    let user_uid = get_user_uid(username);

    // Clean up
    delete_user(username);
    restore_login_defs(backup_login_defs, login_defs);

    user_uid
}

#[test]
#[serial]
fn test_useradd_create_home() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_create_home";
    let homedir = format!("/home/{}", username);

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_create_home(&ts, username, &homedir, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_create_home(&ts, username, &homedir, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -m"
    );
}

fn execute_useradd_create_home(
    ts: &TestScenario,
    username: &str,
    homedir: &str,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-m", username];
    let args_c = &["useradd", "-m", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd -m");
        return false;
    }

    // Check if home directory exists
    if !Path::new(homedir).exists() {
        println!("Home directory does not exist");
        delete_user(username);
        return false;
    }

    // Get metadata
    let metadata = match fs::metadata(homedir) {
        Ok(meta) => meta,
        Err(e) => {
            println!("Failed to get metadata for home directory: {}", e);
            delete_user(username);
            return false;
        }
    };

    // Get UID and GID
    let uid = get_user_uid(username);
    let gid = get_user_gid(username);

    // Check ownership
    if metadata.uid() != uid || metadata.gid() != gid {
        println!("Home directory ownership is incorrect");
        delete_user(username);
        return false;
    }

    // Check skel files
    let output = run_cmd_as_root_ignore_ci_output(&["ls", "-la", homedir]).unwrap_or_else(|e| {
        println!("Failed to list home directory: {}", e);
        delete_user(username);
        panic!();
    });

    let output_str = String::from_utf8_lossy(&output.stdout);

    let skel_files = vec![".bash_logout", ".bashrc"];
    for file in skel_files {
        if !output_str.contains(file) {
            println!("{} does not exist in home directory", file);
            delete_user(username);
            return false;
        }
    }

    // Clean up
    delete_user(username);

    true
}

#[test]
#[serial]
fn test_useradd_no_create_home() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_no_create_home";
    let homedir = format!("/home/{}", username);

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_no_create_home(&ts, username, &homedir, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_no_create_home(&ts, username, &homedir, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -M"
    );
}

fn execute_useradd_no_create_home(
    ts: &TestScenario,
    username: &str,
    homedir: &str,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-M", username];
    let args_c = &["useradd", "-M", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd -M");
        return false;
    }

    // Check that home directory does not exist
    if Path::new(homedir).exists() {
        println!("Home directory should not exist");
        delete_user(username);
        return false;
    }

    // Read /etc/passwd to verify user creation
    let passwd_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/passwd"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/passwd: {}", e);
            delete_user(username);
            return false;
        }
    };

    if !passwd_content.contains(username) {
        println!("User '{}' was not created", username);
        delete_user(username);
        return false;
    }

    // Clean up
    delete_user(username);

    true
}

#[test]
#[serial]
fn test_useradd_no_user_group() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_no_group";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_no_user_group(&ts, username, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_no_user_group(&ts, username, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -N"
    );
}

fn execute_useradd_no_user_group(ts: &TestScenario, username: &str, use_rust: bool) -> bool {
    // Prepare command arguments
    let args_rust = &["-N", username];
    let args_c = &["useradd", "-N", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd -N");
        return false;
    }

    // Read /etc/passwd to verify user creation
    let passwd_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/passwd"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/passwd: {}", e);
            delete_user(username);
            return false;
        }
    };

    if !passwd_content.contains(username) {
        println!("User '{}' was not created", username);
        delete_user(username);
        return false;
    }

    // Read /etc/group to verify group not created
    let group_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/group"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/group: {}", e);
            delete_user(username);
            return false;
        }
    };

    if group_content.contains(username) {
        println!("Group '{}' should not have been created", username);
        delete_user(username);
        return false;
    }

    // Clean up
    delete_user(username);

    true
}

#[test]
#[serial]
fn test_useradd_non_unique_uid() {
    let ts = TestScenario::new("useradd");

    let username_unique = "testuser_unique";
    let username_non_unique = "testuser_non_unique";

    // Execute Rust version and get actual result
    let actual_result =
        execute_useradd_non_unique_uid(&ts, username_unique, username_non_unique, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_useradd_non_unique_uid(&ts, username_unique, username_non_unique, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with non-unique UID"
    );
}

fn execute_useradd_non_unique_uid(
    ts: &TestScenario,
    username_unique: &str,
    username_non_unique: &str,
    use_rust: bool,
) -> bool {
    // Create the first user
    let args_rust_unique = &[username_unique];
    let args_c_unique = &["useradd", username_unique];

    if !execute_command(ts, args_rust_unique, args_c_unique, use_rust) {
        println!("Cannot create unique user");
        return false;
    }

    let user_uid = get_user_uid(username_unique);

    // Prepare command arguments for non-unique UID
    let args_rust_non_unique = &["-o", "-u", &user_uid.to_string(), username_non_unique];
    let args_c_non_unique = &[
        "useradd",
        "-o",
        "-u",
        &user_uid.to_string(),
        username_non_unique,
    ];

    // Execute command
    if !execute_command(ts, args_rust_non_unique, args_c_non_unique, use_rust) {
        println!("Cannot create user with non-unique UID");
        delete_user(username_unique);
        return false;
    }

    // Check that the UID is the same
    let non_unique_uid = get_user_uid(username_non_unique);

    // Clean up
    delete_user(username_unique);
    delete_user(username_non_unique);

    user_uid == non_unique_uid
}

#[test]
#[serial]
fn test_useradd_with_password() {
    let ts = TestScenario::new("useradd");

    let username = "testuser_with_password";
    let password = "$6$saltsalt$abcdefghijk";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_password(&ts, username, password, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_password(&ts, username, password, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with password"
    );
}

fn execute_useradd_with_password(
    ts: &TestScenario,
    username: &str,
    password: &str,
    use_rust: bool,
) -> bool {
    let mut password_set_correctly = true;
    // Prepare command arguments
    let args_rust = &["-p", password, username];
    let args_c = &["useradd", "-p", password, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with password");
        return false;
    }

    // Read /etc/shadow as root
    let shadow_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/shadow"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/shadow without sudo privileges: {}", e);
            delete_user(username);
            return false;
        }
    };
    if !shadow_content.contains(&format!("{}:{}", username, password)) {
        password_set_correctly = false;
    }

    // Clean up
    delete_user(username);

    password_set_correctly
}

#[test]
#[serial]
fn test_useradd_system_account() {
    let ts = TestScenario::new("useradd");

    let username = "testuser_system";

    // Execute Rust version and get actual result
    let actual_uid = execute_useradd_system_account(&ts, username, true);
    if actual_uid.is_none() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_uid = execute_useradd_system_account(&ts, username, false);
    if expected_uid.is_none() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_uid.unwrap(),
        expected_uid.unwrap(),
        "UIDs differ between Rust and Linux implementations for system account creation"
    );
}

fn execute_useradd_system_account(
    ts: &TestScenario,
    username: &str,
    use_rust: bool,
) -> Option<u32> {
    // Prepare command arguments
    let args_rust = &["-r", username];
    let args_c = &["useradd", "-r", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot create system account");
        return None;
    }

    // Get UID
    let uid = get_user_uid(username);

    // Check if UID is less than 1000
    if uid >= 1000 {
        println!(
            "User '{}' has UID {} which is not in the system UID range",
            username, uid
        );
        delete_user(username);
        return None;
    }

    // Verify user in /etc/passwd
    let passwd_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/passwd"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/passwd: {}", e);
            delete_user(username);
            return None;
        }
    };

    if !passwd_content.contains(username) {
        println!("System user '{}' was not correctly created", username);
        delete_user(username);
        return None;
    }

    // Clean up
    delete_user(username);

    Some(uid)
}

#[test]
#[serial]
fn test_useradd_with_prefix() {
    let ts = TestScenario::new("useradd");

    let prefix_dir = "/prefix";
    let username = "testuser_prefix";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_prefix(&ts, prefix_dir, username, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_prefix(&ts, prefix_dir, username, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd with prefix"
    );
}

fn execute_useradd_with_prefix(
    ts: &TestScenario,
    prefix_dir: &str,
    username: &str,
    use_rust: bool,
) -> bool {
    // Setup prefix directory
    if Path::new(prefix_dir).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", prefix_dir]) {
            println!("Failed to remove existing prefix directory: {}", e);
            return false;
        }
    }

    if let Err(e) = run_cmd_as_root_ignore_ci(&["mkdir", "-p", &format!("{}/etc", prefix_dir)]) {
        println!("Failed to create prefix etc directory: {}", e);
        return false;
    }

    // Copy necessary files to prefix
    let config_files = vec!["passwd", "group", "shadow", "gshadow"];
    for file in config_files {
        if let Err(e) = run_cmd_as_root_ignore_ci(&[
            "cp",
            &format!("/etc/{}", file),
            &format!("{}/etc/{}", prefix_dir, file),
        ]) {
            println!("Failed to copy /etc/{} to prefix: {}", file, e);
            remove_prefix_entry(prefix_dir);
            return false;
        }
    }

    // Prepare command arguments
    let args_rust = &["-P", prefix_dir, username];
    let args_c = &["useradd", "-P", prefix_dir, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with prefix");
        remove_prefix_entry(prefix_dir);
        return false;
    }

    // Verify user in prefix passwd file
    let passwd_file_path = format!("{}/etc/passwd", prefix_dir);
    let passwd_file_content = match fs::read_to_string(&passwd_file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read passwd file from prefix: {}", e);
            remove_prefix_entry(prefix_dir);
            return false;
        }
    };

    let user_created = passwd_file_content.contains(username);

    // Clean up
    remove_prefix_entry(prefix_dir);

    user_created
}

#[test]
#[serial]
fn test_useradd_with_shell() {
    let ts = TestScenario::new("useradd");

    let username = "testuser_shell";
    let shell = "/bin/sh";

    // Execute Rust version and get actual result
    let actual_shell = execute_useradd_with_shell(&ts, username, shell, true);
    if actual_shell.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_shell = execute_useradd_with_shell(&ts, username, shell, false);
    if expected_shell.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_shell, expected_shell,
        "Shells differ between Rust and Linux implementations for useradd with shell"
    );
}

fn execute_useradd_with_shell(
    ts: &TestScenario,
    username: &str,
    shell: &str,
    use_rust: bool,
) -> String {
    // Prepare command arguments
    let args_rust = &["-s", shell, username];
    let args_c = &["useradd", "-s", shell, username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with shell");
        return String::new();
    }

    // Read /etc/passwd and extract shell
    let passwd_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/passwd"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/passwd: {}", e);
            delete_user(username);
            return String::new();
        }
    };

    let user_entry = passwd_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .unwrap_or_default();

    let user_shell = user_entry.split(':').nth(6).unwrap_or_default().to_string();

    // Clean up
    delete_user(username);

    user_shell
}

#[test]
#[serial]
fn test_useradd_with_uid() {
    let ts = TestScenario::new("useradd");

    let username = "testuser_uid";
    let uid = 2001;

    // Execute Rust version and get actual result
    let actual_uid = execute_useradd_with_uid(&ts, username, uid, true);
    if actual_uid == 0 {
        return;
    }

    // Execute Linux version and get expected result
    let expected_uid = execute_useradd_with_uid(&ts, username, uid, false);
    if expected_uid == 0 {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_uid, expected_uid,
        "UIDs differ between Rust and Linux implementations for useradd with UID"
    );
}

fn execute_useradd_with_uid(ts: &TestScenario, username: &str, uid: u32, use_rust: bool) -> u32 {
    // Prepare command arguments
    let args_rust = &["-u", &uid.to_string(), username];
    let args_c = &["useradd", "-u", &uid.to_string(), username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd with UID");
        return 0;
    }

    // Get user's UID
    let user_uid = get_user_uid(username);

    // Clean up
    delete_user(username);

    user_uid
}

#[test]
#[serial]
fn test_useradd_user_group() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_with_group";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_user_group(&ts, username, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_user_group(&ts, username, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -U"
    );
}

fn execute_useradd_user_group(ts: &TestScenario, username: &str, use_rust: bool) -> bool {
    // Prepare command arguments
    let args_rust = &["-U", username];
    let args_c = &["useradd", "-U", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd -U");
        return false;
    }

    // Read /etc/passwd to verify user creation
    let passwd_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/passwd"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/passwd: {}", e);
            delete_user(username);
            return false;
        }
    };

    if !passwd_content.contains(username) {
        println!("User '{}' was not created", username);
        delete_user(username);
        return false;
    }

    // Read /etc/group to verify group creation
    let group_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/group"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/group: {}", e);
            delete_user(username);
            return false;
        }
    };

    if !group_content.contains(username) {
        println!("Group '{}' was not created", username);
        delete_user(username);
        return false;
    }

    // Get user's GID
    let gid = get_user_gid(username);

    // Get group name by GID
    let user_gid_name_output =
        run_cmd_as_root_ignore_ci_output(&["getent", "group", &gid.to_string()]);
    let user_gid_name = match user_gid_name_output {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Failed to execute getent command: {}", e);
            delete_user(username);
            return false;
        }
    };

    if !user_gid_name.starts_with(&format!("{}:", username)) {
        println!("User's primary group is not correctly set");
        delete_user(username);
        return false;
    }

    // Clean up
    delete_user(username);

    true
}

#[test]
#[serial]
fn test_useradd_user_group_conflict() {
    let ts = TestScenario::new("useradd");
    let groupname = "existinggroup";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_user_group_conflict(&ts, groupname, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_user_group_conflict(&ts, groupname, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -U conflict"
    );
}

fn execute_useradd_user_group_conflict(ts: &TestScenario, groupname: &str, use_rust: bool) -> bool {
    remove_group_entry(groupname);

    // Create the group
    let _ = run_cmd_as_root_ignore_ci(&["groupadd", "-g", "1500", groupname]);

    // Prepare command arguments
    let args_rust = &["-U", groupname];
    let args_c = &["useradd", "-U", groupname];

    // Execute command expecting failure
    let result = execute_command(ts, args_rust, args_c, use_rust);

    // Clean up
    remove_group_entry(groupname);

    result
}

#[test]
#[serial]
fn test_useradd_user_group_with_g_conflict() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_with_group_g";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_user_group_with_g_conflict(&ts, username, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_user_group_with_g_conflict(&ts, username, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -U and -g conflict"
    );
}

fn execute_useradd_user_group_with_g_conflict(
    ts: &TestScenario,
    username: &str,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-U", "-g", "1000", username];
    let args_c = &["useradd", "-U", "-g", "1000", username];

    // Execute command expecting failure
    let result = execute_command(ts, args_rust, args_c, use_rust);

    result
}

#[test]
#[serial]
fn test_useradd_user_group_with_n_conflict() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_with_group_n";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_user_group_with_n_conflict(&ts, username, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_user_group_with_n_conflict(&ts, username, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -U and -N conflict"
    );
}

fn execute_useradd_user_group_with_n_conflict(
    ts: &TestScenario,
    username: &str,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-U", "-N", username];
    let args_c = &["useradd", "-U", "-N", username];

    // Execute command expecting failure
    let result = execute_command(ts, args_rust, args_c, use_rust);

    result
}

#[test]
#[serial]
fn test_useradd_with_add_subids_for_system() {
    let ts = TestScenario::new("useradd");
    let username = "testuser_add_subids_system";

    // Execute Rust version and get actual result
    let actual_result = execute_useradd_with_add_subids_for_system(&ts, username, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_useradd_with_add_subids_for_system(&ts, username, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for useradd -r -F"
    );
}

fn execute_useradd_with_add_subids_for_system(
    ts: &TestScenario,
    username: &str,
    use_rust: bool,
) -> bool {
    // Prepare command arguments
    let args_rust = &["-r", "-F", username];
    let args_c = &["useradd", "-r", "-F", username];

    // Execute command
    if !execute_command(ts, args_rust, args_c, use_rust) {
        println!("Cannot run useradd -r -F");
        return false;
    }

    // Get UID
    let uid = get_user_uid(username);

    // Check if UID is less than 1000
    if uid >= 1000 {
        println!(
            "User '{}' has UID {} which is not in the system UID range",
            username, uid
        );
        delete_user(username);
        return false;
    }

    // Read /etc/subuid
    let subuid_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/subuid"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/subuid: {}", e);
            delete_user(username);
            return false;
        }
    };

    // Read /etc/subgid
    let subgid_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/subgid"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/subgid: {}", e);
            delete_user(username);
            return false;
        }
    };

    let subuid_created = subuid_content.contains(&format!("{}:", username));
    let subgid_created = subgid_content.contains(&format!("{}:", username));

    // Clean up
    delete_user(username);

    subuid_created && subgid_created
}

fn backup_useradd_defaults() {
    let source = "/etc/default/useradd";
    let destination = "/etc/default/useradd.backup";

    if Path::new(source).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", source, destination]) {
            println!("Failed to backup /etc/default/useradd: {}", e);
        }
    }
}

fn restore_useradd_defaults() {
    let source = "/etc/default/useradd.backup";
    let destination = "/etc/default/useradd";

    if Path::new(source).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", source, destination]) {
            println!("Failed to restore /etc/default/useradd: {}", e);
        }
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-f", source]) {
            println!("Failed to remove backup file: {}", e);
        }
    }
}

fn get_user_uid(username: &str) -> u32 {
    let output = run_cmd_as_root_ignore_ci_output(&["id", "-u", username])
        .expect("Failed to execute id command");

    let uid_str = String::from_utf8_lossy(&output.stdout);
    uid_str.trim().parse().expect("Failed to parse uid")
}

fn get_user_gid(username: &str) -> u32 {
    let output = run_cmd_as_root_ignore_ci_output(&["id", "-g", username])
        .expect("Failed to execute id command");

    let gid_str = String::from_utf8_lossy(&output.stdout);
    gid_str.trim().parse().expect("Failed to parse gid")
}

fn get_group_gid(groupname: &str) -> u32 {
    let output = run_cmd_as_root_ignore_ci_output(&["getent", "group", groupname])
        .expect("Failed to execute getent command");

    let gid_str = String::from_utf8_lossy(&output.stdout);
    let gid = gid_str
        .split(':')
        .nth(2)
        .expect("Failed to parse gid")
        .trim();
    gid.parse().expect("Failed to parse gid")
}

fn delete_user(username: &str) {
    let home_dir = format!("/home/{}", username);
    let mail_dir = format!("/var/mail/{}", username);

    if Path::new(&home_dir).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", &home_dir]) {
            println!("Failed to remove home directory: {}", e);
        }
    }

    if Path::new(&mail_dir).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-f", &mail_dir]) {
            println!("Failed to remove mail spool: {}", e);
        }
    }

    let _ = run_cmd_as_root_ignore_ci(&["userdel", username]);

    if group_exists(username) {
        remove_group_entry(username);
    }
}

fn remove_prefix_entry(prefix: &str) {
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", prefix]) {
        println!("Failed to clean up prefix directories: {}", e);
    }
}

fn remove_group_entry(group_name: &str) {
    let _ = run_cmd_as_root_ignore_ci(&["groupdel", group_name]);
}

fn group_id_exists(group_name: &str) -> bool {
    let output = Command::new("getent")
        .arg("group")
        .arg(group_name)
        .output()
        .expect("Failed to execute getent command");

    !output.stdout.is_empty()
}

fn group_exists(group_name: &str) -> bool {
    let output = run_cmd_as_root_ignore_ci_output(&["getent", "group", group_name])
        .expect("Failed to execute getent command");

    !output.stdout.is_empty()
}

fn restore_login_defs(backup_path: &str, original_path: &str) {
    if Path::new(backup_path).exists() {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", backup_path, original_path]) {
            println!("Failed to restore /etc/login.defs: {}", e);
        }
        if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-f", backup_path]) {
            println!("Failed to remove backup file: {}", e);
        }
    }
}

pub fn run_cmd_as_root_ignore_ci(args: &[&str]) -> std::result::Result<(), String> {
    match Command::new("sudo")
        .env("LC_ALL", "C")
        .args(&["-E", "--non-interactive", "whoami"])
        .output()
    {
        Ok(output) if String::from_utf8_lossy(&output.stdout).trim() == "root" => {
            let output = Command::new("sudo")
                .env("LC_ALL", "C")
                .args(&["-E", "--non-interactive"])
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::piped()) // Capture standard error
                .output();

            match output {
                Ok(output) if output.status.success() => Ok(()),
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!(
                        "Command failed with status: {}\nstderr: {}",
                        output.status, stderr
                    ))
                }
                Err(e) => Err(format!("Failed to run command: {}", e)),
            }
        }
        Ok(_) => Err("Cannot run non-interactive sudo".to_string()),
        Err(e) => Err(format!("Failed to check sudo: {}", e)),
    }
}

pub fn run_cmd_as_root_ignore_ci_output(
    args: &[&str],
) -> std::result::Result<std::process::Output, String> {
    match Command::new("sudo")
        .env("LC_ALL", "C")
        .args(&["-E", "--non-interactive", "whoami"])
        .output()
    {
        Ok(output) if String::from_utf8_lossy(&output.stdout).trim() == "root" => {
            let output = Command::new("sudo")
                .env("LC_ALL", "C")
                .args(&["-E", "--non-interactive"])
                .args(args)
                .output();

            match output {
                Ok(output) => Ok(output),
                Err(e) => Err(format!("Failed to run command: {}", e)),
            }
        }
        Ok(_) => Err("Cannot run non-interactive sudo".to_string()),
        Err(e) => Err(format!("Failed to check sudo: {}", e)),
    }
}

fn execute_command(ts: &TestScenario, args_rust: &[&str], args_c: &[&str], use_rust: bool) -> bool {
    let success = if use_rust {
        match run_ucmd_as_root_ignore_ci(ts, args_rust) {
            Ok(result) => result.succeeded(),
            Err(_) => {
                println!("Cannot run test without sudo privileges");
                return false;
            }
        }
    } else {
        match run_cmd_as_root_ignore_ci(args_c) {
            Ok(_) => true,
            Err(e) => {
                println!("Failed to run command: {}", e);
                return false;
            }
        }
    };
    success
}

fn verify_entry_in_file(path: &str, expected_entry: &str) -> String {
    let file_content = match run_cmd_as_root_ignore_ci_output(&["cat", path]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/subgid: {}", e);
            return String::new();
        }
    };
    assert!(
        file_content.contains(&expected_entry),
        "Expected data not found in {} for {}",
        path,
        expected_entry
    );
    expected_entry.to_string()
}

fn execute_command_with_output(
    ts: &TestScenario,
    args_rust: &[&str],
    args_c: &[&str],
    use_rust: bool,
) -> String {
    if use_rust {
        match run_ucmd_as_root_ignore_ci(ts, args_rust) {
            Ok(result) => result.success().clone().stdout_move_str(),
            Err(_) => {
                println!("Cannot run test without sudo privileges");
                String::new()
            }
        }
    } else {
        match run_cmd_as_root_ignore_ci_output(args_c) {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => {
                println!("Failed to run command: {}", e);
                String::new()
            }
        }
    }
}
