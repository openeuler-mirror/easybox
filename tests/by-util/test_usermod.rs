//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use crate::common::util::*;
use crate::test_hwclock;
use serial_test::serial;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::{Command, Stdio};
use test_hwclock::run_ucmd_as_root_ignore_ci;

#[test]
#[serial]
fn test_usermod_badname() {
    let ts = TestScenario::new("usermod");
    let username = "test_no_badname";
    let badname = "test_:_badname";

    let actual_result = execute_usermod_badname(&ts, username, badname, true);

    let expected_result = execute_usermod_badname(&ts, username, badname, false);

    assert_eq!(
        actual_result, expected_result,
        "Error messages differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_comment() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_comment";
    let initial_comment = "test_usermod_comment_c";
    let new_comment = "test_usermod_comment_rust";

    // get the rust usermod result
    let actual_result = execute_usermod_comment(&ts, username, initial_comment, new_comment, true);
    if actual_result.is_empty() {
        return;
    }

    // get the linux usermod result
    let expected_result =
        execute_usermod_comment(&ts, username, initial_comment, new_comment, false);
    if expected_result.is_empty() {
        return;
    }

    // compare the result
    assert_eq!(
        actual_result, expected_result,
        "The passwd content differs between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_expiry_date() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_expiry_user";
    let expire_date = "2028-08-21";

    // Execute Rust version and get actual result
    let actual_expiry_date = execute_usermod_expiry_date(&ts, username, expire_date, true);
    if actual_expiry_date.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_expiry_date = execute_usermod_expiry_date(&ts, username, expire_date, false);
    if expected_expiry_date.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_expiry_date, expected_expiry_date,
        "Expiry dates do not match between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_inactive() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_inactive_user";
    let inactive_days = "30";

    // Execute Rust version and get actual result
    let actual_inactive_days = execute_usermod_inactive(&ts, username, inactive_days, true);
    if actual_inactive_days.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_inactive_days = execute_usermod_inactive(&ts, username, inactive_days, false);
    if expected_inactive_days.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_inactive_days, expected_inactive_days,
        "Inactive days do not match between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_login() {
    let ts = TestScenario::new("usermod");
    let old_username = "test_usermod_login_olduser";
    let new_username = "test_usermod_login_newuser";

    // Execute Rust version and get actual result
    let actual_username = execute_usermod_login(&ts, old_username, new_username, true);
    if actual_username.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_username = execute_usermod_login(&ts, old_username, new_username, false);
    if expected_username.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_username, expected_username,
        "Usernames do not match between Rust usermod and Linux usermod"
    );
}

#[test]
#[serial]
fn test_usermod_add_subuids() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_add_subuids_user";
    let subuid_range = "200000-200999";

    // Execute Rust version and get actual result
    let actual_subuid_entry = execute_usermod_add_subuids(&ts, username, subuid_range, true);
    if actual_subuid_entry.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_subuid_entry = execute_usermod_add_subuids(&ts, username, subuid_range, false);
    if expected_subuid_entry.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_subuid_entry, expected_subuid_entry,
        "The subuid entries differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_del_subuids() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_del_subuids_user";
    let subuid_range = "300000-300999";

    // Execute Rust version and get actual result
    let actual_subuid_entry = execute_usermod_del_subuids(&ts, username, subuid_range, true);

    // Execute Linux version and get expected result
    let expected_subuid_entry = execute_usermod_del_subuids(&ts, username, subuid_range, false);

    // Compare the results
    assert_eq!(
        actual_subuid_entry, expected_subuid_entry,
        "Subuid entries differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_add_subgids() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_add_subgids_user";
    let subgid_range = "300000-300999";

    // Execute Rust usermod operation and store actual result
    let actual_result = execute_add_subgids(&ts, username, subgid_range, true);
    if actual_result.is_empty() {
        return;
    }

    // Execute Linux usermod operation and store expected result
    let expected_result = execute_add_subgids(&ts, username, subgid_range, false);
    if expected_result.is_empty() {
        return;
    }

    // Compare actual result with expected result
    assert_eq!(
        actual_result, expected_result,
        "The subgid content differs between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_del_subgids() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_del_subgids_user";
    let subgid_range = "300000-300999";

    // Execute Rust version and get actual result
    let actual_subgid_entry = execute_usermod_del_subgids(&ts, username, subgid_range, true);

    // Execute Linux version and get expected result
    let expected_subgid_entry = execute_usermod_del_subgids(&ts, username, subgid_range, false);

    // Compare the results
    assert_eq!(
        actual_subgid_entry, expected_subgid_entry,
        "Subgid entries differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_shell() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_shell_user";
    let initial_shell = "/bin/bash";
    let updated_shell = "/bin/sh";

    // Execute Rust version and get actual result
    let actual_shell = execute_usermod_shell(&ts, username, initial_shell, updated_shell, true);
    if actual_shell.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_shell = execute_usermod_shell(&ts, username, initial_shell, updated_shell, false);
    if expected_shell.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_shell, expected_shell,
        "Shell values differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_password() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_password_user";
    let password = "testpassword";

    // Execute Rust version and get actual result
    let actual_password_hash = execute_usermod_password(&ts, username, password, true);
    if actual_password_hash.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_password_hash = execute_usermod_password(&ts, username, password, false);
    if expected_password_hash.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_password_hash, expected_password_hash,
        "Password hashes differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_lock_user() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_lock_user";
    let password = "123456";

    // Execute Rust version and get actual result
    let actual_password_field = execute_usermod_lock_user(&ts, username, password, true);
    if actual_password_field.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_password_field = execute_usermod_lock_user(&ts, username, password, false);
    if expected_password_field.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_password_field, expected_password_field,
        "Password fields differ between Rust usermod and Linux usermod for {}",
        username
    );
}

#[test]
#[serial]
fn test_usermod_unlock_user() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_unlock_user";
    let password = "123456";

    // Execute Rust version and get actual result
    let actual_password_field = execute_usermod_unlock_user(&ts, username, password, true);
    if actual_password_field.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_password_field = execute_usermod_unlock_user(&ts, username, password, false);
    if expected_password_field.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_password_field, expected_password_field,
        "Password fields differ between Rust usermod and Linux usermod for {}",
        username
    );
}

