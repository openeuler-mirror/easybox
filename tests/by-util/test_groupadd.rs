use crate::{common::util::*, test_hwclock};
use serial_test::serial;
use std::process::{Command, Stdio};
use test_hwclock::run_ucmd_as_root_ignore_ci;

#[test]
#[serial]
fn test_groupadd_force_existing_group() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_force";

    // Execute Rust version and get actual result
    let actual_result = execute_groupadd_force_existing_group(&ts, group_name, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result = execute_groupadd_force_existing_group(&ts, group_name, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for groupadd force existing group"
    );
}

fn execute_groupadd_force_existing_group(
    ts: &TestScenario,
    group_name: &str,
    use_rust: bool,
) -> bool {
    // Remove group
    remove_group_entry(group_name);

    // Prepare command arguments for creating the group
    let args_rust = &[group_name];
    let args_c = &["groupadd", group_name];

    // Execute command to create the group
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group without sudo privileges");
        return false;
    }

    if !group_exists(group_name) {
        println!("Group {} does not exist after creation", group_name);
        return false;
    }

    // Prepare command arguments for creating the group again with --force
    let args_rust = &[group_name, "--force"];
    let args_c = &["groupadd", group_name, "--force"];

    // Execute command to create the group again with --force
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group with --force without sudo privileges");
        return false;
    }

    if !group_exists(group_name) {
        println!("Group {} does not exist after --force", group_name);
        return false;
    }

    // Clean up
    remove_group_entry(group_name);

    true
}

#[test]
#[serial]
fn test_groupadd_with_gid() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_gid";
    let gid = "1234";

    // Execute Rust version and get actual result
    let actual_gid = execute_groupadd_with_gid(&ts, group_name, gid, true);
    if actual_gid.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_gid = execute_groupadd_with_gid(&ts, group_name, gid, false);
    if expected_gid.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_gid, expected_gid,
        "GIDs differ between Rust and Linux implementations for groupadd with GID"
    );
}

fn execute_groupadd_with_gid(
    ts: &TestScenario,
    group_name: &str,
    gid: &str,
    use_rust: bool,
) -> String {
    // Remove group
    remove_group_entry(group_name);

    // Prepare command arguments
    let args_rust = &[group_name, "--gid", gid];
    let args_c = &["groupadd", "--gid", gid, group_name];

    // Execute command to create the group with specified GID
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group with GID without sudo privileges");
        return String::new();
    }

    if !group_exists(group_name) {
        println!("Group {} does not exist after creation", group_name);
        return String::new();
    }

    let gid_in_file = verify_entry_in_file("/etc/group", &format!("{}:x:{}:", group_name, gid));

    // Clean up
    remove_group_entry(group_name);

    gid_in_file
}

#[test]
#[serial]
fn test_groupadd_non_unique_gid() {
    let ts = TestScenario::new("groupadd");
    let group_name1 = "testgroup1_nonunique";
    let group_name2 = "testgroup2_nonunique";
    let gid = "1234";

    // Execute Rust version and get actual result
    let actual_result = execute_groupadd_non_unique_gid(&ts, group_name1, group_name2, gid, true);
    if !actual_result {
        return;
    }

    // Execute Linux version and get expected result
    let expected_result =
        execute_groupadd_non_unique_gid(&ts, group_name1, group_name2, gid, false);
    if !expected_result {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_result, expected_result,
        "Results differ between Rust and Linux implementations for groupadd with non-unique GID"
    );
}

fn execute_groupadd_non_unique_gid(
    ts: &TestScenario,
    group_name1: &str,
    group_name2: &str,
    gid: &str,
    use_rust: bool,
) -> bool {
    // Remove groups
    remove_group_entry(group_name1);
    remove_group_entry(group_name2);

    // Prepare command arguments for creating the first group
    let args_rust1 = &[group_name1, "--gid", gid];
    let args_c1 = &["groupadd", "--gid", gid, group_name1];

    // Execute command to create the first group
    let success = execute_command(ts, args_rust1, args_c1, use_rust);
    if !success {
        println!("Cannot create first group without sudo privileges");
        return false;
    }

    if !group_exists(group_name1) {
        println!("Group {} does not exist after creation", group_name1);
        return false;
    }

    // Prepare command arguments for creating the second group with the same GID
    let args_rust2 = &[group_name2, "--gid", gid, "--non-unique"];
    let args_c2 = &["groupadd", "--gid", gid, "--non-unique", group_name2];

    // Execute command to create the second group
    let success = execute_command(ts, args_rust2, args_c2, use_rust);
    if !success {
        println!("Cannot create second group with non-unique GID without sudo privileges");
        remove_group_entry(group_name1);
        return false;
    }

    if !group_exists(group_name2) {
        println!("Group {} does not exist after creation", group_name2);
        remove_group_entry(group_name1);
        return false;
    }

    // Clean up
    remove_group_entry(group_name1);
    remove_group_entry(group_name2);

    true
}

