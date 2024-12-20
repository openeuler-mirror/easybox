//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::common::util::*;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const C_ARP_PATH: &str = "/usr/sbin/arp";
use crate::test_hwclock::run_ucmd_as_root_ignore_ci;

static MUX: Mutex<()> = Mutex::new(());

fn expected_result_brief(
    bin_path: &str,
    test_scenario: &TestScenario,
    test_args: &[&str],
) -> std::result::Result<CmdResult, String> {
    if !Path::new(bin_path).exists() {
        return Err(format!("Executable file {} not exist.", bin_path));
    }

    let result = test_scenario.cmd(bin_path).args(test_args).run();
    println!("expected code: {}", result.code());
    println!("expected stdout: {}", result.stdout_str());
    println!("expected stderr: {}", result.stderr_str());

    Ok(CmdResult::new(
        bin_path.to_string(),
        Some(test_scenario.util_name.clone()),
        Some(result.tmpd()),
        Some(result.code()),
        result.succeeded(),
        result.stdout(),
        result.stderr(),
    ))
}

fn expected_result_as_root_brief(
    bin_path: &str,
    test_scenario: &TestScenario,
    test_args: &[&str],
) -> std::result::Result<CmdResult, String> {
    if !Path::new(bin_path).exists() {
        return Err(format!("Executable file {} not exist.", bin_path));
    }

    let result = test_scenario
        .cmd("sudo")
        .env("LC_ALL", "C")
        .args(&["-E", "--non-interactive", &test_scenario.util_name])
        .args(test_args)
        .run();
    println!("expected code: {}", result.code());
    println!("expected stdout: {}", result.stdout_str());
    println!("expected stderr: {}", result.stderr_str());

    Ok(CmdResult::new(
        bin_path.to_string(),
        Some(test_scenario.util_name.clone()),
        Some(result.tmpd()),
        Some(result.code()),
        result.succeeded(),
        result.stdout(),
        result.stderr(),
    ))
}

#[test]
fn test_arp_disp() {
    let _lock = MUX.lock();
    let test_scenario = TestScenario::new(util_name!());

    let test_arg_style = ["", "-a", "-e"];
    let test_arg_verbose = ["", "-v", "--verbose"];
    let test_arg_numeric = ["", "-n", "--numeric"];

    for &style in test_arg_style.iter() {
        for &verbose in test_arg_verbose.iter() {
            for &numeric in test_arg_numeric.iter() {
                let mut test_args = vec![style, verbose, numeric];
                test_args.retain(|&x| !x.is_empty());
                let expected_result = unwrap_or_return!(expected_result_brief(
                    C_ARP_PATH,
                    &test_scenario,
                    &test_args
                ));
                test_scenario
                    .ucmd()
                    .args(&test_args)
                    .run()
                    .stdout_is(expected_result.stdout_str())
                    .stderr_is(expected_result.stderr_str())
                    .code_is(expected_result.code());
            }
        }
    }
}