// #[test]
// #[serial]
// fn test_usermod_change_home_directory() {
//     let ts = TestScenario::new("usermod");
//     let username = "test_d";
//     let new_home = "/tmp/test_d";

//     if let Ok(_) = run_cmd_as_root_ignore_ci(&["useradd", username]) {
//     } else {
//         println!("Cannot run test without sudo privileges");
//         return;
//     }

//     if !Path::new("/tmp").exists() {
//         fs::create_dir("/tmp").expect("Failed to create /tmp directory");
//     }

//     if let Ok(result) = run_ucmd_as_root_ignore_ci(&ts, &["-d", new_home, username]) {
//         result.success();
//     } else {
//         println!("Cannot run test without sudo privileges");
//         delete_user(username);
//         return;
//     }

//     let passwd_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/passwd"]) {
//         Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
//         Err(e) => {
//             println!("Cannot read /etc/passwd: {}", e);
//             delete_user(username);
//             return;
//         }
//     };

//     let user_entry = passwd_content
//         .lines()
//         .find(|line| line.contains(username))
//         .expect("User not found in /etc/passwd after modifying home directory");

//     let user_fields: Vec<&str> = user_entry.split(':').collect();
//     assert_eq!(
//         user_fields[5], new_home,
//         "Home directory for {} not updated correctly in /etc/passwd",
//         username
//     );

//     delete_user(username);
// }

#[test]
#[serial]
fn test_usermod_change_home_directory() {
    let ts = TestScenario::new("usermod");
    let username = "test_d";
    let new_home = "/tmp/test_d";

    // Execute Rust version and get actual result
    let actual_home = execute_usermod_change_home_directory(&ts, username, new_home, true);
    if actual_home.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_home = execute_usermod_change_home_directory(&ts, username, new_home, false);
    if expected_home.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_home, expected_home,
        "Home directories differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_change_home_directory(
    ts: &TestScenario,
    username: &str,
    new_home: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Ensure /tmp directory exists
    if !Path::new("/tmp").exists() {
        if let Err(e) = fs::create_dir("/tmp") {
            println!("Failed to create /tmp directory: {}", e);
            delete_user(username);
            return String::new();
        }
    }

    // Prepare command arguments
    let args_rust = &["-d", new_home, username];
    let args_c = &["usermod", "-d", new_home, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

    // Read /etc/passwd and extract the user's home directory
    let passwd_content = verify_entry_in_file("/etc/passwd", username);
    let home_directory = extract_field_from_passwd(&passwd_content, username, 5);

    // Delete user
    delete_user(username);

    home_directory
}

fn extract_field_from_passwd(passwd_content: &str, username: &str, field_index: usize) -> String {
    passwd_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(field_index))
        .unwrap_or_default()
        .to_string()
}