#[test]
#[serial]
fn test_groupadd_with_password() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_password";
    let password = "1234567890";

    // Execute Rust version and get actual result
    let actual_password = execute_groupadd_with_password(&ts, group_name, password, true);
    if actual_password.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_password = execute_groupadd_with_password(&ts, group_name, password, false);
    if expected_password.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_password, expected_password,
        "Passwords differ between Rust and Linux implementations for groupadd with password"
    );
}

fn execute_groupadd_with_password(
    ts: &TestScenario,
    group_name: &str,
    password: &str,
    use_rust: bool,
) -> String {
    // Remove group
    remove_group_entry(group_name);

    // Prepare command arguments
    let args_rust = &[group_name, "--password", password];
    let args_c = &["groupadd", "--password", password, group_name];

    // Execute command to create the group with password
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group with password without sudo privileges");
        return String::new();
    }

    if !group_exists(group_name) {
        println!("Group {} does not exist after creation", group_name);
        return String::new();
    }

    let password_in_file =
        verify_entry_in_file("/etc/gshadow", &format!("{}:{}::", group_name, password));

    // Clean up
    remove_group_entry(group_name);

    password_in_file
}

#[test]
#[serial]
fn test_groupadd_system_group() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testsystemgroup";

    // Execute Rust version and get actual result
    let actual_gid = execute_groupadd_system_group(&ts, group_name, true);
    if actual_gid.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_gid = execute_groupadd_system_group(&ts, group_name, false);
    if expected_gid.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_gid, expected_gid,
        "GIDs differ between Rust and Linux implementations for system group creation"
    );
}

fn execute_groupadd_system_group(ts: &TestScenario, group_name: &str, use_rust: bool) -> String {
    // Remove group
    remove_group_entry(group_name);

    // Prepare command arguments
    let args_rust = &[group_name, "--system"];
    let args_c = &["groupadd", "--system", group_name];

    // Execute command to create the system group
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create system group without sudo privileges");
        return String::new();
    }

    if !group_exists(group_name) {
        println!("Group {} does not exist after creation", group_name);
        return String::new();
    }

    let gid_in_file = verify_entry_in_file("/etc/gshadow", &format!("{}", group_name));

    // Clean up
    remove_group_entry(group_name);

    gid_in_file
}

#[test]
#[serial]
fn test_groupadd_with_chroot() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_with_chroot";
    let chroot_dir = ts.fixtures.plus("mychroot").to_str().unwrap().to_string();

    // Execute Rust version and get actual result
    let actual_exists = execute_groupadd_with_chroot(&ts, group_name, &chroot_dir, true);
    if !actual_exists {
        return;
    }

    // Execute Linux version and get expected result
    let expected_exists = execute_groupadd_with_chroot(&ts, group_name, &chroot_dir, false);
    if !expected_exists {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_exists, expected_exists,
        "Group existence differs between Rust and Linux implementations in chroot environment"
    );
}

fn execute_groupadd_with_chroot(
    ts: &TestScenario,
    group_name: &str,
    chroot_dir: &str,
    use_rust: bool,
) -> bool {
    // Set up chroot environment
    setup_chroot_environment(&chroot_dir);

    // Prepare command arguments
    let args_rust = &[group_name, "--root", chroot_dir];
    let args_c = &["groupadd", "--root", chroot_dir, group_name];

    // Execute command to create the group in chroot
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group in chroot without sudo privileges");
        cleanup_chroot_environment(chroot_dir);
        return false;
    }

    // Check if group exists in chroot
    let exists = group_exists_in_chroot(chroot_dir, group_name);

    // Clean up
    remove_group_entry_in_chroot(chroot_dir, group_name);
    cleanup_chroot_environment(chroot_dir);

    exists
}

fn cleanup_chroot_environment(chroot_dir: &str) {
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", chroot_dir]) {
        println!("Failed to clean up chroot directories: {}", e);
    }
}

#[test]
#[serial]
fn test_groupadd_with_prefix() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_prefix";
    let prefix_dir = ts.fixtures.plus("prefix").to_str().unwrap().to_string();

    // Execute Rust version and get actual result
    let actual_exists = execute_groupadd_with_prefix(&ts, group_name, &prefix_dir, true);
    if !actual_exists {
        return;
    }

    // Execute Linux version and get expected result
    let expected_exists = execute_groupadd_with_prefix(&ts, group_name, &prefix_dir, false);
    if !expected_exists {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_exists, expected_exists,
        "Group existence differs between Rust and Linux implementations with prefix"
    );
}

fn execute_groupadd_with_prefix(
    ts: &TestScenario,
    group_name: &str,
    prefix_dir: &str,
    use_rust: bool,
) -> bool {
    // Set up prefix environment
    ts.fixtures.mkdir("prefix");
    ts.fixtures.mkdir("prefix/etc");
    ts.fixtures.touch("prefix/etc/group");
    ts.fixtures.touch("prefix/etc/gshadow");

    // Prepare command arguments
    let args_rust = &[group_name, "--prefix", prefix_dir];
    let args_c = &["groupadd", "--prefix", prefix_dir, group_name];

    // Execute command to create the group with prefix
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group with prefix without sudo privileges");
        remove_prefix_group_entry(prefix_dir);
        return false;
    }

    let group_file_content = ts.fixtures.read("prefix/etc/group");
    if !group_file_content.contains("testgroup_prefix:x:") {
        return false;
    }
    remove_prefix_group_entry(&prefix_dir);

    true
}