#[test]
fn test_arp_set_del_file() {
    let _lock = MUX.lock();
    let test_scenario = TestScenario::new(util_name!());

    let device_1 = "veth-arp-1";
    let device_2 = "veth-arp-2";
    let address_1 = "192.168.88.1";
    let address_2 = "192.168.88.2";
    let address_3 = "192.168.88.3";
    let hwaddress = "a3:2b:07:9:0c:d7";
    let hwaddress_norm = "a3:2b:07:09:0c:d7";
    let test_args_set_1 = ["-v", "-i", device_1, "-s", address_1, hwaddress];
    let test_args_set_2 = ["-v", "-i", device_2, "-Ds", address_2, device_1];
    let test_args_file = ["-v", "-i", device_2, "-f", "ethers_file"];
    let test_args_del_1 = ["-v", "-i", device_1, "-d", address_1];
    let test_args_del_2 = ["-v", "-i", device_2, "-d", address_2];
    let test_args_del_3 = ["-v", "-i", device_2, "-d", address_3];

    match run_ucmd_as_root_ignore_ci(&test_scenario, &[]) {
        Ok(_) => {
            let privileged = test_scenario
                .cmd("sudo")
                .env("LC_ALL", "C")
                .args(&["-E", "--non-interactive"])
                .arg("bash")
                .args(&[
                    "-c", &format!("ip link add {} type veth peer name {} && ip link set {} up && ip link set {} up && ip addr add {}/24 dev {} && ip addr add {}/24 dev {}", device_1, device_2, device_1, device_2, address_1, device_1, address_2, device_2)
                ])
                .run().succeeded();

            if privileged {
                println!("Test environment is privileged.");
            } else {
                println!("Test environment is not privileged, some tests are skipped.");
            }

            test_scenario
                .cmd("sudo")
                .env("LC_ALL", "C")
                .args(&["-E", "--non-interactive"])
                .arg("bash")
                .args(&["-c", &format!("tmpfile=$(mktemp -u /tmp/tp_arp_XXXXXX) && ln -s $(which sleep) \"$tmpfile\" && $tmpfile 10 && ip link delete {} && rm $tmpfile", device_1)])
                .run_no_wait();

            thread::sleep(Duration::from_secs(1));

            if let Ok(result_set_1) = run_ucmd_as_root_ignore_ci(&test_scenario, &test_args_set_1) {
                let result_after_set_1 = test_scenario.cmd_keepenv(C_ARP_PATH).arg("-n").run();
                println!("result_after_set_1: {}", result_after_set_1.stdout_str());

                if let Ok(result_del_1) =
                    run_ucmd_as_root_ignore_ci(&test_scenario, &test_args_del_1)
                {
                    let result_after_del_1 = test_scenario.cmd_keepenv(C_ARP_PATH).arg("-n").run();
                    println!("result_after_del_1: {}", result_after_del_1.stdout_str());

                    let expected_result_set_1 = unwrap_or_return!(expected_result_as_root_brief(
                        C_ARP_PATH,
                        &test_scenario,
                        &test_args_set_1
                    ));

                    let expected_result_del_1 = unwrap_or_return!(expected_result_as_root_brief(
                        C_ARP_PATH,
                        &test_scenario,
                        &test_args_del_1
                    ));

                    result_set_1
                        .stdout_is(expected_result_set_1.stdout_str())
                        .stderr_is(expected_result_set_1.stderr_str())
                        .code_is(expected_result_set_1.code());
                    if privileged {
                        result_after_set_1
                            .stdout_contains(address_1)
                            .stdout_contains(hwaddress_norm);
                    }

                    result_del_1
                        .stdout_is(expected_result_del_1.stdout_str())
                        .stderr_is(expected_result_del_1.stderr_str())
                        .code_is(expected_result_del_1.code());
                    if privileged {
                        result_after_del_1
                            .stdout_does_not_contain(address_1)
                            .stdout_does_not_contain(hwaddress_norm);
                    }
                }
            }

            if let Ok(result_set_2) = run_ucmd_as_root_ignore_ci(&test_scenario, &test_args_set_2) {
                let result_after_set_2 = test_scenario.cmd_keepenv(C_ARP_PATH).arg("-n").run();
                println!("result_after_set_2: {}", result_after_set_2.stdout_str());

                if let Ok(result_del_2) =
                    run_ucmd_as_root_ignore_ci(&test_scenario, &test_args_del_2)
                {
                    let result_after_del_2 = test_scenario.cmd_keepenv(C_ARP_PATH).arg("-n").run();
                    println!("result_after_del_2: {}", result_after_del_2.stdout_str());

                    let expected_result_set_2 = unwrap_or_return!(expected_result_as_root_brief(
                        C_ARP_PATH,
                        &test_scenario,
                        &test_args_set_2
                    ));

                    let expected_result_del_2 = unwrap_or_return!(expected_result_as_root_brief(
                        C_ARP_PATH,
                        &test_scenario,
                        &test_args_del_2
                    ));

                    result_set_2
                        .stdout_is(expected_result_set_2.stdout_str())
                        .stderr_is(expected_result_set_2.stderr_str())
                        .code_is(expected_result_set_2.code());
                    if privileged {
                        result_after_set_2.stdout_contains(address_2);
                    }

                    result_del_2
                        .stdout_is(expected_result_del_2.stdout_str())
                        .stderr_is(expected_result_del_2.stderr_str())
                        .code_is(expected_result_del_2.code());
                    if privileged {
                        result_after_del_2.stdout_does_not_contain(address_2);
                    }
                }
            }

            if let Ok(result_file) = run_ucmd_as_root_ignore_ci(&test_scenario, &test_args_file) {
                let result_after_file = test_scenario.cmd_keepenv(C_ARP_PATH).arg("-n").run();
                println!("result_after_file: {}", result_after_file.stdout_str());

                if let Ok(result_del_3) =
                    run_ucmd_as_root_ignore_ci(&test_scenario, &test_args_del_3)
                {
                    let result_after_del_3 = test_scenario.cmd_keepenv(C_ARP_PATH).arg("-n").run();
                    println!("result_after_del_3: {}", result_after_del_3.stdout_str());

                    let expected_result_set_3 = unwrap_or_return!(expected_result_as_root_brief(
                        C_ARP_PATH,
                        &test_scenario,
                        &test_args_file
                    ));

                    let expected_result_del_3 = unwrap_or_return!(expected_result_as_root_brief(
                        C_ARP_PATH,
                        &test_scenario,
                        &test_args_del_3
                    ));

                    result_file
                        .stdout_is(expected_result_set_3.stdout_str())
                        .stderr_is(expected_result_set_3.stderr_str())
                        .code_is(expected_result_set_3.code());
                    if privileged {
                        result_after_file.stdout_contains(address_3);
                    }

                    result_del_3
                        .stdout_is(expected_result_del_3.stdout_str())
                        .stderr_is(expected_result_del_3.stderr_str())
                        .code_is(expected_result_del_3.code());
                    if privileged {
                        result_after_del_3.stdout_does_not_contain(address_3);
                    }
                }
            }

            return;
        }
        Err(e) => println!("{}", e),
    }

    println!("TEST SKIPPED");
}