#[test]
#[serial]
fn test_usermod_move_home_directory() {
    let ts = TestScenario::new("usermod");
    let username = "test_move_home_directory";
    let new_home = "/tmp/test_move_home_directory";

    // Execute Rust version and get actual result
    let actual_home = execute_usermod_move_home_directory(&ts, username, new_home, true);
    if actual_home.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_home = execute_usermod_move_home_directory(&ts, username, new_home, false);
    if expected_home.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_home, expected_home,
        "Home directories differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_move_home_directory(
    ts: &TestScenario,
    username: &str,
    new_home: &str,
    use_rust: bool,
) -> String {
    // Add user with home directory
    if !add_user(&["useradd", "-m", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Ensure /tmp directory exists
    if !Path::new("/tmp").exists() {
        if let Err(e) = fs::create_dir("/tmp") {
            println!("Failed to create /tmp directory: {}", e);
            delete_user(username);
            return String::new();
        }
    }

    // Prepare command arguments
    let args_rust = &["-d", new_home, "-m", username];
    let args_c = &["usermod", "-d", new_home, "-m", username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

    // Read /etc/passwd and extract the user's home directory
    let passwd_content = verify_entry_in_file("/etc/passwd", username);
    let home_directory = extract_field_from_passwd(&passwd_content, username, 5);

    // Check if home directory exists and ownership
    let metadata = match fs::metadata(new_home) {
        Ok(meta) => meta,
        Err(e) => {
            println!("Failed to get metadata for new home directory: {}", e);
            delete_user(username);
            return String::new();
        }
    };

    let owner_uid = metadata.uid();
    let owner_name = match get_username_from_uid(owner_uid) {
        Some(name) => name,
        None => {
            println!("Failed to get username from UID");
            delete_user(username);
            return String::new();
        }
    };

    if owner_name != username {
        println!("Owner of {} is not {}", new_home, username);
        delete_user(username);
        return String::new();
    }

    // Delete user and clean up home directory
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", new_home]) {
        println!("Failed to remove new home directory: {}", e);
    }
    delete_user(username);

    home_directory
}

fn get_username_from_uid(uid: u32) -> Option<String> {
    let output = Command::new("getent")
        .args(&["passwd", &uid.to_string()])
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let user_entry = output_str.lines().next()?;
    let username = user_entry.split(':').next()?;
    Some(username.to_string())
}

#[test]
#[serial]
fn test_usermod_primary_group() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_user";
    let groupname = "test_usermod_group";

    // Execute Rust version and get actual result
    let actual_group_id = execute_usermod_primary_group(&ts, username, groupname, true);
    if actual_group_id.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_group_id = execute_usermod_primary_group(&ts, username, groupname, false);
    if expected_group_id.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_group_id, expected_group_id,
        "Primary group IDs differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_primary_group(
    ts: &TestScenario,
    username: &str,
    groupname: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Add group
    if let Err(_) = run_cmd_as_root_ignore_ci(&["groupadd", groupname]) {
        println!("Cannot add group");
        delete_user(username);
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-g", groupname, username];
    let args_c = &["usermod", "-g", groupname, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        remove_group_entry(groupname);
        return String::new();
    }

    // Read /etc/passwd and extract the user's group ID
    let passwd_content = verify_entry_in_file("/etc/passwd", username);
    let group_id = extract_field_from_passwd(&passwd_content, username, 3);

    // Delete user and group
    delete_user(username);
    remove_group_entry(groupname);

    group_id
}

#[test]
#[serial]
fn test_usermod_groups() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_groups_user";
    let group1 = "test_group1";
    let group2 = "test_group2";

    // Execute Rust version and get actual result
    let actual_groups = execute_usermod_groups(&ts, username, &[group1, group2], true);
    if actual_groups.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_groups = execute_usermod_groups(&ts, username, &[group1, group2], false);
    if expected_groups.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_groups, expected_groups,
        "Group memberships differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_groups(
    ts: &TestScenario,
    username: &str,
    groups: &[&str],
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Add groups
    for group in groups {
        if let Err(_) = run_cmd_as_root_ignore_ci(&["groupadd", group]) {
            println!("Cannot add group {}", group);
            delete_user(username);
            for g in groups {
                remove_group_entry(g);
            }
            return String::new();
        }
    }

    // Prepare command arguments
    let groups_str = groups.join(",");
    let args_rust = &["-G", &groups_str, username];
    let args_c = &["usermod", "-G", &groups_str, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        for group in groups {
            remove_group_entry(group);
        }
        return String::new();
    }

    // Read /etc/group and extract group memberships
    let group_content = verify_entry_in_file("/etc/group", "");
    let user_groups = extract_user_groups(&group_content, username);

    // Delete user and groups
    delete_user(username);
    for group in groups {
        remove_group_entry(group);
    }

    user_groups
}

fn extract_user_groups(group_content: &str, username: &str) -> String {
    let mut groups = Vec::new();
    for line in group_content.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() >= 4 && fields[3].contains(username) {
            groups.push(fields[0].to_string());
        }
    }
    groups.sort();
    groups.join(",")
}

#[test]
#[serial]
fn test_usermod_uid() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_uid_user";

    // Execute Rust version and get actual result
    let actual_uid = execute_usermod_uid(&ts, username, true);
    if actual_uid.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_uid = execute_usermod_uid(&ts, username, false);
    if expected_uid.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_uid, expected_uid,
        "UIDs differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_uid(ts: &TestScenario, username: &str, use_rust: bool) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

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
        .find(|line| line.contains(username))
        .expect("User not found in /etc/passwd");

    let user_fields: Vec<&str> = user_entry.split(':').collect();
    let initial_uid: u32 = user_fields[2].parse().expect("Failed to parse initial UID");

    let new_uid = (initial_uid + 1).to_string();

    // Prepare command arguments
    let args_rust = &["-u", &new_uid, username];
    let args_c = &["usermod", "-u", &new_uid, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

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
        .find(|line| line.contains(username))
        .expect("User not found in /etc/passwd after modifying UID");

    let user_fields: Vec<&str> = user_entry.split(':').collect();
    // Delete user
    delete_user(username);

    user_fields[2].to_string()
}