#[test]
#[serial]
fn test_groupadd_with_users() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_users";
    let users = ["test_user1", "test_user2"];

    // Execute Rust version and get actual result
    let actual_users = execute_groupadd_with_users(&ts, group_name, &users, true);
    if actual_users.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_users = execute_groupadd_with_users(&ts, group_name, &users, false);
    if expected_users.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_users, expected_users,
        "User lists differ between Rust and Linux implementations for groupadd with users"
    );
}

fn execute_groupadd_with_users(
    ts: &TestScenario,
    group_name: &str,
    users: &[&str],
    use_rust: bool,
) -> String {
    // Create users
    for user in users {
        create_user(user);
    }

    // Remove group
    remove_group_entry(group_name);

    // Prepare command arguments
    let users_str = users.join(",");
    let args_rust = &[group_name, "--users", &users_str];
    let args_c = &["groupadd", "--users", &users_str, group_name];

    // Execute command to create the group with users
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group with users without sudo privileges");
        for user in users {
            remove_user(user);
        }
        return String::new();
    }

    let user_list = verify_entry_in_file("/etc/group", &format!("{}", users_str));

    // Clean up
    remove_group_entry(group_name);
    for user in users {
        remove_user(user);
    }

    user_list
}

#[test]
#[serial]
fn test_groupadd_key_override() {
    let ts = TestScenario::new("groupadd");
    let group_name = "testgroup_key_override";

    // Execute Rust version and get actual result
    let actual_gid = execute_groupadd_key_override(&ts, group_name, true);
    if actual_gid.is_empty() {
        return;
    }

    // Execute Linux version and get expected result
    let expected_gid = execute_groupadd_key_override(&ts, group_name, false);
    if expected_gid.is_empty() {
        return;
    }

    // Compare the results
    assert_eq!(
        actual_gid, expected_gid,
        "GIDs differ between Rust and Linux implementations for groupadd with key override"
    );
}

fn execute_groupadd_key_override(ts: &TestScenario, group_name: &str, use_rust: bool) -> String {
    // Remove group
    remove_group_entry(group_name);

    // Prepare command arguments
    let args_rust = &[group_name, "-K", "GID_MIN=5000", "-K", "GID_MAX=10000"];
    let args_c = &[
        "groupadd",
        "-K",
        "GID_MIN=5000",
        "-K",
        "GID_MAX=10000",
        group_name,
    ];

    // Execute command to create the group with key override
    let success = execute_command(ts, args_rust, args_c, use_rust);
    if !success {
        println!("Cannot create group with key override without sudo privileges");
        return String::new();
    }

    if !group_exists(group_name) {
        println!("Group {} does not exist after creation", group_name);
        return String::new();
    }

    let gid_in_file = verify_entry_in_file("/etc/group", &format!("{}:x:{}:", group_name, "5000"));

    // Clean up
    remove_group_entry(group_name);

    gid_in_file
}

fn remove_group_entry(group_name: &str) {
    let _ = run_cmd_as_root_ignore_ci(&["groupdel", group_name]);
}

fn group_exists(group_name: &str) -> bool {
    let output = Command::new("getent")
        .arg("group")
        .arg(group_name)
        .output()
        .expect("Failed to execute getent command");

    !output.stdout.is_empty()
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

fn group_exists_in_chroot(chroot_dir: &str, group_name: &str) -> bool {
    let output =
        run_cmd_as_root_ignore_ci_output(&["chroot", chroot_dir, "getent", "group", group_name]);
    match output {
        Ok(output) => !output.stdout.is_empty(),
        Err(e) => {
            println!("Failed to execute getent inside chroot: {}", e);
            false
        }
    }
}

fn remove_group_entry_in_chroot(chroot_dir: &str, group_name: &str) {
    if let Err(e) = run_cmd_as_root_ignore_ci(&["chroot", chroot_dir, "groupdel", group_name]) {
        println!("Failed to remove group in chroot: {}", e);
    }
}

fn remove_prefix_group_entry(prefix: &str) {
    if let Err(e) = run_cmd_as_root_ignore_ci(&["rm", "-rf", prefix]) {
        println!("Failed to clean up prefix directories: {}", e);
    }
}

fn create_user(username: &str) {
    let _ = run_cmd_as_root_ignore_ci(&["userdel", "-r", username]);

    if let Err(e) = run_cmd_as_root_ignore_ci(&["useradd", username]) {
        println!("Cannot create user without sudo privileges: {}", e);
    }
}

fn remove_user(username: &str) {
    let _ = run_cmd_as_root_ignore_ci(&["userdel", "-r", username]);
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