#[test]
#[serial]
fn test_usermod_non_unique_uid() {
    let ts = TestScenario::new("usermod");
    let username1 = "test_usermod_user1";
    let username2 = "test_usermod_user2";

    // Execute Rust version and get actual result
    let actual_uid = execute_usermod_non_unique_uid(&ts, username1, username2, true);
    if actual_uid.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_uid = execute_usermod_non_unique_uid(&ts, username1, username2, false);
    if expected_uid.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_uid, expected_uid,
        "UIDs differ between Rust usermod and Linux usermod for {} and {}",
        username1, username2
    );
}

fn execute_usermod_non_unique_uid(
    ts: &TestScenario,
    username1: &str,
    username2: &str,
    use_rust: bool,
) -> String {
    // Add first user
    if !add_user(&["useradd", username1]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Read UID of first user
    let passwd_content = verify_entry_in_file("/etc/passwd", username1);
    let user1_uid = extract_field_from_passwd(&passwd_content, username1, 2);

    // Add second user
    if !add_user(&["useradd", username2]) {
        println!("Cannot run test without sudo privileges");
        delete_user(username1);
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-o", "-u", &user1_uid, username2];
    let args_c = &["usermod", "-o", "-u", &user1_uid, username2];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username1);
        delete_user(username2);
        return String::new();
    }

    // Read updated UID of second user
    let passwd_content = verify_entry_in_file("/etc/passwd", username2);
    let user2_uid = extract_field_from_passwd(&passwd_content, username2, 2);

    // Delete users
    delete_user(username1);
    delete_user(username2);

    user2_uid
}

#[test]
#[serial]
fn test_usermod_append_groups() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_append_user";
    let group1 = "test_group1";
    let group2 = "test_group2";

    // Execute Rust version and get actual result
    let actual_groups = execute_usermod_append_groups(&ts, username, group1, group2, true);
    if actual_groups.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_groups = execute_usermod_append_groups(&ts, username, group1, group2, false);
    if expected_groups.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_groups, expected_groups,
        "Group memberships differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_append_groups(
    ts: &TestScenario,
    username: &str,
    group1: &str,
    group2: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Add groups
    for group in &[group1, group2] {
        if let Err(_) = run_cmd_as_root_ignore_ci(&["groupadd", group]) {
            println!("Cannot add group {}", group);
            delete_user(username);
            for g in &[group1, group2] {
                remove_group_entry(g);
            }
            return String::new();
        }
    }

    // Add user to group1
    if let Err(_) = run_cmd_as_root_ignore_ci(&["usermod", "-G", group1, username]) {
        println!("Cannot add user to group {}", group1);
        delete_user(username);
        for g in &[group1, group2] {
            remove_group_entry(g);
        }
        return String::new();
    }

    // Prepare command arguments to append group2
    let args_rust = &["-a", "-G", group2, username];
    let args_c = &["usermod", "-a", "-G", group2, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        for group in &[group1, group2] {
            remove_group_entry(group);
        }
        return String::new();
    }

    // Read /etc/group and extract group memberships
    let group_content = verify_entry_in_file("/etc/group", "");
    let user_groups = extract_user_groups(&group_content, username);

    // Delete user and groups
    delete_user(username);
    for group in &[group1, group2] {
        remove_group_entry(group);
    }

    user_groups
}

#[test]
#[serial]
fn test_usermod_remove_from_groups() {
    let ts = TestScenario::new("usermod");
    let username = "test_usermod_remove_user";
    let group1 = "test_group1";
    let group2 = "test_group2";
    let group3 = "test_group3";

    // Execute Rust version and get actual result
    let actual_groups =
        execute_usermod_remove_from_groups(&ts, username, &[group1, group2, group3], true);
    if actual_groups.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_groups =
        execute_usermod_remove_from_groups(&ts, username, &[group1, group2, group3], false);
    if expected_groups.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_groups, expected_groups,
        "Group memberships differ between Rust usermod and Linux usermod for {}",
        username
    );
}

fn execute_usermod_remove_from_groups(
    ts: &TestScenario,
    username: &str,
    groups: &[&str],
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Add groups
    for group in groups {
        if let Err(_) = run_cmd_as_root_ignore_ci(&["groupadd", group]) {
            println!("Cannot add group {}", group);
            delete_user(username);
            for g in groups {
                remove_group_entry(g);
            }
            return String::new();
        }
    }

    // Add user to all groups
    let groups_str = groups.join(",");
    if let Err(_) = run_cmd_as_root_ignore_ci(&["usermod", "-G", &groups_str, username]) {
        println!("Cannot add user to groups");
        delete_user(username);
        for g in groups {
            remove_group_entry(g);
        }
        return String::new();
    }

    // Prepare command arguments to remove from group2
    let args_rust = &["-r", "-G", groups[1], username];
    let args_c = &["usermod", "-r", "-G", groups[1], username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        for group in groups {
            remove_group_entry(group);
        }
        return String::new();
    }

    // Read /etc/group and extract group memberships
    let group_content = verify_entry_in_file("/etc/group", "");
    let user_groups = extract_user_groups(&group_content, username);

    // Delete user and groups
    delete_user(username);
    for group in groups {
        remove_group_entry(group);
    }

    user_groups
}

#[test]
#[serial]
fn test_usermod_with_prefix() {
    let ts = TestScenario::new("usermod");

    let prefix_dir = ts.fixtures.plus("myprefix").to_str().unwrap().to_string();
    ts.fixtures.mkdir("myprefix");
    ts.fixtures.mkdir("myprefix/etc");

    // Execute Rust version and get actual result
    let actual_shell = execute_usermod_with_prefix(&ts, &prefix_dir, true);
    if actual_shell.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_shell = execute_usermod_with_prefix(&ts, &prefix_dir, false);
    if expected_shell.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_shell, expected_shell,
        "Shell values differ between Rust usermod and Linux usermod for user in prefix"
    );
}

fn execute_usermod_with_prefix(ts: &TestScenario, prefix_dir: &str, use_rust: bool) -> String {
    let username = "testuser_mod_prefix";

    // Set up the prefix environment by copying necessary files
    if !setup_prefix_environment(prefix_dir) {
        println!("Failed to set up prefix environment");
        return String::new();
    }

    // Add user using useradd with prefix
    if !add_user_with_prefix(username, prefix_dir) {
        println!("Cannot run test without sudo privileges");
        remove_prefix_entry(prefix_dir);
        return String::new();
    }

    // Verify user exists in prefix passwd file
    let passwd_file_content = read_file_in_prefix(prefix_dir, "passwd");
    if !passwd_file_content.contains(username) {
        println!("User not found in {}/etc/passwd after useradd", prefix_dir);
        remove_prefix_entry(prefix_dir);
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-P", prefix_dir, "-s", "/bin/sh", username];
    let args_c = &["usermod", "-P", prefix_dir, "-s", "/bin/sh", username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command with prefix");
        remove_prefix_entry(prefix_dir);
        return String::new();
    }

    // Read passwd file in prefix and extract shell
    let passwd_file_content = read_file_in_prefix(prefix_dir, "passwd");
    let shell = extract_field_from_passwd(&passwd_file_content, username, 6);

    // Clean up
    remove_prefix_entry(prefix_dir);

    shell
}

fn setup_prefix_environment(prefix_dir: &str) -> bool {
    let files = ["passwd", "group", "shadow", "gshadow"];
    for file in &files {
        let src = format!("/etc/{}", file);
        let dest = format!("{}/etc/{}", prefix_dir, file);
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", &src, &dest]) {
            println!("Failed to copy {} to prefix: {}", file, e);
            return false;
        }
    }
    true
}

fn add_user_with_prefix(username: &str, prefix_dir: &str) -> bool {
    run_cmd_as_root_ignore_ci(&["useradd", "-P", prefix_dir, username]).is_ok()
}

fn read_file_in_prefix(prefix_dir: &str, filename: &str) -> String {
    let file_path = format!("{}/etc/{}", prefix_dir, filename);
    match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read {}: {}", file_path, e);
            String::new()
        }
    }
}

fn remove_prefix_entry(prefix_dir: &str) {
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", prefix_dir]) {
        println!("Failed to clean up prefix directories: {}", e);
    }
}

#[test]
#[serial]
fn test_usermod_with_chroot() {
    let ts = TestScenario::new("usermod");

    let chroot_dir = ts
        .fixtures
        .plus("my_test_chroot")
        .to_str()
        .unwrap()
        .to_string();
    let username = "chroot_mod_test_user";

    // Execute Rust version and get actual result
    let actual_shell = execute_usermod_with_chroot(&ts, &chroot_dir, username, true);
    if actual_shell.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_shell = execute_usermod_with_chroot(&ts, &chroot_dir, username, false);
    if expected_shell.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_shell, expected_shell,
        "Shell values differ between Rust usermod and Linux usermod in chroot environment"
    );
}

fn execute_usermod_with_chroot(
    ts: &TestScenario,
    chroot_dir: &str,
    username: &str,
    use_rust: bool,
) -> String {
    // Set up the chroot environment
    setup_chroot_environment(chroot_dir);

    // Add user using useradd with chroot
    if !add_user_with_chroot(username, chroot_dir) {
        println!("Cannot run test without sudo privileges");
        cleanup_chroot_environment(chroot_dir);
        return String::new();
    }

    // Verify user exists in chroot passwd file
    let output =
        run_cmd_as_root_ignore_ci_output(&["chroot", chroot_dir, "getent", "passwd", username]);
    match output {
        Ok(output) => {
            if output.stdout.is_empty() {
                println!(
                    "User '{}' was not correctly created in chroot environment",
                    username
                );
                cleanup_chroot_environment(chroot_dir);
                return String::new();
            }
        }
        Err(e) => {
            println!("Failed to execute getent inside chroot: {}", e);
            cleanup_chroot_environment(chroot_dir);
            return String::new();
        }
    }

    // Prepare command arguments
    let args_rust = &["-R", chroot_dir, "-s", "/bin/zsh", username];
    let args_c = &["usermod", "-R", chroot_dir, "-s", "/bin/zsh", username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command with chroot");
        cleanup_chroot_environment(chroot_dir);
        return String::new();
    }

    // Verify user's shell in chroot environment
    let output =
        run_cmd_as_root_ignore_ci_output(&["chroot", chroot_dir, "getent", "passwd", username]);
    let shell = match output {
        Ok(output) => {
            let passwd_content = String::from_utf8_lossy(&output.stdout);
            extract_field_from_passwd(&passwd_content, username, 6)
        }
        Err(e) => {
            println!("Failed to execute getent inside chroot: {}", e);
            cleanup_chroot_environment(chroot_dir);
            return String::new();
        }
    };

    // Clean up
    cleanup_chroot_environment(chroot_dir);

    shell
}

fn add_user_with_chroot(username: &str, chroot_dir: &str) -> bool {
    run_cmd_as_root_ignore_ci(&["useradd", "-R", chroot_dir, username]).is_ok()
}

fn setup_chroot_environment(chroot_dir: &str) {
    let dirs_to_create = vec![
        format!("{}/bin", chroot_dir),
        format!("{}/etc", chroot_dir),
        format!("{}/lib", chroot_dir),
        format!("{}/lib64", chroot_dir),
        format!("{}/usr/sbin", chroot_dir),
        format!("{}/usr/bin", chroot_dir),
    ];
    for dir in dirs_to_create {
        if let Err(e) = run_cmd_as_root_ignore_ci(&["mkdir", "-p", &dir]) {
            println!("Failed to create directory {}: {}", dir, e);
        }
    }

    let binaries_to_copy = vec![
        ("/bin/bash", "bin"),
        ("/usr/sbin/groupdel", "usr/sbin"),
        ("/usr/bin/getent", "usr/bin"),
    ];
    for (bin, target_dir) in binaries_to_copy {
        let dest = format!("{}/{}", chroot_dir, target_dir);
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", bin, &dest]) {
            println!("Failed to copy {} to {}: {}", bin, dest, e);
        }
    }

    let config_files_to_copy = vec!["passwd", "group", "shadow", "gshadow"];
    for file in config_files_to_copy {
        let src = format!("/etc/{}", file);
        let dest = format!("{}/etc/", chroot_dir);
        if let Err(e) = run_cmd_as_root_ignore_ci(&["cp", &src, &dest]) {
            println!("Failed to copy {} to {}: {}", src, dest, e);
        }
    }

    let all_binaries = vec![
        format!("{}/bin/bash", chroot_dir),
        format!("{}/usr/sbin/groupdel", chroot_dir),
    ];

    for bin in all_binaries.iter() {
        let output = Command::new("ldd")
            .arg(bin)
            .output()
            .expect("Failed to execute ldd command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            // Skip lines that are not relevant
            if line.contains("statically linked") {
                continue;
            }

            if let Some(pos) = line.find("=>") {
                let path_part = line[pos + 2..].trim();
                let path = path_part.split_whitespace().next().unwrap();
                if path.starts_with('/') {
                    if let Err(e) =
                        run_cmd_as_root_ignore_ci(&["cp", "--parents", path, chroot_dir])
                    {
                        println!("Failed to copy library {}: {}", path, e);
                    }
                }
            } else {
                // Line may be '/path/to/lib.so (address)'
                let path = line.trim().split_whitespace().next().unwrap();
                if path.starts_with('/') {
                    if let Err(e) =
                        run_cmd_as_root_ignore_ci(&["cp", "--parents", path, chroot_dir])
                    {
                        println!("Failed to copy library {}: {}", path, e);
                    }
                }
            }
        }
    }
}

fn cleanup_chroot_environment(chroot_dir: &str) {
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", chroot_dir]) {
        println!("Failed to remove chroot environment: {}", e);
    }
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

fn group_exists(group_name: &str) -> bool {
    let output = run_cmd_as_root_ignore_ci_output(&["getent", "group", group_name])
        .expect("Failed to execute getent command");

    !output.stdout.is_empty()
}

fn remove_group_entry(group_name: &str) {
    let _ = run_cmd_as_root_ignore_ci(&["groupdel", group_name]);
}

pub fn run_cmd_as_root_ignore_ci(args: &[&str]) -> std::result::Result<(), String> {
    let output = Command::new("sudo")
        .env("LC_ALL", "C")
        .args(&["-E", "--non-interactive", "whoami"])
        .output()
        .map_err(|e| format!("Failed to check sudo: {}", e))?;

    if String::from_utf8_lossy(&output.stdout).trim() != "root" {
        return Err("Cannot run non-interactive sudo".to_string());
    }

    let output = Command::new("sudo")
        .env("LC_ALL", "C")
        .args(&["-E", "--non-interactive"])
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "Command failed with status: {}\nstderr: {}",
            output.status, stderr
        ))
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

fn execute_usermod_badname(
    ts: &TestScenario,
    username: &str,
    badname: &str,
    use_rust: bool,
) -> bool {
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return false;
    }

    let args_rust = &["-l", badname, username];
    let args_c = &["usermod", "-l", badname, username];

    let success = execute_command(ts, args_rust, args_c, use_rust);

    delete_user(username);

    success
}

fn execute_usermod_comment(
    ts: &TestScenario,
    username: &str,
    initial_comment: &str,
    new_comment: &str,
    use_rust: bool,
) -> String {
    if !add_user(&["useradd", "-c", initial_comment, username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    let _ = match verify_entry_in_file("/etc/passwd", username) {
        content if !content.is_empty() => content,
        _ => {
            delete_user(username);
            return String::new();
        }
    };

    let args_rust = &["-c", new_comment, username];
    let args_c = &["usermod", "-c", new_comment, username];

    let success = execute_command(ts, args_rust, args_c, use_rust);

    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

    let passwd_content = match verify_entry_in_file("/etc/passwd", new_comment) {
        content if !content.is_empty() => content,
        _ => {
            delete_user(username);
            return String::new();
        }
    };

    delete_user(username);

    passwd_content
}

fn execute_usermod_expiry_date(
    ts: &TestScenario,
    username: &str,
    expire_date: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-e", expire_date, username];
    let args_c = &["usermod", "-e", expire_date, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

    // Verify entry in /etc/shadow and extract expiry date
    let shadow_content = match verify_entry_in_file("/etc/shadow", username) {
        content if !content.is_empty() => content,
        _ => {
            println!("Cannot read /etc/shadow");
            delete_user(username);
            return String::new();
        }
    };
    let expiry_date = extract_expiry_date(&shadow_content, username);

    // Delete user
    delete_user(username);

    expiry_date
}

fn extract_expiry_date(shadow_content: &str, username: &str) -> String {
    shadow_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(7))
        .unwrap_or_default()
        .to_string()
}

fn execute_usermod_inactive(
    ts: &TestScenario,
    username: &str,
    inactive_days: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-f", inactive_days, username];
    let args_c = &["usermod", "-f", inactive_days, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

    // Verify entry in /etc/shadow and extract inactive days
    let shadow_content = match verify_entry_in_file("/etc/shadow", username) {
        content if !content.is_empty() => content,
        _ => {
            println!("Cannot read /etc/shadow");
            delete_user(username);
            return String::new();
        }
    };
    let inactive_field = extract_inactive_days(&shadow_content, username);

    // Delete user
    delete_user(username);

    inactive_field
}

fn extract_inactive_days(shadow_content: &str, username: &str) -> String {
    shadow_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(6))
        .unwrap_or_default()
        .to_string()
}

fn execute_usermod_login(
    ts: &TestScenario,
    old_username: &str,
    new_username: &str,
    use_rust: bool,
) -> String {
    // Add user with old username
    if !add_user(&["useradd", old_username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-l", new_username, old_username];
    let args_c = &["usermod", "-l", new_username, old_username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(old_username);
        return String::new();
    }

    // Verify new username in /etc/passwd
    let passwd_content = match verify_entry_in_file("/etc/passwd", new_username) {
        content if !content.is_empty() => content,
        _ => {
            println!("Cannot read /etc/passwd");
            delete_user(new_username);
            return String::new();
        }
    };
    let username_field = extract_username(&passwd_content, new_username);

    // Delete user
    delete_user(new_username);

    username_field
}

fn extract_username(passwd_content: &str, username: &str) -> String {
    passwd_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(0))
        .unwrap_or_default()
        .to_string()
}

fn execute_usermod_add_subuids(
    ts: &TestScenario,
    username: &str,
    subuid_range: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-v", subuid_range, username];
    let args_c = &["usermod", "-v", subuid_range, username];

    // Execute command
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command");
        delete_user(username);
        return String::new();
    }

    // Read /etc/subuid and extract the user's entry
    let subuid_content = match verify_entry_in_file("/etc/subuid", username) {
        content if !content.is_empty() => content,
        _ => {
            println!("Cannot read /etc/subuid");
            delete_user(username);
            return String::new();
        }
    };
    let subuid_entry = extract_subuid_entry(&subuid_content, username);

    // Delete user
    delete_user(username);

    subuid_entry
}

fn extract_subuid_entry(subuid_content: &str, username: &str) -> String {
    subuid_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .unwrap_or_default()
        .to_string()
}

fn execute_usermod_del_subuids(
    ts: &TestScenario,
    username: &str,
    subuid_range: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Add subuid range using system usermod
    if let Err(e) = run_cmd_as_root_ignore_ci(&["usermod", "-v", subuid_range, username]) {
        println!("Failed to add subuid range: {}", e);
        delete_user(username);
        return String::new();
    }

    // Prepare command arguments for deleting subuids
    let args_rust = &["-V", subuid_range, username];
    let args_c = &["usermod", "-V", subuid_range, username];

    // Execute command to delete subuid range
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command to delete subuid range");
        delete_user(username);
        return String::new();
    }

    // Read /etc/subuid and extract the user's entry
    let subuid_content = match verify_entry_in_file("/etc/subuid", username) {
        content => content,
    };
    let subuid_entry = extract_subuid_entry(&subuid_content, username);

    // Delete user
    delete_user(username);

    subuid_entry
}

fn execute_add_subgids(
    ts: &TestScenario,
    username: &str,
    subgid_range: &str,
    use_rust: bool,
) -> String {
    if !add_user(&["useradd", "-U", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    let rust_args = &["-w", subgid_range, username];
    let linux_args = &["usermod", "-w", subgid_range, username];
    let success = execute_command(ts, rust_args, linux_args, use_rust);

    if !success {
        delete_user(username);
        return String::new();
    }

    // Verify in /etc/subgid
    let subgid_content = match run_cmd_as_root_ignore_ci_output(&["cat", "/etc/subgid"]) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            println!("Cannot read /etc/subgid: {}", e);
            delete_user(username);
            return String::new();
        }
    };

    let expected_entry = format!("{}:300000:1000", username);

    if !subgid_content.contains(&expected_entry) {
        delete_user(username);
        return String::new();
    }

    delete_user(username);
    return expected_entry;
}

fn execute_usermod_del_subgids(
    ts: &TestScenario,
    username: &str,
    subgid_range: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", "-U", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Add subgid range using system usermod
    if let Err(e) = run_cmd_as_root_ignore_ci(&["usermod", "-w", subgid_range, username]) {
        println!("Failed to add subgid range: {}", e);
        delete_user(username);
        return String::new();
    }

    // Prepare command arguments for deleting subgids
    let args_rust = &["-W", subgid_range, username];
    let args_c = &["usermod", "-W", subgid_range, username];

    // Execute command to delete subgid range
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command to delete subgid range");
        delete_user(username);
        return String::new();
    }

    // Read /etc/subgid and extract the user's entry
    let subgid_content = verify_entry_in_file("/etc/subgid", username);

    // Delete user
    delete_user(username);

    subgid_content
}

fn execute_usermod_shell(
    ts: &TestScenario,
    username: &str,
    initial_shell: &str,
    updated_shell: &str,
    use_rust: bool,
) -> String {
    // Add user with initial shell
    if !add_user(&["useradd", "-s", initial_shell, username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Prepare command arguments
    let args_rust = &["-s", updated_shell, username];
    let args_c = &["usermod", "-s", updated_shell, username];

    // Execute command to update shell
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to execute usermod command to update shell");
        delete_user(username);
        return String::new();
    }

    // Verify updated shell
    let updated_shell_value = verify_entry_in_file("/etc/passwd", updated_shell);

    // Delete user
    delete_user(username);

    updated_shell_value
}

fn execute_usermod_password(
    ts: &TestScenario,
    username: &str,
    password: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Set password using usermod
    let args_rust = &["-p", password, username];
    let args_c = &["usermod", "-p", password, username];

    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to set password using usermod");
        delete_user(username);
        return String::new();
    }

    // Read /etc/shadow to verify the password is set correctly
    let shadow_content = verify_entry_in_file("/etc/shadow", username);
    let shadow_password = extract_field_from_shadow(&shadow_content, username, 1);

    // Delete user
    delete_user(username);

    shadow_password
}

fn extract_field_from_shadow(shadow_content: &str, username: &str, field_index: usize) -> String {
    shadow_content
        .lines()
        .find(|line| line.starts_with(&format!("{}:", username)))
        .and_then(|line| line.split(':').nth(field_index))
        .unwrap_or_default()
        .to_string()
}

fn execute_usermod_lock_user(
    ts: &TestScenario,
    username: &str,
    password: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Generate encrypted password
    let encrypted_password = generate_encrypted_password(password);

    // Set password using usermod
    if let Err(_) = run_cmd_as_root_ignore_ci(&["usermod", "-p", &encrypted_password, username]) {
        println!("Failed to set password using usermod");
        delete_user(username);
        return String::new();
    }

    // Lock the user
    let args_rust = &["-L", username];
    let args_c = &["usermod", "-L", username];

    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to lock user using usermod");
        delete_user(username);
        return String::new();
    }

    // Verify user is locked in /etc/shadow
    let shadow_content = verify_entry_in_file("/etc/shadow", username);
    let password_field = extract_field_from_shadow(&shadow_content, username, 1);

    // Delete user
    delete_user(username);

    password_field
}

fn generate_encrypted_password(password: &str) -> String {
    let output = Command::new("openssl")
        .args(&["passwd", "-1", password])
        .output()
        .expect("Failed to execute openssl");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn execute_usermod_unlock_user(
    ts: &TestScenario,
    username: &str,
    password: &str,
    use_rust: bool,
) -> String {
    // Add user
    if !add_user(&["useradd", username]) {
        println!("Cannot run test without sudo privileges");
        return String::new();
    }

    // Generate encrypted password
    let encrypted_password = generate_encrypted_password(password);

    // Set password using usermod
    if let Err(_) = run_cmd_as_root_ignore_ci(&["usermod", "-p", &encrypted_password, username]) {
        println!("Failed to set password using usermod");
        delete_user(username);
        return String::new();
    }

    // Lock the user first
    if let Err(_) = run_cmd_as_root_ignore_ci(&["usermod", "-L", username]) {
        println!("Failed to lock user");
        delete_user(username);
        return String::new();
    }

    // Unlock the user
    let args_rust = &["-U", username];
    let args_c = &["usermod", "-U", username];

    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Failed to unlock user using usermod");
        delete_user(username);
        return String::new();
    }

    // Verify user is unlocked in /etc/shadow
    let shadow_content = verify_entry_in_file("/etc/shadow", username);
    let password_field = extract_field_from_shadow(&shadow_content, username, 1);

    // Delete user
    delete_user(username);

    password_field
}

fn add_user(args: &[&str]) -> bool {
    run_cmd_as_root_ignore_ci(args).is_ok()
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
